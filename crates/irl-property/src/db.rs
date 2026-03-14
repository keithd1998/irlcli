use std::path::PathBuf;

use rusqlite::{params, Connection};

use irl_core::config::Config;

use crate::models::{parse_price, parse_yes_no, PropertySale};

const DB_NAME: &str = "property.db";

pub struct PropertyDb {
    conn: Connection,
}

impl PropertyDb {
    pub fn open() -> Result<Self, anyhow::Error> {
        let data_dir = Config::data_dir();
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }
        let db_path = data_dir.join(DB_NAME);
        let conn = Connection::open(&db_path)?;
        let db = Self { conn };
        db.create_tables()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing)
    #[cfg(test)]
    pub fn open_memory() -> Result<Self, anyhow::Error> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<(), anyhow::Error> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS property_sales (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                address TEXT NOT NULL,
                county TEXT NOT NULL,
                eircode TEXT NOT NULL DEFAULT '',
                price REAL NOT NULL,
                not_full_market_price INTEGER NOT NULL DEFAULT 0,
                vat_exclusive INTEGER NOT NULL DEFAULT 0,
                description TEXT NOT NULL DEFAULT '',
                property_size TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_county ON property_sales(county);
            CREATE INDEX IF NOT EXISTS idx_date ON property_sales(date);
            CREATE INDEX IF NOT EXISTS idx_price ON property_sales(price);",
        )?;
        Ok(())
    }

    pub fn data_path() -> PathBuf {
        Config::data_dir().join(DB_NAME)
    }

    pub fn record_count(&self) -> Result<u64, anyhow::Error> {
        let count: u64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM property_sales", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn clear(&self) -> Result<(), anyhow::Error> {
        self.conn.execute("DELETE FROM property_sales", [])?;
        Ok(())
    }

    pub fn import_csv(&self, path: &str) -> Result<u64, anyhow::Error> {
        // Read the file as bytes and handle encoding
        let bytes = std::fs::read(path)?;

        // Try to detect encoding - PSRA CSVs may be Windows-1252 or UTF-8
        let content = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            // UTF-8 BOM
            String::from_utf8_lossy(&bytes[3..]).to_string()
        } else {
            match String::from_utf8(bytes.clone()) {
                Ok(s) => s,
                Err(_) => {
                    // Try Windows-1252 encoding
                    let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
                    decoded.to_string()
                }
            }
        };

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(content.as_bytes());

        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0u64;

        for result in reader.records() {
            let record = match result {
                Ok(r) => r,
                Err(_) => continue,
            };

            // PSRA CSV columns:
            // 0: Date of Sale (dd/mm/yyyy)
            // 1: Address
            // 2: County
            // 3: Eircode
            // 4: Price (€)
            // 5: Not Full Market Price
            // 6: VAT Exclusive
            // 7: Description of Property
            // 8: Property Size Description
            if record.len() < 5 {
                continue;
            }

            let date_raw = record.get(0).unwrap_or("").trim();
            let date = convert_date(date_raw);
            let address = record.get(1).unwrap_or("").trim().to_string();
            let county = record.get(2).unwrap_or("").trim().to_string();
            let eircode = record.get(3).unwrap_or("").trim().to_string();
            let price = parse_price(record.get(4).unwrap_or("0"));
            let not_full_market_price = parse_yes_no(record.get(5).unwrap_or("No"));
            let vat_exclusive = parse_yes_no(record.get(6).unwrap_or("No"));
            let description = record.get(7).unwrap_or("").trim().to_string();
            let property_size = record.get(8).unwrap_or("").trim().to_string();

            if price <= 0.0 || address.is_empty() {
                continue;
            }

            tx.execute(
                "INSERT INTO property_sales \
                 (date, address, county, eircode, price, not_full_market_price, \
                  vat_exclusive, description, property_size) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    date,
                    address,
                    county,
                    eircode,
                    price,
                    not_full_market_price as i32,
                    vat_exclusive as i32,
                    description,
                    property_size,
                ],
            )?;
            count += 1;
        }

        tx.commit()?;
        Ok(count)
    }

    pub fn search(
        &self,
        county: Option<&str>,
        year: Option<&str>,
        min_price: Option<f64>,
        max_price: Option<f64>,
        address: Option<&str>,
        limit: u32,
    ) -> Result<Vec<PropertySale>, anyhow::Error> {
        let mut sql = "SELECT date, address, county, eircode, price, \
                       not_full_market_price, vat_exclusive, description, property_size \
                       FROM property_sales WHERE 1=1"
            .to_string();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(county) = county {
            sql.push_str(" AND LOWER(county) LIKE ?");
            param_values.push(Box::new(format!("%{}%", county.to_lowercase())));
        }
        if let Some(year) = year {
            sql.push_str(" AND date LIKE ?");
            param_values.push(Box::new(format!("{}%", year)));
        }
        if let Some(min) = min_price {
            sql.push_str(" AND price >= ?");
            param_values.push(Box::new(min));
        }
        if let Some(max) = max_price {
            sql.push_str(" AND price <= ?");
            param_values.push(Box::new(max));
        }
        if let Some(addr) = address {
            sql.push_str(" AND LOWER(address) LIKE ?");
            param_values.push(Box::new(format!("%{}%", addr.to_lowercase())));
        }

        sql.push_str(" ORDER BY date DESC");
        sql.push_str(&format!(" LIMIT {}", limit));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(PropertySale {
                date: row.get(0)?,
                address: row.get(1)?,
                county: row.get(2)?,
                eircode: row.get(3)?,
                price: row.get(4)?,
                not_full_market_price: row.get::<_, i32>(5)? != 0,
                vat_exclusive: row.get::<_, i32>(6)? != 0,
                description: row.get(7)?,
                property_size: row.get(8)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_prices(
        &self,
        county: Option<&str>,
        year: Option<&str>,
    ) -> Result<Vec<f64>, anyhow::Error> {
        let mut sql = "SELECT price FROM property_sales WHERE 1=1".to_string();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(county) = county {
            sql.push_str(" AND LOWER(county) LIKE ?");
            param_values.push(Box::new(format!("%{}%", county.to_lowercase())));
        }
        if let Some(year) = year {
            sql.push_str(" AND date LIKE ?");
            param_values.push(Box::new(format!("{}%", year)));
        }

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_ref.as_slice(), |row| row.get::<_, f64>(0))?;

        let mut prices = Vec::new();
        for row in rows {
            prices.push(row?);
        }
        Ok(prices)
    }
}

/// Convert dd/mm/yyyy to yyyy-mm-dd
fn convert_date(s: &str) -> String {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 3 {
        format!("{}-{}-{}", parts[2], parts[1], parts[0])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_convert_date() {
        assert_eq!(convert_date("15/01/2024"), "2024-01-15");
        assert_eq!(convert_date("2024-01-15"), "2024-01-15");
        assert_eq!(convert_date("invalid"), "invalid");
    }

    #[test]
    fn test_create_db() {
        let db = PropertyDb::open_memory().unwrap();
        let count = db.record_count().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_import_and_search() {
        let db = PropertyDb::open_memory().unwrap();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "Date of Sale (dd/mm/yyyy),Address,County,Eircode,Price (\u{20ac}),Not Full Market Price,VAT Exclusive,Description of Property,Property Size Description"
        )
        .unwrap();
        writeln!(
            file,
            "15/01/2024,1 Main Street Dublin,Dublin,D01 AB12,\"\u{20ac}350,000.00\",No,No,Second-Hand Dwelling house /Apartment,greater than or equal to 38 sq metres and less than 125 sq metres"
        )
        .unwrap();
        writeln!(
            file,
            "20/03/2024,5 High Street Cork,Cork,T12 XY34,\"\u{20ac}250,000.00\",No,No,New Dwelling house /Apartment,greater than or equal to 125 sq metres"
        )
        .unwrap();
        writeln!(
            file,
            "10/06/2023,10 Park Lane Galway,Galway,H91 ZZ99,\"\u{20ac}180,000.00\",Yes,No,Second-Hand Dwelling house /Apartment,"
        )
        .unwrap();

        let count = db.import_csv(file.path().to_str().unwrap()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(db.record_count().unwrap(), 3);

        // Search by county
        let results = db
            .search(Some("Dublin"), None, None, None, None, 50)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].county, "Dublin");
        assert!((results[0].price - 350000.0).abs() < 0.01);

        // Search by year
        let results = db.search(None, Some("2024"), None, None, None, 50).unwrap();
        assert_eq!(results.len(), 2);

        // Search by price range
        let results = db
            .search(None, None, Some(200000.0), Some(300000.0), None, 50)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].county, "Cork");

        // Search by address
        let results = db
            .search(None, None, None, None, Some("main street"), 50)
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_get_prices() {
        let db = PropertyDb::open_memory().unwrap();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "Date,Address,County,Eircode,Price,NotFull,VAT,Desc,Size"
        )
        .unwrap();
        writeln!(file, "15/01/2024,1 Main St,Dublin,,350000,No,No,Dwelling,").unwrap();
        writeln!(file, "20/03/2024,5 High St,Dublin,,250000,No,No,Dwelling,").unwrap();
        writeln!(file, "10/06/2024,10 Park Ln,Cork,,180000,No,No,Dwelling,").unwrap();

        db.import_csv(file.path().to_str().unwrap()).unwrap();

        let prices = db.get_prices(Some("Dublin"), None).unwrap();
        assert_eq!(prices.len(), 2);

        let prices = db.get_prices(None, Some("2024")).unwrap();
        assert_eq!(prices.len(), 3);
    }

    #[test]
    fn test_clear() {
        let db = PropertyDb::open_memory().unwrap();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Date,Address,County,Eircode,Price,NF,VAT,Desc,Size").unwrap();
        writeln!(file, "15/01/2024,1 Main St,Dublin,,350000,No,No,,").unwrap();

        db.import_csv(file.path().to_str().unwrap()).unwrap();
        assert_eq!(db.record_count().unwrap(), 1);

        db.clear().unwrap();
        assert_eq!(db.record_count().unwrap(), 0);
    }

    #[test]
    fn test_import_skips_bad_rows() {
        let db = PropertyDb::open_memory().unwrap();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Date,Address,County,Eircode,Price,NF,VAT,Desc,Size").unwrap();
        writeln!(file, "15/01/2024,1 Main St,Dublin,,350000,No,No,,").unwrap();
        writeln!(file, "15/01/2024,,Dublin,,350000,No,No,,").unwrap(); // empty address
        writeln!(file, "15/01/2024,2 Main St,Dublin,,0,No,No,,").unwrap(); // zero price
        writeln!(file, "short").unwrap(); // too few columns

        let count = db.import_csv(file.path().to_str().unwrap()).unwrap();
        assert_eq!(count, 1);
    }
}
