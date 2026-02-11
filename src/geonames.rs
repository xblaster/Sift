//! GeoNames database for offline reverse geocoding.
//!
//! This module provides an embedded database of major cities and locations
//! worldwide, enabling offline reverse geocoding without external API calls.
//! The database includes the top major cities with populations.

use crate::clustering::GeoNameEntry;
use std::io;

/// Returns an embedded list of major GeoNames entries for reverse geocoding.
///
/// This database contains major cities worldwide and is compiled into the binary.
/// This is a minimal subset for reverse geocoding
pub fn load_geonames() -> Vec<GeoNameEntry> {
    vec![
        // Europe
        GeoNameEntry { name: "London".to_string(), latitude: 51.5074, longitude: -0.1278, population: 8_982_000 },
        GeoNameEntry { name: "Paris".to_string(), latitude: 48.8566, longitude: 2.3522, population: 2_161_000 },
        GeoNameEntry { name: "Berlin".to_string(), latitude: 52.5200, longitude: 13.4050, population: 3_645_000 },
        GeoNameEntry { name: "Madrid".to_string(), latitude: 40.4168, longitude: -3.7038, population: 3_223_000 },
        GeoNameEntry { name: "Rome".to_string(), latitude: 41.9028, longitude: 12.4964, population: 2_761_000 },
        GeoNameEntry { name: "Amsterdam".to_string(), latitude: 52.3676, longitude: 4.9041, population: 873_000 },
        GeoNameEntry { name: "Brussels".to_string(), latitude: 50.8503, longitude: 4.3517, population: 1_210_000 },
        GeoNameEntry { name: "Vienna".to_string(), latitude: 48.2082, longitude: 16.3738, population: 1_920_000 },
        GeoNameEntry { name: "Prague".to_string(), latitude: 50.0755, longitude: 14.4378, population: 1_319_000 },
        GeoNameEntry { name: "Barcelona".to_string(), latitude: 41.3851, longitude: 2.1734, population: 1_637_000 },

        // Asia
        GeoNameEntry { name: "Tokyo".to_string(), latitude: 35.6762, longitude: 139.6503, population: 37_393_000 },
        GeoNameEntry { name: "Beijing".to_string(), latitude: 39.9042, longitude: 116.4074, population: 21_540_000 },
        GeoNameEntry { name: "Shanghai".to_string(), latitude: 31.2304, longitude: 121.4737, population: 27_058_000 },
        GeoNameEntry { name: "Delhi".to_string(), latitude: 28.7041, longitude: 77.1025, population: 32_941_000 },
        GeoNameEntry { name: "Mumbai".to_string(), latitude: 19.0760, longitude: 72.8777, population: 20_962_000 },
        GeoNameEntry { name: "Bangkok".to_string(), latitude: 13.7563, longitude: 100.5018, population: 10_156_000 },
        GeoNameEntry { name: "Singapore".to_string(), latitude: 1.3521, longitude: 103.8198, population: 5_850_000 },
        GeoNameEntry { name: "Hong Kong".to_string(), latitude: 22.3193, longitude: 114.1694, population: 7_645_000 },

        // Americas
        GeoNameEntry { name: "New York".to_string(), latitude: 40.7128, longitude: -74.0060, population: 8_336_000 },
        GeoNameEntry { name: "Los Angeles".to_string(), latitude: 34.0522, longitude: -118.2437, population: 3_979_000 },
        GeoNameEntry { name: "Chicago".to_string(), latitude: 41.8781, longitude: -87.6298, population: 2_693_000 },
        GeoNameEntry { name: "Toronto".to_string(), latitude: 43.6532, longitude: -79.3832, population: 2_930_000 },
        GeoNameEntry { name: "Mexico City".to_string(), latitude: 19.4326, longitude: -99.1332, population: 21_581_000 },
        GeoNameEntry { name: "São Paulo".to_string(), latitude: -23.5505, longitude: -46.6333, population: 12_252_000 },
        GeoNameEntry { name: "Buenos Aires".to_string(), latitude: -34.6037, longitude: -58.3816, population: 15_369_000 },

        // Africa
        GeoNameEntry { name: "Cairo".to_string(), latitude: 30.0444, longitude: 31.2357, population: 21_750_000 },
        GeoNameEntry { name: "Lagos".to_string(), latitude: 6.5244, longitude: 3.3792, population: 13_463_000 },
        GeoNameEntry { name: "Johannesburg".to_string(), latitude: -26.2023, longitude: 28.0436, population: 5_635_000 },

        // Oceania
        GeoNameEntry { name: "Sydney".to_string(), latitude: -33.8688, longitude: 151.2093, population: 5_312_000 },
        GeoNameEntry { name: "Melbourne".to_string(), latitude: -37.8136, longitude: 144.9631, population: 5_159_000 },
        GeoNameEntry { name: "Auckland".to_string(), latitude: -37.0082, longitude: 174.7850, population: 1_657_000 },
    ]
}

/// Parses a single line from the GeoNames cities1000.txt file format.
///
/// This function can be used to load external GeoNames data files if you want
/// to update the embedded database with more recent data or additional locations.
///
/// # Format
///
/// The GeoNames file uses tab-separated values:
/// `geonameid\tname\tasciiname\talternatenames\tlatitude\tlongitude\t...\tpopulation\t...`
///
/// # Arguments
///
/// * `line` - A line from the GeoNames cities1000.txt file
///
/// # Returns
///
/// * `Some(GeoNameEntry)` - Successfully parsed entry
/// * `None` - If the line cannot be parsed
///
/// # Examples
///
/// ```
/// # use sift::geonames;
/// let line = "2988507\tParis\tParis\t\t48.85341\t2.3488\t\t\t\t\t\t\t\t\t2161000\t";
/// let entry = geonames::parse_geonames_line(line);
/// assert!(entry.is_some());
/// assert_eq!(entry.unwrap().name, "Paris");
/// ```
#[allow(dead_code)]
pub fn parse_geonames_line(line: &str) -> Option<GeoNameEntry> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 6 {
        return None;
    }

    let name = parts[1].to_string();
    let latitude = parts[4].parse::<f64>().ok()?;
    let longitude = parts[5].parse::<f64>().ok()?;
    let population = parts.get(14).and_then(|p| p.parse::<u32>().ok()).unwrap_or(0);

    Some(GeoNameEntry {
        name,
        latitude,
        longitude,
        population,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_geonames_not_empty() {
        let locations = load_geonames();
        assert!(!locations.is_empty());
    }

    #[test]
    fn test_load_geonames_contains_major_cities() {
        let locations = load_geonames();
        assert!(locations.iter().any(|l| l.name == "Paris"));
        assert!(locations.iter().any(|l| l.name == "Tokyo"));
        assert!(locations.iter().any(|l| l.name == "New York"));
        assert!(locations.iter().any(|l| l.name == "London"));
        assert!(locations.iter().any(|l| l.name == "Berlin"));
    }

    #[test]
    fn test_load_geonames_has_coordinates() {
        let locations = load_geonames();
        for location in locations {
            // Latitude should be between -90 and 90
            assert!(location.latitude >= -90.0 && location.latitude <= 90.0);
            // Longitude should be between -180 and 180
            assert!(location.longitude >= -180.0 && location.longitude <= 180.0);
        }
    }

    #[test]
    fn test_load_geonames_population_reasonable() {
        let locations = load_geonames();
        for location in locations {
            // Population should be non-negative and reasonable
            assert!(location.population >= 0);
            assert!(location.population < 100_000_000); // Less than 100 million
        }
    }

    #[test]
    fn test_parse_geonames_line_valid() {
        let line = "2988507\tParis\tParis\tParis city\t48.85341\t2.3488\t\t\t\t\t\t\t\t\t2161000\t";
        let entry = parse_geonames_line(line);
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert_eq!(e.name, "Paris");
        assert_eq!(e.population, 2161000);
        assert_eq!(e.latitude, 48.85341);
        assert_eq!(e.longitude, 2.3488);
    }

    #[test]
    fn test_parse_geonames_line_missing_population() {
        let line = "2988507\tParis\tParis\tParis city\t48.85341\t2.3488\t\t\t\t\t\t\t\t\t\t";
        let entry = parse_geonames_line(line);
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert_eq!(e.name, "Paris");
        assert_eq!(e.population, 0); // Should default to 0
    }

    #[test]
    fn test_parse_geonames_line_invalid_latitude() {
        let line = "2988507\tParis\tParis\tParis city\tinvalid\t2.3488\t\t\t\t\t\t\t\t\t2161000\t";
        let entry = parse_geonames_line(line);
        assert!(entry.is_none());
    }

    #[test]
    fn test_parse_geonames_line_invalid_longitude() {
        let line = "2988507\tParis\tParis\tParis city\t48.85341\tinvalid\t\t\t\t\t\t\t\t\t2161000\t";
        let entry = parse_geonames_line(line);
        assert!(entry.is_none());
    }

    #[test]
    fn test_parse_geonames_line_too_short() {
        let line = "2988507\tParis\tParis";
        let entry = parse_geonames_line(line);
        assert!(entry.is_none());
    }

    #[test]
    fn test_parse_geonames_line_empty() {
        let line = "";
        let entry = parse_geonames_line(line);
        assert!(entry.is_none());
    }

    #[test]
    fn test_parse_geonames_line_zero_population() {
        let line = "123\tTestCity\tTestCity\t\t10.0\t20.0\t\t\t\t\t\t\t\t\t0\t";
        let entry = parse_geonames_line(line);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().population, 0);
    }

    #[test]
    fn test_load_geonames_all_have_names() {
        let locations = load_geonames();
        for location in locations {
            assert!(!location.name.is_empty());
        }
    }
}
