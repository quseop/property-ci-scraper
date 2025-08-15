use crate::models::property::{Property, PropertyQuery, ExportFormat, ExportRequest};
use crate::repository::property_repo::PropertyRepo;
use csv::WriterBuilder;
use serde_json;
use anyhow::{Result, anyhow};
use log::{info, error};
use std::io::Write;
// Parquet support temporarily disabled due to compatibility issues
// use arrow::array::{StringArray, Int64Array, Float64Array, Int16Array, BooleanArray, ArrayRef, RecordBatch};
// use arrow::datatypes::{DataType, Field, Schema};
// use parquet::arrow::ArrowWriter;
// use parquet::file::properties::WriterProperties;
// use std::sync::Arc;
use actix_web::HttpResponse;

#[derive(Clone)]
pub struct DataExportService {
    repository: PropertyRepo,
}

impl DataExportService {
    pub fn new(repository: PropertyRepo) -> Self {
        Self { repository }
    }

    /// Export properties based on request parameters
    pub async fn export_data(&self, request: ExportRequest) -> Result<Vec<u8>> {
        info!("Starting data export with format: {:?}", request.format);
        
        // Get filtered properties
        let properties = self.get_filtered_properties(request.query).await?;
        
        if properties.is_empty() {
            return Err(anyhow!("No properties found matching the query"));
        }

        info!("Exporting {} properties", properties.len());

        match request.format {
            ExportFormat::Csv => self.export_to_csv(&properties).await,
            ExportFormat::Json => self.export_to_json(&properties).await,
            ExportFormat::Parquet => {
                error!("Parquet export temporarily disabled due to library compatibility issues");
                Err(anyhow!("Parquet export is currently not available. Please use CSV or JSON."))
            },
        }
    }

    /// Get properties with optional filtering
    async fn get_filtered_properties(&self, query: Option<PropertyQuery>) -> Result<Vec<Property>> {
        match query {
            Some(filter) => self.get_properties_with_filter(&filter).await,
            None => self.repository.find_all_properties().await
                .map_err(|e| anyhow!("Database error: {}", e)),
        }
    }

    /// Get properties with complex filtering (mock implementation)
    async fn get_properties_with_filter(&self, _filter: &PropertyQuery) -> Result<Vec<Property>> {
        // In a real implementation, this would use the filter parameters
        // to build a dynamic SQL query. For now, just return all properties.
        self.repository.find_all_properties().await
            .map_err(|e| anyhow!("Database error: {}", e))
    }

    /// Export to CSV format
    async fn export_to_csv(&self, properties: &[Property]) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(&mut buffer);

        // Write CSV headers
        writer.write_record(&[
            "id", "title", "price", "address", "province", "city", "suburb",
            "property_type", "bedrooms", "bathrooms", "garage_spaces",
            "land_size", "floor_size", "source_url", "latitude", "longitude"
        ])?;

        // Write property data
        for property in properties {
            let record = vec![
                property.id.clone(),
                property.title.clone(),
                property.price.map(|p| p.to_string()).unwrap_or_default(),
                property.address.clone(),
                property.province.clone(),
                property.city.clone(),
                property.suburb.as_ref().unwrap_or(&String::new()).clone(),
                property.property_type.clone(),
                property.bedrooms.map(|b| b.to_string()).unwrap_or_default(),
                property.bathrooms.map(|b| b.to_string()).unwrap_or_default(),
                property.garage_spaces.map(|g| g.to_string()).unwrap_or_default(),
                property.land_size.map(|l| l.to_string()).unwrap_or_default(),
                property.floor_size.map(|f| f.to_string()).unwrap_or_default(),
                property.source_url.clone(),
                property.latitude.map(|lat| lat.to_string()).unwrap_or_default(),
                property.longitude.map(|lon| lon.to_string()).unwrap_or_default(),
            ];
            writer.write_record(&record)?;
        }

        writer.flush()?;
        drop(writer);

        info!("Successfully exported {} properties to CSV", properties.len());
        Ok(buffer)
    }

    /// Export to JSON format
    async fn export_to_json(&self, properties: &[Property]) -> Result<Vec<u8>> {
        let json_data = serde_json::to_string_pretty(properties)?;
        info!("Successfully exported {} properties to JSON", properties.len());
        Ok(json_data.into_bytes())
    }


    /// Create an ML-ready dataset with feature engineering
    pub async fn export_ml_dataset(&self, query: Option<PropertyQuery>) -> Result<Vec<u8>> {
        let properties = self.get_filtered_properties(query).await?;
        
        if properties.is_empty() {
            return Err(anyhow!("No properties found for ML dataset export"));
        }

        // Feature engineering for ML
        let ml_records: Vec<MLPropertyRecord> = properties
            .into_iter()
            .filter_map(|p| self.create_ml_record(p))
            .collect();

        info!("Created {} ML records", ml_records.len());

        // Export as CSV (most common for ML)
        let mut buffer = Vec::new();
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(&mut buffer);

        // ML-specific headers
        writer.write_record(&[
            "id", "price", "price_per_sqm_floor", "price_per_sqm_land",
            "bedrooms", "bathrooms", "garage_spaces", "land_size", "floor_size",
            "property_type_encoded", "province_encoded", "city_encoded",
            "has_suburb", "latitude", "longitude", "price_category"
        ])?;

        for record in ml_records {
            let csv_record = vec![
                record.id,
                record.price.to_string(),
                record.price_per_sqm_floor.map(|p| p.to_string()).unwrap_or_default(),
                record.price_per_sqm_land.map(|p| p.to_string()).unwrap_or_default(),
                record.bedrooms.to_string(),
                record.bathrooms.to_string(),
                record.garage_spaces.to_string(),
                record.land_size.map(|l| l.to_string()).unwrap_or_default(),
                record.floor_size.map(|f| f.to_string()).unwrap_or_default(),
                record.property_type_encoded.to_string(),
                record.province_encoded.to_string(),
                record.city_encoded.to_string(),
                if record.has_suburb { "1" } else { "0" }.to_string(),
                record.latitude.map(|lat| lat.to_string()).unwrap_or_default(),
                record.longitude.map(|lon| lon.to_string()).unwrap_or_default(),
                record.price_category,
            ];
            writer.write_record(&csv_record)?;
        }

        writer.flush()?;
        drop(writer);

        Ok(buffer)
    }

    /// Create ML-ready record with engineered features
    fn create_ml_record(&self, property: Property) -> Option<MLPropertyRecord> {
        // Skip properties without price (can't train on them)
        let price = property.price?;

        // Calculate derived features
        let price_per_sqm_floor = property.floor_size
            .map(|size| if size > 0.0 { price as f64 / size } else { 0.0 });

        let price_per_sqm_land = property.land_size
            .map(|size| if size > 0.0 { price as f64 / size } else { 0.0 });

        // Encode categorical variables (simple hash-based encoding for demo)
        let property_type_encoded = self.hash_encode(&property.property_type);
        let province_encoded = self.hash_encode(&property.province);
        let city_encoded = self.hash_encode(&property.city);

        // Price categorization
        let price_category = self.categorize_price(price);

        Some(MLPropertyRecord {
            id: property.id,
            price,
            price_per_sqm_floor,
            price_per_sqm_land,
            bedrooms: property.bedrooms.unwrap_or(0),
            bathrooms: property.bathrooms.unwrap_or(0),
            garage_spaces: property.garage_spaces.unwrap_or(0),
            land_size: property.land_size,
            floor_size: property.floor_size,
            property_type_encoded,
            province_encoded,
            city_encoded,
            has_suburb: property.suburb.is_some(),
            latitude: property.latitude,
            longitude: property.longitude,
            price_category,
        })
    }

    /// Simple hash-based encoding for categorical variables
    fn hash_encode(&self, value: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    /// Categorize price into buckets
    fn categorize_price(&self, price: i64) -> String {
        match price {
            0..=500_000 => "low".to_string(),
            500_001..=1_000_000 => "medium".to_string(),
            1_000_001..=2_000_000 => "high".to_string(),
            _ => "premium".to_string(),
        }
    }

    /// Get export statistics
    pub async fn get_export_stats(&self) -> Result<ExportStats> {
        let all_properties = self.repository.find_all_properties().await
            .map_err(|e| anyhow!("Database error: {}", e))?;

        let total_properties = all_properties.len();
        let properties_with_price = all_properties.iter()
            .filter(|p| p.price.is_some())
            .count();
        let properties_with_coordinates = all_properties.iter()
            .filter(|p| p.latitude.is_some() && p.longitude.is_some())
            .count();

        let unique_cities = all_properties.iter()
            .map(|p| p.city.as_str())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let unique_provinces = all_properties.iter()
            .map(|p| p.province.as_str())
            .collect::<std::collections::HashSet<_>>()
            .len();

        Ok(ExportStats {
            total_properties,
            properties_with_price,
            properties_with_coordinates,
            unique_cities,
            unique_provinces,
        })
    }
}

/// ML-ready property record with engineered features
#[derive(Debug)]
struct MLPropertyRecord {
    id: String,
    price: i64,
    price_per_sqm_floor: Option<f64>,
    price_per_sqm_land: Option<f64>,
    bedrooms: i16,
    bathrooms: i16,
    garage_spaces: i16,
    land_size: Option<f64>,
    floor_size: Option<f64>,
    property_type_encoded: u64,
    province_encoded: u64,
    city_encoded: u64,
    has_suburb: bool,
    latitude: Option<f64>,
    longitude: Option<f64>,
    price_category: String,
}

/// Export statistics
#[derive(serde::Serialize, Debug)]
pub struct ExportStats {
    pub total_properties: usize,
    pub properties_with_price: usize,
    pub properties_with_coordinates: usize,
    pub unique_cities: usize,
    pub unique_provinces: usize,
}
