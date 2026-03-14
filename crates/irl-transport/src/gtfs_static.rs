// Static GTFS data (stops.txt, routes.txt) from Transport for Ireland.
// Downloaded once and cached locally at ~/.irl/data/gtfs/.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use irl_core::config::Config;
use irl_core::error::IrlError;

const GTFS_ZIP_URL: &str = "https://www.transportforireland.ie/transitData/Data/GTFS_All.zip";
const CACHE_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 3600); // 7 days

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GtfsStop {
    pub stop_id: String,
    pub stop_name: String,
    pub stop_lat: f64,
    pub stop_lon: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GtfsRoute {
    pub route_id: String,
    pub route_short_name: String,
    pub route_long_name: String,
    pub route_type: String,
}

pub struct GtfsData {
    pub stops: Vec<GtfsStop>,
    pub routes: Vec<GtfsRoute>,
    stops_by_id: HashMap<String, usize>,
    routes_by_id: HashMap<String, usize>,
}

impl GtfsData {
    fn gtfs_dir() -> PathBuf {
        Config::data_dir().join("gtfs")
    }

    fn is_cached_and_fresh() -> bool {
        let dir = Self::gtfs_dir();
        let stops_path = dir.join("stops.txt");
        let routes_path = dir.join("routes.txt");

        if !stops_path.exists() || !routes_path.exists() {
            return false;
        }

        // Check age
        if let Ok(metadata) = fs::metadata(&stops_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = SystemTime::now().duration_since(modified) {
                    return age < CACHE_MAX_AGE;
                }
            }
        }

        false
    }

    /// Download the GTFS zip and extract stops.txt and routes.txt.
    pub async fn download(verbose: bool, quiet: bool) -> Result<(), IrlError> {
        let dir = Self::gtfs_dir();
        fs::create_dir_all(&dir).map_err(|e| IrlError::Other(format!("Failed to create GTFS dir: {}", e)))?;

        let zip_path = dir.join("gtfs_all.zip");

        if !quiet {
            eprintln!("Downloading GTFS static data from Transport for Ireland...");
        }

        let client = irl_core::http::HttpClient::new(verbose, quiet)?;
        let bytes = client.get_bytes(GTFS_ZIP_URL).await?;
        fs::write(&zip_path, &bytes)
            .map_err(|e| IrlError::Other(format!("Failed to write GTFS zip: {}", e)))?;

        if verbose {
            eprintln!("  Downloaded {} bytes to {}", bytes.len(), zip_path.display());
        }

        // Extract stops.txt and routes.txt using unzip
        let output = Command::new("unzip")
            .args(["-o", "-j"])
            .arg(zip_path.to_str().unwrap())
            .args(["stops.txt", "routes.txt"])
            .arg("-d")
            .arg(dir.to_str().unwrap())
            .output()
            .map_err(|e| IrlError::Other(format!("Failed to run unzip: {}", e)))?;

        if !output.status.success() {
            return Err(IrlError::Other(format!(
                "unzip failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Remove the zip to save space
        let _ = fs::remove_file(&zip_path);

        if !quiet {
            eprintln!("GTFS data cached at {}", dir.display());
        }

        Ok(())
    }

    /// Load GTFS data from cache, downloading if needed.
    pub async fn load(verbose: bool, quiet: bool) -> Result<Self, IrlError> {
        if !Self::is_cached_and_fresh() {
            Self::download(verbose, quiet).await?;
        }

        let dir = Self::gtfs_dir();
        let stops = Self::parse_stops(&dir.join("stops.txt"))?;
        let routes = Self::parse_routes(&dir.join("routes.txt"))?;

        let stops_by_id: HashMap<String, usize> = stops
            .iter()
            .enumerate()
            .map(|(i, s)| (s.stop_id.clone(), i))
            .collect();

        let routes_by_id: HashMap<String, usize> = routes
            .iter()
            .enumerate()
            .map(|(i, r)| (r.route_id.clone(), i))
            .collect();

        Ok(Self {
            stops,
            routes,
            stops_by_id,
            routes_by_id,
        })
    }

    fn parse_stops(path: &PathBuf) -> Result<Vec<GtfsStop>, IrlError> {
        let mut file = fs::File::open(path)
            .map_err(|e| IrlError::Other(format!("Failed to open stops.txt: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| IrlError::Other(format!("Failed to read stops.txt: {}", e)))?;

        let mut reader = csv::Reader::from_reader(contents.as_bytes());
        let mut stops = Vec::new();

        for result in reader.records() {
            let record = result.map_err(|e| IrlError::Parse(format!("CSV parse error: {}", e)))?;
            // stop_id,stop_code,stop_name,stop_desc,stop_lat,stop_lon,...
            if record.len() >= 6 {
                let lat = record[4].parse::<f64>().unwrap_or(0.0);
                let lon = record[5].parse::<f64>().unwrap_or(0.0);
                if lat != 0.0 && lon != 0.0 {
                    stops.push(GtfsStop {
                        stop_id: record[0].to_string(),
                        stop_name: record[2].to_string(),
                        stop_lat: lat,
                        stop_lon: lon,
                    });
                }
            }
        }

        Ok(stops)
    }

    fn parse_routes(path: &PathBuf) -> Result<Vec<GtfsRoute>, IrlError> {
        let mut file = fs::File::open(path)
            .map_err(|e| IrlError::Other(format!("Failed to open routes.txt: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| IrlError::Other(format!("Failed to read routes.txt: {}", e)))?;

        let mut reader = csv::Reader::from_reader(contents.as_bytes());
        let mut routes = Vec::new();

        for result in reader.records() {
            let record = result.map_err(|e| IrlError::Parse(format!("CSV parse error: {}", e)))?;
            // route_id,agency_id,route_short_name,route_long_name,route_desc,route_type,...
            if record.len() >= 6 {
                routes.push(GtfsRoute {
                    route_id: record[0].to_string(),
                    route_short_name: record[2].to_string(),
                    route_long_name: record[3].to_string(),
                    route_type: record[5].to_string(),
                });
            }
        }

        Ok(routes)
    }

    /// Look up a stop by ID.
    pub fn get_stop(&self, stop_id: &str) -> Option<&GtfsStop> {
        self.stops_by_id.get(stop_id).map(|&i| &self.stops[i])
    }

    /// Look up a route by ID. Supports both full GTFS IDs (e.g., "5512_123876")
    /// and real-time feed numeric IDs (e.g., "123876") via suffix matching.
    pub fn get_route(&self, route_id: &str) -> Option<&GtfsRoute> {
        // Try exact match first
        if let Some(&i) = self.routes_by_id.get(route_id) {
            return Some(&self.routes[i]);
        }
        // Try suffix match (real-time feed uses the numeric suffix)
        self.routes.iter().find(|r| {
            r.route_id.ends_with(&format!("_{}", route_id))
        })
    }

    /// Search stops by name (case-insensitive substring).
    pub fn search_stops(&self, query: &str) -> Vec<&GtfsStop> {
        let lower = query.to_lowercase();
        self.stops
            .iter()
            .filter(|s| s.stop_name.to_lowercase().contains(&lower))
            .collect()
    }

    /// Search routes by short name or long name (case-insensitive substring).
    pub fn search_routes(&self, query: &str) -> Vec<&GtfsRoute> {
        let lower = query.to_lowercase();
        self.routes
            .iter()
            .filter(|r| {
                r.route_short_name.to_lowercase().contains(&lower)
                    || r.route_long_name.to_lowercase().contains(&lower)
            })
            .collect()
    }
}
