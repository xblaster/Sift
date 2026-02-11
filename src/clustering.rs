//! Geographic clustering and reverse geocoding functionality.
//!
//! This module implements the DBSCAN clustering algorithm with Haversine
//! distance metric for grouping photos by geographic location. It also provides
//! reverse geocoding to find the nearest named location for a cluster.
//!
//! # Examples
//!
//! Cluster geographic points:
//! ```no_run
//! # use sift::clustering::{GeoPoint, dbscan};
//! let points = vec![
//!     GeoPoint { id: 0, latitude: 48.8566, longitude: 2.3522 },
//!     GeoPoint { id: 1, latitude: 48.8567, longitude: 2.3523 },
//! ];
//! let clusters = dbscan(&points, 1.0, 2); // 1km radius, min 2 points
//! println!("Found {} clusters", clusters.len());
//! ```

use std::collections::{HashMap, HashSet};

/// A geographic point with latitude and longitude coordinates.
///
/// # Fields
///
/// * `id` - Unique identifier for the point
/// * `latitude` - Latitude in decimal degrees (-90 to 90)
/// * `longitude` - Longitude in decimal degrees (-180 to 180)
#[derive(Debug, Clone)]
pub struct GeoPoint {
    pub id: usize,
    pub latitude: f64,
    pub longitude: f64,
}

/// A named geographic location from the GeoNames database.
///
/// # Fields
///
/// * `name` - Name of the location (city, town, etc.)
/// * `latitude` - Latitude of the location
/// * `longitude` - Longitude of the location
/// * `population` - Population of the location (0 if unknown)
#[derive(Debug, Clone)]
pub struct GeoNameEntry {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub population: u32,
}

/// Calculates the distance in kilometers between two geographic points.
///
/// Uses the Haversine formula to compute great-circle distance on Earth.
/// This formula is more accurate than simple Euclidean distance for
/// geographic coordinates.
///
/// # Arguments
///
/// * `point1` - First geographic point
/// * `point2` - Second geographic point
///
/// # Returns
///
/// Distance in kilometers
///
/// # Examples
///
/// ```
/// # use sift::clustering::{GeoPoint, haversine_distance};
/// let paris = GeoPoint {
///     id: 0,
///     latitude: 48.8566,
///     longitude: 2.3522,
/// };
/// let london = GeoPoint {
///     id: 1,
///     latitude: 51.5074,
///     longitude: -0.1278,
/// };
/// let distance = haversine_distance(&paris, &london);
/// assert!((distance - 344.0).abs() < 5.0); // ~344 km
/// ```
pub fn haversine_distance(point1: &GeoPoint, point2: &GeoPoint) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = point1.latitude.to_radians();
    let lat2_rad = point2.latitude.to_radians();
    let delta_lat = (point2.latitude - point1.latitude).to_radians();
    let delta_lon = (point2.longitude - point1.longitude).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

/// Performs DBSCAN clustering on geographic points.
///
/// DBSCAN (Density-Based Spatial Clustering of Applications with Noise) groups
/// points that are close together, identifying clusters of arbitrary shape.
/// Points in clusters are marked as core points or border points, while isolated
/// points are considered noise.
///
/// # Arguments
///
/// * `points` - Slice of geographic points to cluster
/// * `eps_km` - Maximum distance in kilometers between points in a cluster
/// * `min_points` - Minimum number of points to form a cluster
///
/// # Returns
///
/// A HashMap where keys are cluster IDs and values are vectors of point IDs
///
/// # Examples
///
/// ```
/// # use sift::clustering::{GeoPoint, dbscan};
/// let points = vec![
///     GeoPoint { id: 0, latitude: 0.0, longitude: 0.0 },
///     GeoPoint { id: 1, latitude: 0.01, longitude: 0.01 },
///     GeoPoint { id: 2, latitude: 10.0, longitude: 10.0 },
/// ];
/// let clusters = dbscan(&points, 2.0, 2);
/// // Points 0 and 1 are close and form a cluster
/// ```
pub fn dbscan(points: &[GeoPoint], eps_km: f64, min_points: usize) -> HashMap<usize, Vec<usize>> {
    let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut visited = HashSet::new();
    let mut cluster_id = 0;

    for point in points {
        if visited.contains(&point.id) {
            continue;
        }

        let neighbors = find_neighbors(point, points, eps_km);

        if neighbors.len() < min_points {
            // Mark as noise, not assigned to any cluster
            visited.insert(point.id);
            continue;
        }

        // Start a new cluster
        let mut current_cluster = vec![point.id];
        visited.insert(point.id);

        let mut seed_set = neighbors;
        while let Some(current_point_id) = seed_set.pop() {
            

            if !visited.contains(&current_point_id) {
                visited.insert(current_point_id);

                let current_point = &points[current_point_id];
                let neighbors_of_current = find_neighbors(current_point, points, eps_km);

                if neighbors_of_current.len() >= min_points {
                    for neighbor_id in neighbors_of_current {
                        if !visited.contains(&neighbor_id) {
                            seed_set.push(neighbor_id);
                        }
                    }
                }

                current_cluster.push(current_point_id);
            }
        }

        if !current_cluster.is_empty() {
            clusters.insert(cluster_id, current_cluster);
            cluster_id += 1;
        }
    }

    clusters
}

/// Find all neighbors within eps_km of a point
fn find_neighbors(point: &GeoPoint, points: &[GeoPoint], eps_km: f64) -> Vec<usize> {
    points
        .iter()
        .filter(|p| {
            p.id != point.id && haversine_distance(point, p) <= eps_km
        })
        .map(|p| p.id)
        .collect()
}

/// Finds the closest named location to a geographic point.
///
/// Performs reverse geocoding by finding the nearest GeoNames entry to the
/// given point. Useful for assigning human-readable location names to clusters.
///
/// # Arguments
///
/// * `point` - The geographic point to find the closest location for
/// * `locations` - Slice of available GeoNames entries
///
/// # Returns
///
/// * `Some(String)` - Name of the closest location
/// * `None` - If no locations are provided
///
/// # Examples
///
/// ```
/// # use sift::clustering::{GeoPoint, GeoNameEntry, find_closest_location};
/// let point = GeoPoint {
///     id: 0,
///     latitude: 48.8566,
///     longitude: 2.3522,
/// };
/// let locations = vec![
///     GeoNameEntry {
///         name: "Paris".to_string(),
///         latitude: 48.8566,
///         longitude: 2.3522,
///         population: 2_161_000,
///     },
/// ];
/// let closest = find_closest_location(&point, &locations);
/// assert_eq!(closest, Some("Paris".to_string()));
/// ```
pub fn find_closest_location(point: &GeoPoint, locations: &[GeoNameEntry]) -> Option<String> {
    if locations.is_empty() {
        return None;
    }

    locations
        .iter()
        .map(|loc| {
            let distance = haversine_distance(
                point,
                &GeoPoint {
                    id: 0,
                    latitude: loc.latitude,
                    longitude: loc.longitude,
                },
            );
            (loc.name.clone(), distance)
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(name, _)| name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance_paris_london() {
        // Distance between Paris and London (approximately 344 km)
        let paris = GeoPoint {
            id: 0,
            latitude: 48.8566,
            longitude: 2.3522,
        };
        let london = GeoPoint {
            id: 1,
            latitude: 51.5074,
            longitude: -0.1278,
        };

        let distance = haversine_distance(&paris, &london);
        assert!((distance - 344.0).abs() < 5.0); // Within 5km tolerance
    }

    #[test]
    fn test_haversine_distance_same_point() {
        let point = GeoPoint {
            id: 0,
            latitude: 48.8566,
            longitude: 2.3522,
        };

        let distance = haversine_distance(&point, &point);
        assert!(distance < 0.001); // Should be near zero
    }

    #[test]
    fn test_haversine_distance_antipodal() {
        // Points on opposite sides of Earth
        let north = GeoPoint {
            id: 0,
            latitude: 90.0,
            longitude: 0.0,
        };
        let south = GeoPoint {
            id: 1,
            latitude: -90.0,
            longitude: 0.0,
        };

        let distance = haversine_distance(&north, &south);
        assert!((distance - 20015.0).abs() < 100.0); // Half Earth circumference ~20015 km
    }

    #[test]
    fn test_haversine_distance_commutative() {
        let point1 = GeoPoint {
            id: 0,
            latitude: 48.8566,
            longitude: 2.3522,
        };
        let point2 = GeoPoint {
            id: 1,
            latitude: 51.5074,
            longitude: -0.1278,
        };

        let dist_1_to_2 = haversine_distance(&point1, &point2);
        let dist_2_to_1 = haversine_distance(&point2, &point1);
        assert!((dist_1_to_2 - dist_2_to_1).abs() < 0.001);
    }

    #[test]
    fn test_dbscan_clustering_basic() {
        let points = vec![
            GeoPoint { id: 0, latitude: 0.0, longitude: 0.0 },
            GeoPoint { id: 1, latitude: 0.01, longitude: 0.01 },
            GeoPoint { id: 2, latitude: 0.02, longitude: 0.02 },
            GeoPoint { id: 3, latitude: 10.0, longitude: 10.0 },
            GeoPoint { id: 4, latitude: 10.01, longitude: 10.01 },
        ];

        let clusters = dbscan(&points, 2.0, 2);
        assert!(clusters.len() >= 1);
    }

    #[test]
    fn test_dbscan_single_point() {
        let points = vec![GeoPoint { id: 0, latitude: 0.0, longitude: 0.0 }];
        let clusters = dbscan(&points, 2.0, 2);
        assert_eq!(clusters.len(), 0); // Single point can't form a cluster with min_points=2
    }

    #[test]
    fn test_dbscan_no_clusters() {
        // Points too far apart to cluster
        let points = vec![
            GeoPoint { id: 0, latitude: 0.0, longitude: 0.0 },
            GeoPoint { id: 1, latitude: 45.0, longitude: 45.0 },
            GeoPoint { id: 2, latitude: -45.0, longitude: -45.0 },
        ];

        let clusters = dbscan(&points, 1.0, 2); // Very tight epsilon
        assert_eq!(clusters.len(), 0);
    }

    #[test]
    fn test_dbscan_tight_cluster() {
        // Points very close together
        let points = vec![
            GeoPoint { id: 0, latitude: 48.8566, longitude: 2.3522 },
            GeoPoint { id: 1, latitude: 48.8567, longitude: 2.3523 },
            GeoPoint { id: 2, latitude: 48.8568, longitude: 2.3524 },
        ];

        let clusters = dbscan(&points, 1.0, 2); // 1km radius should capture these
        assert!(clusters.len() >= 1);
    }

    #[test]
    fn test_dbscan_empty_list() {
        let points = vec![];
        let clusters = dbscan(&points, 2.0, 2);
        assert_eq!(clusters.len(), 0);
    }

    #[test]
    fn test_find_closest_location_exact_match() {
        let point = GeoPoint {
            id: 0,
            latitude: 48.8566,
            longitude: 2.3522,
        };

        let locations = vec![
            GeoNameEntry {
                name: "Paris".to_string(),
                latitude: 48.8566,
                longitude: 2.3522,
                population: 2_161_000,
            },
        ];

        let closest = find_closest_location(&point, &locations);
        assert_eq!(closest, Some("Paris".to_string()));
    }

    #[test]
    fn test_find_closest_location_multiple() {
        let point = GeoPoint {
            id: 0,
            latitude: 48.8566,
            longitude: 2.3522,
        };

        let locations = vec![
            GeoNameEntry {
                name: "Paris".to_string(),
                latitude: 48.8566,
                longitude: 2.3522,
                population: 2_161_000,
            },
            GeoNameEntry {
                name: "London".to_string(),
                latitude: 51.5074,
                longitude: -0.1278,
                population: 8_982_000,
            },
        ];

        let closest = find_closest_location(&point, &locations);
        assert_eq!(closest, Some("Paris".to_string()));
    }

    #[test]
    fn test_find_closest_location_empty() {
        let point = GeoPoint {
            id: 0,
            latitude: 48.8566,
            longitude: 2.3522,
        };

        let locations = vec![];
        let closest = find_closest_location(&point, &locations);
        assert_eq!(closest, None);
    }

    #[test]
    fn test_find_closest_location_distant() {
        let point = GeoPoint {
            id: 0,
            latitude: 48.8566,
            longitude: 2.3522,
        };

        let locations = vec![
            GeoNameEntry {
                name: "Tokyo".to_string(),
                latitude: 35.6762,
                longitude: 139.6503,
                population: 37_393_000,
            },
            GeoNameEntry {
                name: "New York".to_string(),
                latitude: 40.7128,
                longitude: -74.0060,
                population: 8_336_000,
            },
        ];

        let closest = find_closest_location(&point, &locations);
        // London is closer to Paris than either of these, but Tokyo or NY should be chosen
        assert!(closest.is_some());
    }

    #[test]
    fn test_geo_point_creation() {
        let point = GeoPoint {
            id: 0,
            latitude: 48.8566,
            longitude: 2.3522,
        };

        assert_eq!(point.id, 0);
        assert_eq!(point.latitude, 48.8566);
        assert_eq!(point.longitude, 2.3522);
    }

    #[test]
    fn test_geoname_entry_creation() {
        let entry = GeoNameEntry {
            name: "Paris".to_string(),
            latitude: 48.8566,
            longitude: 2.3522,
            population: 2_161_000,
        };

        assert_eq!(entry.name, "Paris");
        assert_eq!(entry.population, 2_161_000);
    }
}
