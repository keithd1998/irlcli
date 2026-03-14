use anyhow::Result;
use clap::Subcommand;

use irl_core::output::OutputConfig;

use crate::api::GeoApi;
use crate::models::*;

#[derive(Debug, Subcommand)]
pub enum GeoCommands {
    /// Fetch boundary data (counties, provinces, electoral divisions)
    ///
    /// Queries ArcGIS REST services for Irish boundary data.
    /// Note: FeatureServer URLs may need to be updated as service IDs are discovered.
    ///
    /// Types: county, province, electoral
    ///
    /// Examples:
    ///   irl geo boundaries --type county
    ///   irl geo boundaries --type province
    Boundaries {
        /// Boundary type (county, province, electoral)
        #[arg(long, name = "type")]
        boundary_type: String,
    },

    /// Find what boundary contains a geographic point
    ///
    /// Given a latitude and longitude, finds which boundaries (county, etc.)
    /// contain that point.
    ///
    /// Examples:
    ///   irl geo search --lat 53.35 --lon -6.26
    Search {
        /// Latitude (WGS84)
        #[arg(long)]
        lat: f64,
        /// Longitude (WGS84)
        #[arg(long)]
        lon: f64,
    },

    /// List available spatial datasets
    ///
    /// Shows the known ArcGIS FeatureServer datasets available for query.
    /// Note: Additional datasets may be available as service IDs are discovered.
    ///
    /// Examples:
    ///   irl geo datasets
    Datasets,

    /// Download a spatial dataset
    ///
    /// Fetches data from an ArcGIS FeatureServer and outputs it
    /// in the requested format.
    ///
    /// Examples:
    ///   irl geo fetch Counties_National_Statutory_Boundary_2019
    ///   irl geo fetch Counties_National_Statutory_Boundary_2019 --format geojson
    Fetch {
        /// Dataset/service name or ID
        dataset_id: String,
        /// Output format (json, geojson)
        #[arg(long, default_value = "geojson")]
        format: String,
    },
}

pub async fn handle_command(
    cmd: &GeoCommands,
    output: &OutputConfig,
    verbose: bool,
    quiet: bool,
    no_cache: bool,
) -> Result<()> {
    let api = GeoApi::new(verbose, quiet, no_cache)?;

    match cmd {
        GeoCommands::Boundaries { boundary_type } => {
            output.print_header("GeoHive Boundary Data");
            output.print_info(
                "Note: ArcGIS FeatureServer URLs are being configured. \
                 Results depend on correct service IDs being available.",
            );
            match api.query_boundaries(boundary_type).await {
                Ok(response) => {
                    // Detect fields before consuming features
                    let name_field = detect_name_field(&response);
                    let id_field = detect_id_field(&response);
                    let features = response.features.unwrap_or_default();
                    let mut rows: Vec<BoundaryRow> = features
                        .iter()
                        .map(|f| {
                            let mut row =
                                BoundaryRow::from_feature(f, &name_field, &id_field);
                            row.boundary_type = boundary_type.clone();
                            row
                        })
                        .collect();
                    rows.sort_by(|a, b| a.name.cmp(&b.name));
                    output.print_info(&format!("{} boundaries found", rows.len()));
                    output.render(&rows)?;
                }
                Err(e) => {
                    output.print_info(&format!(
                        "GeoHive data request failed: {}\n\n\
                         FeatureServer URLs may need to be configured. \
                         Visit https://data.gov.ie for Irish spatial data.",
                        e
                    ));
                }
            }
        }

        GeoCommands::Search { lat, lon } => {
            output.print_header("GeoHive Point Search");
            output.print_info(
                "Note: ArcGIS FeatureServer URLs are being configured. \
                 Results depend on correct service IDs being available.",
            );
            match api.query_point(*lat, *lon).await {
                Ok(response) => {
                    let name_field = detect_name_field(&response);
                    let features = response.features.unwrap_or_default();
                    let rows: Vec<SearchResultRow> = features
                        .iter()
                        .map(|f| {
                            let attrs = f.attributes.as_ref();
                            let name = attrs
                                .and_then(|a| a.get(&name_field))
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            SearchResultRow {
                                name,
                                boundary_type: "County".to_string(),
                                contains: format!("{}, {}", lat, lon),
                            }
                        })
                        .collect();
                    output.print_info(&format!(
                        "{} boundaries contain point ({}, {})",
                        rows.len(),
                        lat,
                        lon
                    ));
                    output.render(&rows)?;
                }
                Err(e) => {
                    output.print_info(&format!(
                        "GeoHive point query failed: {}\n\n\
                         FeatureServer URLs may need to be configured. \
                         Visit https://data.gov.ie for Irish spatial data.",
                        e
                    ));
                }
            }
        }

        GeoCommands::Datasets => {
            output.print_header("Available Spatial Datasets");
            output.print_info(
                "Note: FeatureServer URLs are being configured. \
                 The following are known dataset patterns:",
            );
            // List known dataset names that follow the ArcGIS naming pattern
            let known_datasets = vec![
                DatasetRow {
                    name: "Counties_National_Statutory_Boundary_2019".to_string(),
                    service_type: "FeatureServer".to_string(),
                    description: "County boundaries of Ireland".to_string(),
                },
                DatasetRow {
                    name: "Provinces_National_Statutory_Boundary_2019".to_string(),
                    service_type: "FeatureServer".to_string(),
                    description: "Province boundaries of Ireland".to_string(),
                },
                DatasetRow {
                    name: "Electoral_Divisions_National_Statutory_Boundary_2019".to_string(),
                    service_type: "FeatureServer".to_string(),
                    description: "Electoral division boundaries".to_string(),
                },
            ];
            output.print_info(&format!("{} known datasets", known_datasets.len()));
            output.render(&known_datasets)?;
        }

        GeoCommands::Fetch { dataset_id, format } => {
            output.print_header("Fetch Spatial Dataset");
            output.print_info(
                "Note: ArcGIS FeatureServer URLs are being configured. \
                 Results depend on correct service IDs being available.",
            );
            match api.fetch_dataset(dataset_id, format).await {
                Ok(data) => {
                    // Output raw data directly for piping
                    println!("{}", data);
                }
                Err(e) => {
                    output.print_info(&format!(
                        "Failed to fetch dataset '{}': {}\n\n\
                         FeatureServer URLs may need to be configured. \
                         Visit https://data.gov.ie for Irish spatial data.",
                        dataset_id, e
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Detect the most likely name field from ArcGIS field definitions
fn detect_name_field(response: &QueryResponse) -> String {
    if let Some(fields) = &response.fields {
        let name_candidates = [
            "COUNTY_NAME",
            "COUNTYNAME",
            "CONTAE",
            "NAME",
            "ENGLISH",
            "ED_ENGLISH",
            "PROVINCE",
        ];
        for candidate in &name_candidates {
            if fields
                .iter()
                .any(|f| f.name.as_deref() == Some(candidate))
            {
                return candidate.to_string();
            }
        }
    }
    "NAME".to_string()
}

/// Detect the most likely ID field from ArcGIS field definitions
fn detect_id_field(response: &QueryResponse) -> String {
    if let Some(fields) = &response.fields {
        let id_candidates = [
            "COUNTY_ID",
            "OBJECTID",
            "FID",
            "CC_ID",
            "NUTS3",
        ];
        for candidate in &id_candidates {
            if fields
                .iter()
                .any(|f| f.name.as_deref() == Some(candidate))
            {
                return candidate.to_string();
            }
        }
    }
    "OBJECTID".to_string()
}
