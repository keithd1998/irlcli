use crate::db::PropertyDb;
use crate::models::{PropertySale, PropertyStats};

pub struct PropertyData;

impl PropertyData {
    pub fn is_loaded() -> bool {
        let db = match PropertyDb::open() {
            Ok(db) => db,
            Err(_) => return false,
        };
        db.record_count().unwrap_or(0) > 0
    }

    pub fn record_count() -> Result<u64, anyhow::Error> {
        let db = PropertyDb::open()?;
        db.record_count()
    }

    pub fn import_csv(path: &str) -> Result<u64, anyhow::Error> {
        let db = PropertyDb::open()?;
        db.clear()?;
        db.import_csv(path)
    }

    pub fn search(
        county: Option<&str>,
        year: Option<&str>,
        min_price: Option<f64>,
        max_price: Option<f64>,
        address: Option<&str>,
    ) -> Result<Vec<PropertySale>, anyhow::Error> {
        let db = PropertyDb::open()?;
        db.search(county, year, min_price, max_price, address, 100)
    }

    pub fn stats(county: Option<&str>, year: Option<&str>) -> Result<PropertyStats, anyhow::Error> {
        let db = PropertyDb::open()?;
        let prices = db.get_prices(county, year)?;
        Ok(PropertyStats::calculate(&prices))
    }
}
