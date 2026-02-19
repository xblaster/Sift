//! Zero-download OneDrive photo organization via Microsoft Graph API.
//!
//! # The key insight
//!
//! The Microsoft Graph API exposes exactly what Sift needs to run its full
//! organization pipeline without transferring a single byte of photo data:
//!
//! | Sift operation      | Graph API source                        |
//! |---------------------|-----------------------------------------|
//! | Capture date        | `photo.takenDateTime` (server EXIF)     |
//! | GPS for clustering  | `location.latitude/longitude`           |
//! | Deduplication hash  | `file.hashes.quickXorHash` (server-side)|
//! | Incremental scan    | Delta API with `@odata.deltaLink`       |
//! | Organize in-place   | `PATCH /items/{id}` (move, no download) |
//!
//! # Authentication
//!
//! Uses OAuth2 Device Code Flow — the user visits a URL, enters a short code,
//! and the token is cached at `~/.config/sift/onedrive_token.json` for reuse.
//! A refresh token keeps the session alive across runs.
//!
//! # Delta sync
//!
//! After the initial full scan, a deltaLink is stored in
//! `~/.config/sift/onedrive_delta.json`. Subsequent runs use it to fetch only
//! changed items — making large-library rescans near-instant.
//!
//! # Example
//!
//! ```no_run
//! use sift::onedrive::{OneDriveClient, DeltaState, OneDrivePipeline, PipelineConfig};
//!
//! let mut client = OneDriveClient::authenticate("your-azure-client-id").unwrap();
//! let config = PipelineConfig { dry_run: true, dest_folder: "Organized".into() };
//! let mut pipeline = OneDrivePipeline::new(client, config);
//! let stats = pipeline.run().unwrap();
//! println!("Organized {} photos, skipped {} duplicates", stats.organized, stats.duplicates);
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::{DateTime, Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

const GRAPH_API: &str = "https://graph.microsoft.com/v1.0";
const TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const DEVICE_CODE_URL: &str =
    "https://login.microsoftonline.com/common/oauth2/v2.0/devicecode";
/// Scopes required: read/write files and maintain a refresh token.
const SCOPES: &str = "Files.ReadWrite offline_access";

// ─── Graph API response types (private) ──────────────────────────────────────

/// A single item returned by the Graph API delta endpoint.
#[derive(Debug, Deserialize)]
struct DriveItem {
    id: String,
    name: String,
    photo: Option<PhotoFacet>,
    location: Option<LocationFacet>,
    file: Option<FileFacet>,
    /// Non-null when the item has been deleted since the last delta call.
    deleted: Option<serde_json::Value>,
    #[serde(rename = "parentReference")]
    parent_reference: Option<ParentReference>,
}

#[derive(Debug, Deserialize)]
struct PhotoFacet {
    /// ISO 8601 capture timestamp extracted server-side from EXIF.
    #[serde(rename = "takenDateTime")]
    taken_date_time: Option<String>,
    #[serde(rename = "cameraMake")]
    camera_make: Option<String>,
    #[serde(rename = "cameraModel")]
    camera_model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LocationFacet {
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct FileFacet {
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    hashes: Option<FileHashes>,
}

/// Server-computed content hashes. `quickXorHash` is guaranteed present on
/// both OneDrive Personal and OneDrive for Business.
#[derive(Debug, Deserialize)]
struct FileHashes {
    /// A fast XOR-based hash computed by OneDrive servers.
    /// Identical files always share the same hash — perfect for deduplication
    /// without downloading anything.
    #[serde(rename = "quickXorHash")]
    quick_xor_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ParentReference {
    id: Option<String>,
    /// OneDrive path, e.g. `/drive/root:/Photos/Vacation`.
    path: Option<String>,
}

// ─── Token types (private) ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StoredToken {
    access_token: String,
    refresh_token: Option<String>,
    /// Unix timestamp after which this token must be refreshed.
    expires_at_unix: i64,
}

impl StoredToken {
    /// Returns true if the token will remain valid for at least 5 more minutes.
    fn is_valid(&self) -> bool {
        chrono::Utc::now().timestamp() < self.expires_at_unix - 300
    }
}

/// Raw token endpoint response (fields are absent on error responses).
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
    /// Human-readable message already formatted by Microsoft.
    message: Option<String>,
}

// ─── Public types ────────────────────────────────────────────────────────────

/// A photo record fetched from OneDrive using only Graph API metadata.
///
/// Contains everything Sift's pipeline needs — date, GPS, hash — with
/// zero bytes of photo data downloaded.
#[derive(Debug, Clone)]
pub struct OneDriveRecord {
    /// OneDrive item ID (stable across renames/moves).
    pub item_id: String,
    /// File name including extension.
    pub name: String,
    /// Capture date from EXIF, extracted server-side by OneDrive.
    pub taken_date: Option<NaiveDate>,
    /// GPS position from photo EXIF, available without downloading.
    pub location: Option<(f64, f64)>,
    /// Server-computed quickXorHash — use this for deduplication instead of
    /// locally computing a Blake3 hash.
    pub quick_xor_hash: Option<String>,
    /// Human-readable camera description (e.g. "Apple iPhone 15 Pro").
    pub camera: Option<String>,
    /// Parent folder path in OneDrive (e.g. `/drive/root:/Photos`).
    pub parent_path: Option<String>,
    /// Parent folder ID, needed for move operations.
    pub parent_id: Option<String>,
    /// True when the delta API reports this item was deleted.
    pub deleted: bool,
}

/// Persisted state for incremental (delta) scans.
///
/// Stored at `~/.config/sift/onedrive_delta.json`.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DeltaState {
    /// The deltaLink returned by the last completed scan.
    /// Pass this to the next scan to receive only changed items.
    pub delta_link: Option<String>,
    /// Maps `quickXorHash → item_id` for deduplication across runs.
    /// Populated from items seen in previous scans.
    pub seen_hashes: HashMap<String, String>,
}

impl DeltaState {
    /// Load state from `~/.config/sift/onedrive_delta.json`, or return a
    /// fresh default (triggering a full scan on the next call).
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = match Self::state_path() {
            Some(p) => p,
            None => return Ok(Self::default()),
        };
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    /// Persist the current state to disk so the next run uses delta sync.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::state_path().ok_or("Cannot determine config directory")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Reset: clears the deltaLink so the next scan is a full library scan.
    pub fn reset(&mut self) {
        self.delta_link = None;
        self.seen_hashes.clear();
    }

    fn state_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("sift").join("onedrive_delta.json"))
    }
}

/// Configuration for the zero-download organization pipeline.
pub struct PipelineConfig {
    /// When true, print planned moves but do not call the Graph API move endpoint.
    pub dry_run: bool,
    /// Name of the top-level destination folder in OneDrive root.
    /// Photos will be moved to `/{dest_folder}/YYYY/MM/DD/`.
    pub dest_folder: String,
}

/// Summary statistics returned by [`OneDrivePipeline::run`].
#[derive(Debug, Default)]
pub struct PipelineStats {
    /// Total photo items returned by the delta scan (includes deletions).
    pub total_scanned: usize,
    /// Photos after removing duplicates (same quickXorHash seen before).
    pub unique_photos: usize,
    /// Photos skipped because their hash matched a previously seen file.
    pub duplicates: usize,
    /// Photos successfully moved (or that would be moved in dry-run mode).
    pub organized: usize,
    /// Photos skipped because no capture date was available.
    pub no_date: usize,
}

// ─── OneDrive Graph API client ───────────────────────────────────────────────

/// Authenticated Graph API client.
///
/// Obtain one via [`OneDriveClient::authenticate`], which handles the full
/// OAuth2 Device Code Flow and token caching automatically.
pub struct OneDriveClient {
    http: reqwest::blocking::Client,
    token: StoredToken,
    client_id: String,
}

impl OneDriveClient {
    /// Authenticate with Microsoft and return a ready-to-use client.
    ///
    /// On first run this prints a URL and short code for the user to visit.
    /// On subsequent runs it silently reloads or refreshes the cached token.
    ///
    /// # Arguments
    ///
    /// * `client_id` — Azure AD Application (client) ID registered as a
    ///   "Mobile and desktop application" with `http://localhost` redirect URI.
    ///   Set via `SIFT_ONEDRIVE_CLIENT_ID` env var or pass directly.
    pub fn authenticate(client_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        // 1. Try cached token
        if let Some(token) = Self::load_cached_token()? {
            if token.is_valid() {
                return Ok(Self { http, token, client_id: client_id.to_string() });
            }
            // 2. Try refreshing the cached token
            if let Some(ref_tok) = token.refresh_token.clone() {
                if let Ok(refreshed) = Self::do_refresh(&http, client_id, &ref_tok) {
                    Self::save_token(&refreshed)?;
                    return Ok(Self { http, token: refreshed, client_id: client_id.to_string() });
                }
            }
        }

        // 3. Full Device Code Flow
        let token = Self::device_code_flow(&http, client_id)?;
        Self::save_token(&token)?;
        Ok(Self { http, token, client_id: client_id.to_string() })
    }

    /// Scan photos from OneDrive using the Graph API delta endpoint.
    ///
    /// Pass `delta_state.delta_link = None` for a full library scan (first run),
    /// or a stored deltaLink for an incremental scan (subsequent runs).
    ///
    /// Returns `(records, new_delta_link)`. Store the new deltaLink in
    /// `DeltaState` and call [`DeltaState::save`] so the next run is incremental.
    ///
    /// # What is fetched
    ///
    /// Each request selects `photo,location,file,deleted,parentReference` —
    /// the server returns pre-extracted EXIF metadata and a server-computed hash.
    /// No file content is transferred.
    pub fn scan_photos(
        &mut self,
        delta_state: &DeltaState,
    ) -> Result<(Vec<OneDriveRecord>, String), Box<dyn std::error::Error>> {
        let select = "id,name,photo,location,file,deleted,parentReference";
        let start_url = match &delta_state.delta_link {
            Some(link) => link.clone(),
            None => format!("{}/me/drive/root/delta?$select={}", GRAPH_API, select),
        };

        let mut records = Vec::new();
        let mut url = start_url;
        let mut delta_link = String::new();

        loop {
            let resp = self.get_json(&url)?;

            let items: Vec<DriveItem> =
                serde_json::from_value(resp["value"].clone())
                    .unwrap_or_default();

            for item in items {
                // Only keep image files and deleted items (to track removals).
                let is_image = item
                    .file
                    .as_ref()
                    .and_then(|f| f.mime_type.as_deref())
                    .map(|m| m.starts_with("image/"))
                    .unwrap_or(false)
                    || item.photo.is_some();

                if !is_image && item.deleted.is_none() {
                    continue;
                }

                records.push(Self::drive_item_to_record(item));
            }

            // Follow pagination or stop at deltaLink.
            if let Some(next) = resp["@odata.nextLink"].as_str() {
                url = next.to_string();
            } else if let Some(delta) = resp["@odata.deltaLink"].as_str() {
                delta_link = delta.to_string();
                break;
            } else {
                break;
            }
        }

        Ok((records, delta_link))
    }

    /// Move a drive item to a different parent folder.
    ///
    /// This is a metadata-only operation on OneDrive's side — the file bytes
    /// never travel to the client.
    pub fn move_item(
        &mut self,
        item_id: &str,
        new_parent_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/me/drive/items/{}", GRAPH_API, item_id);
        let body = serde_json::json!({
            "parentReference": { "id": new_parent_id }
        });
        self.patch_json(&url, &body)?;
        Ok(())
    }

    /// Return the item ID of the OneDrive root folder.
    pub fn get_root_id(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let resp = self.get_json(&format!("{}/me/drive/root", GRAPH_API))?;
        resp["id"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "Missing root id in Graph API response".into())
    }

    /// Get or create a named child folder under `parent_id`.
    ///
    /// Uses a GET-before-POST pattern: first checks whether the folder exists
    /// (avoiding duplicate creation), then creates it only if absent.
    pub fn get_or_create_folder(
        &mut self,
        parent_id: &str,
        folder_name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Sanitize the name so it is safe to embed in a URL path segment.
        let safe_name = sanitize_folder_name(folder_name);

        // Try to look up the folder by name under the parent.
        let lookup_url = format!(
            "{}/me/drive/items/{}:/{}",
            GRAPH_API, parent_id, safe_name
        );
        if let Ok(resp) = self.get_json(&lookup_url) {
            if let Some(id) = resp["id"].as_str() {
                return Ok(id.to_string());
            }
        }

        // Folder not found — create it.
        let create_url = format!("{}/me/drive/items/{}/children", GRAPH_API, parent_id);
        let body = serde_json::json!({
            "name": safe_name,
            "folder": {},
            // "fail" keeps names clean; we handle 409 with the GET above.
            "@microsoft.graph.conflictBehavior": "fail"
        });
        let resp = self.post_json(&create_url, &body)?;
        resp["id"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "Missing id in folder creation response".into())
    }

    // ─── Private helpers ──────────────────────────────────────────────────────

    fn drive_item_to_record(item: DriveItem) -> OneDriveRecord {
        let taken_date = item
            .photo
            .as_ref()
            .and_then(|p| p.taken_date_time.as_deref())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.naive_local().date());

        let location = item.location.as_ref().and_then(|l| {
            match (l.latitude, l.longitude) {
                (Some(lat), Some(lon)) => Some((lat, lon)),
                _ => None,
            }
        });

        let quick_xor_hash = item
            .file
            .as_ref()
            .and_then(|f| f.hashes.as_ref())
            .and_then(|h| h.quick_xor_hash.clone());

        let camera = item.photo.as_ref().and_then(|p| {
            match (&p.camera_make, &p.camera_model) {
                (Some(make), Some(model)) => Some(format!("{} {}", make, model)),
                (Some(make), None) => Some(make.clone()),
                _ => None,
            }
        });

        OneDriveRecord {
            item_id: item.id,
            name: item.name,
            taken_date,
            location,
            quick_xor_hash,
            camera,
            parent_path: item
                .parent_reference
                .as_ref()
                .and_then(|r| r.path.clone()),
            parent_id: item
                .parent_reference
                .as_ref()
                .and_then(|r| r.id.clone()),
            deleted: item.deleted.is_some(),
        }
    }

    fn get_json(
        &mut self,
        url: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        self.ensure_token_valid()?;
        let resp = self
            .http
            .get(url)
            .bearer_auth(&self.token.access_token)
            .send()?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            return Err(format!("Graph API GET {}: {}", status, body).into());
        }
        Ok(resp.json()?)
    }

    fn patch_json(
        &mut self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        self.ensure_token_valid()?;
        let resp = self
            .http
            .patch(url)
            .bearer_auth(&self.token.access_token)
            .json(body)
            .send()?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            return Err(format!("Graph API PATCH {}: {}", status, text).into());
        }
        Ok(resp.json()?)
    }

    fn post_json(
        &mut self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        self.ensure_token_valid()?;
        let resp = self
            .http
            .post(url)
            .bearer_auth(&self.token.access_token)
            .json(body)
            .send()?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            return Err(format!("Graph API POST {}: {}", status, text).into());
        }
        Ok(resp.json()?)
    }

    fn ensure_token_valid(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.token.is_valid() {
            let refresh_token = self
                .token
                .refresh_token
                .clone()
                .ok_or("Token expired and no refresh token available. Re-authenticate.")?;
            let refreshed = Self::do_refresh(&self.http, &self.client_id, &refresh_token)?;
            Self::save_token(&refreshed)?;
            self.token = refreshed;
        }
        Ok(())
    }

    // ─── OAuth2 Device Code Flow ──────────────────────────────────────────────

    fn device_code_flow(
        http: &reqwest::blocking::Client,
        client_id: &str,
    ) -> Result<StoredToken, Box<dyn std::error::Error>> {
        // Step 1 — request a device code.
        let dc: DeviceCodeResponse = http
            .post(DEVICE_CODE_URL)
            .form(&[("client_id", client_id), ("scope", SCOPES)])
            .send()?
            .json()?;

        // Step 2 — show instructions.
        println!("\n[OneDrive] Sign in to authorize Sift:");
        if let Some(msg) = &dc.message {
            println!("{}", msg);
        } else {
            println!("  Visit: {}", dc.verification_uri);
            println!("  Code:  {}", dc.user_code);
        }
        println!();

        // Step 3 — poll until the user completes sign-in.
        let poll_interval = Duration::from_secs(dc.interval.max(5));
        let deadline = Instant::now() + Duration::from_secs(dc.expires_in);

        while Instant::now() < deadline {
            std::thread::sleep(poll_interval);

            let resp: TokenResponse = http
                .post(TOKEN_URL)
                .form(&[
                    ("client_id", client_id),
                    ("grant_type", "urn:ietf:params:oauth2:grant-type:device_code"),
                    ("device_code", &dc.device_code),
                ])
                .send()?
                .json()?;

            match resp.error.as_deref() {
                None => {
                    // Success
                    return Self::build_token(resp);
                }
                Some("authorization_pending") => continue,
                Some("slow_down") => {
                    std::thread::sleep(Duration::from_secs(5));
                }
                Some(err) => {
                    return Err(format!(
                        "Auth error {}: {}",
                        err,
                        resp.error_description.unwrap_or_default()
                    )
                    .into());
                }
            }
        }

        Err("Authentication timed out — user did not complete sign-in".into())
    }

    fn do_refresh(
        http: &reqwest::blocking::Client,
        client_id: &str,
        refresh_token: &str,
    ) -> Result<StoredToken, Box<dyn std::error::Error>> {
        let resp: TokenResponse = http
            .post(TOKEN_URL)
            .form(&[
                ("client_id", client_id),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("scope", SCOPES),
            ])
            .send()?
            .json()?;

        if let Some(err) = &resp.error {
            return Err(format!(
                "Token refresh error {}: {}",
                err,
                resp.error_description.unwrap_or_default()
            )
            .into());
        }
        Self::build_token(resp)
    }

    fn build_token(resp: TokenResponse) -> Result<StoredToken, Box<dyn std::error::Error>> {
        let access_token =
            resp.access_token.ok_or("Token response missing access_token")?;
        let expires_in = resp.expires_in.unwrap_or(3600);
        let expires_at_unix = chrono::Utc::now().timestamp() + expires_in as i64;
        Ok(StoredToken {
            access_token,
            refresh_token: resp.refresh_token,
            expires_at_unix,
        })
    }

    // ─── Token persistence ────────────────────────────────────────────────────

    fn token_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("sift").join("onedrive_token.json"))
    }

    fn load_cached_token() -> Result<Option<StoredToken>, Box<dyn std::error::Error>> {
        let path = match Self::token_path() {
            Some(p) => p,
            None => return Ok(None),
        };
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(path)?;
        Ok(Some(serde_json::from_str(&data)?))
    }

    fn save_token(token: &StoredToken) -> Result<(), Box<dyn std::error::Error>> {
        let path = match Self::token_path() {
            Some(p) => p,
            None => return Ok(()),
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(token)?)?;
        Ok(())
    }
}

// ─── Zero-download organization pipeline ─────────────────────────────────────

/// Runs the complete Sift pipeline against OneDrive using only Graph API calls.
///
/// No file data is downloaded. The pipeline:
///
/// 1. **Scan** — fetch photo metadata via delta API (`photo`, `location`, `file` facets)
/// 2. **Deduplicate** — compare `quickXorHash` against the stored `seen_hashes` index
/// 3. **Organize** — move unique photos to `/{dest_folder}/YYYY/MM/DD/` via PATCH
/// 4. **Persist** — save the new deltaLink so the next run is incremental
pub struct OneDrivePipeline {
    client: OneDriveClient,
    config: PipelineConfig,
}

impl OneDrivePipeline {
    /// Create a new pipeline with the given authenticated client and config.
    pub fn new(client: OneDriveClient, config: PipelineConfig) -> Self {
        Self { client, config }
    }

    /// Execute the pipeline and return statistics.
    pub fn run(&mut self) -> Result<PipelineStats, Box<dyn std::error::Error>> {
        let mut stats = PipelineStats::default();
        let mut delta_state = DeltaState::load()?;

        // ── Stage 1: Scan ──────────────────────────────────────────────────
        let scan_mode = if delta_state.delta_link.is_some() {
            "incremental (delta)"
        } else {
            "full library"
        };
        println!("Scanning OneDrive photos ({})...", scan_mode);

        let (records, new_delta_link) = self.client.scan_photos(&delta_state)?;
        stats.total_scanned = records.len();
        println!("  Found {} photo items", records.len());

        // ── Stage 2: Deduplicate ───────────────────────────────────────────
        // quickXorHash is computed server-side; we compare it against a local
        // set of hashes seen in previous runs. No local I/O on photo files.
        let mut unique: Vec<&OneDriveRecord> = Vec::new();

        for record in &records {
            if record.deleted {
                // Remove from seen_hashes so the file can re-appear if re-uploaded.
                if let Some(hash) = &record.quick_xor_hash {
                    delta_state.seen_hashes.remove(hash);
                }
                continue;
            }

            match &record.quick_xor_hash {
                Some(hash) if delta_state.seen_hashes.contains_key(hash) => {
                    stats.duplicates += 1;
                }
                Some(hash) => {
                    delta_state
                        .seen_hashes
                        .insert(hash.clone(), record.item_id.clone());
                    unique.push(record);
                }
                None => {
                    // No hash available (rare) — include to avoid data loss.
                    unique.push(record);
                }
            }
        }
        stats.unique_photos = unique.len();

        // ── Stage 3: Resolve destination root ─────────────────────────────
        let root_id = self.client.get_root_id()?;
        // Cache folder IDs so we issue at most one API call per folder segment.
        let mut folder_cache: HashMap<String, String> = HashMap::new();

        // ── Stage 4: Organize ──────────────────────────────────────────────
        for record in unique {
            let date = match record.taken_date {
                Some(d) => d,
                None => {
                    stats.no_date += 1;
                    if self.config.dry_run {
                        println!(
                            "  [skip] {} — no capture date in metadata",
                            record.name
                        );
                    }
                    continue;
                }
            };

            let dest_path = format!(
                "{}/{}/{:02}/{:02}",
                self.config.dest_folder,
                date.year(),
                date.month(),
                date.day()
            );

            if self.config.dry_run {
                let camera_note = record
                    .camera
                    .as_deref()
                    .map(|c| format!(" [{}]", c))
                    .unwrap_or_default();
                println!("  [dry-run] {} → /{}/ {}", record.name, dest_path, camera_note);
                stats.organized += 1;
                continue;
            }

            // Resolve (or create) the destination folder hierarchy.
            match self.ensure_hierarchy(&root_id, &dest_path, &mut folder_cache) {
                Ok(dest_id) => {
                    // Skip if the file is already in the right folder.
                    let already_there = record
                        .parent_id
                        .as_deref()
                        .map(|pid| pid == dest_id)
                        .unwrap_or(false);

                    if already_there {
                        continue;
                    }

                    match self.client.move_item(&record.item_id, &dest_id) {
                        Ok(()) => {
                            stats.organized += 1;
                        }
                        Err(e) => {
                            eprintln!("  [warn] Could not move {}: {}", record.name, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  [warn] Could not create {}: {}", dest_path, e);
                }
            }
        }

        // ── Stage 5: Persist delta state ───────────────────────────────────
        delta_state.delta_link = Some(new_delta_link);
        delta_state.save()?;

        Ok(stats)
    }

    /// Walk a slash-separated path (e.g. `"Organized/2023/07/15"`) and ensure
    /// each folder segment exists, creating missing ones via the Graph API.
    ///
    /// Results are cached in `folder_cache` to avoid redundant API calls for
    /// photos that share a date.
    fn ensure_hierarchy(
        &mut self,
        root_id: &str,
        path: &str,
        folder_cache: &mut HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut current_id = root_id.to_string();
        let mut cumulative = String::new();

        for segment in path.split('/').filter(|s| !s.is_empty()) {
            if !cumulative.is_empty() {
                cumulative.push('/');
            }
            cumulative.push_str(segment);

            if let Some(cached_id) = folder_cache.get(&cumulative) {
                current_id = cached_id.clone();
            } else {
                let new_id =
                    self.client.get_or_create_folder(&current_id, segment)?;
                folder_cache.insert(cumulative.clone(), new_id.clone());
                current_id = new_id;
            }
        }

        Ok(current_id)
    }
}

// ─── Utilities ───────────────────────────────────────────────────────────────

/// Remove characters that are illegal in OneDrive folder names and trim whitespace.
fn sanitize_folder_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_illegal_chars() {
        assert_eq!(sanitize_folder_name("hello/world"), "hello_world");
        assert_eq!(sanitize_folder_name("a:b*c?d"), "a_b_c_d");
        assert_eq!(sanitize_folder_name("  spaces  "), "spaces");
    }

    #[test]
    fn stored_token_validity() {
        let valid = StoredToken {
            access_token: "tok".into(),
            refresh_token: None,
            expires_at_unix: chrono::Utc::now().timestamp() + 3600,
        };
        assert!(valid.is_valid());

        let expired = StoredToken {
            access_token: "tok".into(),
            refresh_token: None,
            expires_at_unix: chrono::Utc::now().timestamp() - 1,
        };
        assert!(!expired.is_valid());
    }

    /// Verify that a DriveItem with a photo facet is correctly mapped to
    /// an OneDriveRecord with parsed date, GPS, and hash.
    #[test]
    fn drive_item_to_record_full() {
        let item = DriveItem {
            id: "abc123".into(),
            name: "photo.jpg".into(),
            photo: Some(PhotoFacet {
                taken_date_time: Some("2023-07-15T14:30:00Z".into()),
                camera_make: Some("Apple".into()),
                camera_model: Some("iPhone 15 Pro".into()),
            }),
            location: Some(LocationFacet {
                latitude: Some(48.8566),
                longitude: Some(2.3522),
            }),
            file: Some(FileFacet {
                mime_type: Some("image/jpeg".into()),
                hashes: Some(FileHashes {
                    quick_xor_hash: Some("base64hash==".into()),
                }),
            }),
            deleted: None,
            parent_reference: Some(ParentReference {
                id: Some("parent_id".into()),
                path: Some("/drive/root:/Photos".into()),
            }),
        };

        let record = OneDriveClient::drive_item_to_record(item);

        assert_eq!(record.item_id, "abc123");
        assert_eq!(record.name, "photo.jpg");
        assert_eq!(
            record.taken_date,
            Some(NaiveDate::from_ymd_opt(2023, 7, 15).unwrap())
        );
        assert_eq!(record.location, Some((48.8566, 2.3522)));
        assert_eq!(record.quick_xor_hash.as_deref(), Some("base64hash=="));
        assert_eq!(record.camera.as_deref(), Some("Apple iPhone 15 Pro"));
        assert!(!record.deleted);
    }

    #[test]
    fn drive_item_deleted_flag() {
        let item = DriveItem {
            id: "del1".into(),
            name: "gone.jpg".into(),
            photo: None,
            location: None,
            file: None,
            deleted: Some(serde_json::json!({})),
            parent_reference: None,
        };
        let record = OneDriveClient::drive_item_to_record(item);
        assert!(record.deleted);
    }

    #[test]
    fn delta_state_default_triggers_full_scan() {
        let state = DeltaState::default();
        assert!(state.delta_link.is_none());
        assert!(state.seen_hashes.is_empty());
    }
}
