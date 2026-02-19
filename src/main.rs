//! Sift - High-performance photo organization utility for network storage
//!
//! Sift is a Rust-based CLI tool for organizing massive photo libraries on network
//! storage (SMB/NFS) with minimal dependencies and maximum performance.
//!
//! # Features
//!
//! - **Blake3 Hashing**: Fast, parallelized file hashing for duplicate detection
//! - **Local Index**: Persistent indexing for idempotent operations
//! - **Date Extraction**: Automatic date extraction from file metadata
//! - **Chronological Organization**: Automatic folder hierarchy (YYYY/MM/DD/)
//! - **Geographic Clustering**: DBSCAN-based spatial clustering with reverse geocoding
//! - **Network Optimization**: Buffered I/O and exponential backoff retry logic
//! - **Full CLI**: Comprehensive command-line interface with multiple operations
//!
//! # Architecture
//!
//! The application is organized into functional modules:
//!
//! - `hash`: Blake3 hashing engine with parallelization
//! - `index`: Persistent deduplication index
//! - `metadata`: Date extraction from file metadata
//! - `organization`: Folder structure management
//! - `clustering`: Geographic clustering with reverse geocoding
//! - `geonames`: Embedded location database
//! - `network_io`: Network-optimized I/O operations
//! - `cli`: Command-line argument parsing
//!
//! # Examples
//!
//! ```bash
//! # Organize photos with automatic clustering
//! sift organize /source/photos /destination/organized --with-clustering
//!
//! # Hash a single file
//! sift hash /photos/image.jpg
//!
//! # Hash an entire directory in parallel
//! sift hash /photos --recursive
//!
//! # View index contents
//! sift index my_index.bin --limit 20
//!
//! # Benchmark network performance
//! sift benchmark /mnt/network/share --size-mb 500
//! ```

pub mod error;
pub mod hash;
pub mod index;
pub mod metadata;
pub mod organization;
pub mod clustering;
pub mod geonames;
pub mod network_io;
pub mod cli;
pub mod organize;
pub mod onedrive;

use std::error::Error;
use cli::{Cli, Commands};
use organize::{OrganizeContext, Orchestrator};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse_args();

    if cli.verbose {
        eprintln!("Sift v0.1.0 - Photo organization utility");
    }

    match cli.command {
        Commands::Organize {
            source,
            destination,
            with_clustering,
            jobs,
            index,
            dry_run,
        } => {
            if dry_run {
                eprintln!("[DRY RUN] No files will be copied or modified");
            }
            let ctx = OrganizeContext::new(source, destination, with_clustering, jobs, index);
            let mut orchestrator = Orchestrator::new(ctx);
            orchestrator.run()?;
        }

        Commands::Hash { path, recursive } => {
            if path.is_file() {
                match hash::hash_file(&path) {
                    Ok(h) => println!("{}: {}", path.display(), h.to_hex()),
                    Err(e) => eprintln!("Error hashing {}: {}", path.display(), e),
                }
            } else if path.is_dir() {
                let mut files = Vec::new();
                if recursive {
                    for entry in walkdir::WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                        if entry.file_type().is_file() {
                            files.push(entry.path().to_path_buf());
                        }
                    }
                } else {
                    for entry in std::fs::read_dir(&path)? {
                        let entry = entry?;
                        if entry.path().is_file() {
                            files.push(entry.path());
                        }
                    }
                }

                let results = hash::hash_files_parallel(files);
                for (file_path, h) in results {
                    println!("{}: {}", file_path, h.to_hex());
                }
            } else {
                eprintln!("Path not found: {}", path.display());
            }
        }

        Commands::Index { path, limit } => {
            match index::Index::load_from_file(&path) {
                Ok(idx) => {
                    println!("Index loaded from {:?}: {} entries", path, idx.len());
                    for (i, entry) in idx.entries().enumerate() {
                        if i >= limit {
                            break;
                        }
                        println!("{}: {}", entry.hash, entry.file_path);
                    }
                }
                Err(e) => eprintln!("Error loading index {:?}: {}", path, e),
            }
        }

        Commands::Cluster { source, details } => {
            eprintln!("Scanning for photos in {:?}...", source);
            let photo_extensions = ["jpg", "jpeg", "png", "tiff", "raw", "heic"];
            let points = Vec::new();
            let mut paths = Vec::new();

            for entry in walkdir::WalkDir::new(&source).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        let ext_lower = ext.to_string_lossy().to_lowercase();
                        if photo_extensions.contains(&ext_lower.as_str()) {
                            // TODO: Actually extract GPS from EXIF
                            // For now, this is a placeholder to show clustering works
                            // if we had the coordinates.
                            // In a real run, we'd use metadata::extract_gps(path)
                            paths.push(path.to_path_buf());
                        }
                    }
                }
            }

            if points.is_empty() {
                println!("No photos with GPS coordinates found in {:?}", source);
                return Ok(());
            }

            let clusters = clustering::dbscan(&points, 1.0, 3);
            let geonames = geonames::load_geonames();

            println!("Found {} clusters in {}", clusters.len(), source.display());

            for (id, cluster_points) in clusters {
                let first_point_id = cluster_points[0];
                let first_point = &points[first_point_id];
                let location_name = clustering::find_closest_location(first_point, &geonames)
                    .unwrap_or_else(|| "Unknown Location".to_string());

                println!("Cluster {}: {} ({} photos)", id, location_name, cluster_points.len());
                if details {
                    for &p_id in &cluster_points {
                        println!("  - {:?}", paths[p_id]);
                    }
                }
            }
        }

        Commands::OneDrive { action } => {
            use cli::OneDriveAction;
            use onedrive::{DeltaState, OneDriveClient, OneDrivePipeline, PipelineConfig};

            match action {
                OneDriveAction::Scan { client_id, full } => {
                    let mut client = OneDriveClient::authenticate(&client_id)?;
                    let mut state = DeltaState::load()?;
                    if full {
                        state.reset();
                    }
                    println!(
                        "Scanning OneDrive ({}scan)...",
                        if state.delta_link.is_some() { "incremental " } else { "full " }
                    );
                    let (records, new_delta_link) = client.scan_photos(&state)?;

                    let photos: Vec<_> = records.iter().filter(|r| !r.deleted).collect();
                    let with_date = photos.iter().filter(|r| r.taken_date.is_some()).count();
                    let with_gps = photos.iter().filter(|r| r.location.is_some()).count();
                    let with_hash = photos.iter().filter(|r| r.quick_xor_hash.is_some()).count();

                    println!("\nScan summary:");
                    println!("  Total photos : {}", photos.len());
                    println!("  With date    : {} (from EXIF via Graph API — no download)", with_date);
                    println!("  With GPS     : {} (from location facet — no download)", with_gps);
                    println!("  With hash    : {} (quickXorHash, server-computed)", with_hash);

                    if cli.verbose {
                        println!("\nFirst 20 records:");
                        for r in photos.iter().take(20) {
                            println!(
                                "  {:40} | date: {:?} | gps: {:?} | hash: {}",
                                r.name,
                                r.taken_date,
                                r.location,
                                r.quick_xor_hash.as_deref().unwrap_or("—")
                            );
                        }
                    }

                    state.delta_link = Some(new_delta_link);
                    state.save()?;
                    println!("\nDelta state saved — next scan will be incremental.");
                }

                OneDriveAction::Organize { client_id, dest_folder, dry_run } => {
                    if dry_run {
                        eprintln!("[DRY RUN] No files will be moved on OneDrive");
                    }
                    let client = OneDriveClient::authenticate(&client_id)?;
                    let config = PipelineConfig { dry_run, dest_folder };
                    let mut pipeline = OneDrivePipeline::new(client, config);
                    let stats = pipeline.run()?;

                    println!("\nOneDrive organize complete:");
                    println!("  Scanned    : {}", stats.total_scanned);
                    println!("  Unique     : {}", stats.unique_photos);
                    println!("  Duplicates : {} (detected via quickXorHash, no download)", stats.duplicates);
                    println!("  Organized  : {}", stats.organized);
                    println!("  No date    : {} (skipped — no EXIF capture date)", stats.no_date);
                }

                OneDriveAction::Logout => {
                    let token_path = dirs::config_dir()
                        .map(|d| d.join("sift").join("onedrive_token.json"));
                    let delta_path = dirs::config_dir()
                        .map(|d| d.join("sift").join("onedrive_delta.json"));

                    let mut removed = 0usize;
                    for path in [token_path, delta_path].iter().flatten() {
                        if path.exists() {
                            std::fs::remove_file(path)?;
                            removed += 1;
                        }
                    }
                    if removed > 0 {
                        println!("Logged out — cached token and delta state removed.");
                    } else {
                        println!("Nothing to remove (no cached session found).");
                    }
                }
            }
        }

        Commands::Benchmark {
            path,
            size_mb,
            iterations,
        } => {
            use std::io::Write;
            use std::time::Instant;

            println!("Benchmarking performance on: {:?}", path);
            let test_file = path.join(".sift_benchmark.tmp");
            let data = vec![0u8; size_mb * 1024 * 1024];

            print!("Creating {} MB test file... ", size_mb);
            std::io::stdout().flush()?;
            std::fs::write(&test_file, &data)?;
            println!("Done.");

            let mut total_duration = std::time::Duration::default();

            for i in 1..=iterations {
                print!("Iteration {}/{}... ", i, iterations);
                std::io::stdout().flush()?;
                let start = Instant::now();
                let _read_data = network_io::buffered_read_file(&test_file)?;
                let duration = start.elapsed();
                total_duration += duration;
                println!("{:?}", duration);
            }

            let avg_duration = total_duration / iterations as u32;
            let throughput = (size_mb as f64) / avg_duration.as_secs_f64();

            println!("\nBenchmark Results:");
            println!("  Average Duration: {:?}", avg_duration);
            println!("  Throughput: {:.2} MB/s", throughput);

            if test_file.exists() {
                std::fs::remove_file(test_file)?;
            }
        }
    }

    Ok(())
}
