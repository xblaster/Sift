use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct GeoPoint {
    pub id: usize,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone)]
pub struct GeoNameEntry {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub population: u32,
}

/// Calculate distance in kilometers between two points using Haversine formula
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

/// DBSCAN clustering algorithm
/// Returns a map of cluster_id -> list of point IDs
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
        while !seed_set.is_empty() {
            let current_point_id = seed_set.pop().unwrap();

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

/// Find the closest GeoName entry to a point
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
    fn test_haversine_distance() {
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
    fn test_dbscan_clustering() {
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
    fn test_find_closest_location() {
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
}
