use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Deserialize)]
pub struct PropertyNew {
    pub title: String,
    pub price: Option<i64>,
    pub address: String,
    pub province: String,
    pub city: String,
    pub suburb: Option<String>,
    pub property_type: String, // residential, commercial, industrial, etc.
    pub bedrooms: Option<i16>,
    pub bathrooms: Option<i16>,
    pub garage_spaces: Option<i16>,
    pub land_size: Option<f64>, // in square meters
    pub floor_size: Option<f64>, // in square meters
    pub source_url: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Property {
    pub id: String,
    pub title: String,
    pub price: Option<i64>,
    pub address: String,
    pub province: String,
    pub city: String,
    pub suburb: Option<String>,
    pub property_type: String, // residential, commercial, industrial, etc.
    pub bedrooms: Option<i16>,
    pub bathrooms: Option<i16>,
    pub garage_spaces: Option<i16>,
    pub land_size: Option<f64>, // in square meters
    pub floor_size: Option<f64>, // in square meters
    pub source_url: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Property {
    pub fn new(
        title: String,
        price: Option<i64>,
        address: String,
        province: String,
        city: String,
        suburb: Option<String>,
        property_type: String,
        bedrooms: Option<i16>,
        bathrooms: Option<i16>,
        garage_spaces: Option<i16>,
        land_size: Option<f64>,
        floor_size: Option<f64>,
        source_url: String,
        latitude: Option<f64>,
        longitude: Option<f64>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            price,
            address,
            province,
            city,
            suburb,
            property_type,
            bedrooms,
            bathrooms,
            garage_spaces,
            land_size,
            floor_size,
            source_url,
            latitude,
            longitude,
        }
    }
    
    pub fn from(property: &PropertyNew) -> Self {
        Self::new(
            property.title.clone(),
            property.price,
            property.address.clone(),
            property.province.clone(),
            property.city.clone(),
            property.suburb.clone(),
            property.property_type.clone(),
            property.bedrooms,
            property.bathrooms,
            property.garage_spaces,
            property.land_size,
            property.floor_size,
            property.source_url.clone(),
            property.latitude,
            property.longitude,       
        )
    }

    pub fn new_with_id(id: String, property: &PropertyNew) -> Self {
        Self{
            id,
            title: property.title.clone(),
            price: property.price,
            address: property.address.clone(),
            province: property.province.clone(),
            city: property.city.clone(),
            suburb: property.suburb.clone(),
            property_type: property.property_type.clone(),
            bedrooms: property.bedrooms,
            bathrooms: property.bathrooms,
            garage_spaces: property.garage_spaces,
            land_size: property.land_size,
            floor_size: property.floor_size,
            source_url: property.source_url.clone(),
            latitude: property.latitude,
            longitude: property.longitude,
        }
    }
}

// Query parameters for filtering properties
#[derive(Deserialize, Debug)]
pub struct PropertyQuery {
    pub city: Option<String>,
    pub province: Option<String>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub property_type: Option<String>,
    pub min_bedrooms: Option<i16>,
    pub max_bedrooms: Option<i16>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Export format options for ML datasets
#[derive(Deserialize, Debug)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub query: Option<PropertyQuery>,
    pub include_metadata: Option<bool>,
}

#[derive(Deserialize, Clone, Debug)]
pub enum ExportFormat {
    #[serde(rename = "csv")]
    Csv,
    #[serde(rename = "parquet")]
    Parquet,
    #[serde(rename = "json")]
    Json,
}

// Scraping job creation request (without server-generated fields)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScrapingJobRequest {
    pub name: String,
    pub target_url: String,
    pub selectors: PropertySelectors,
    pub schedule: String, // Cron expression
    pub active: bool,
}

// Scraping job configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScrapingJob {
    pub id: String,
    pub name: String,
    pub target_url: String,
    pub selectors: PropertySelectors,
    pub schedule: String, // Cron expression
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
}

impl ScrapingJob {
    pub fn from_request(request: ScrapingJobRequest) -> Self {
        Self {
            id: String::new(), // Will be generated by scheduler
            name: request.name,
            target_url: request.target_url,
            selectors: request.selectors,
            schedule: request.schedule,
            active: request.active,
            created_at: Utc::now(),
            last_run: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PropertySelectors {
    pub title: String,
    pub price: Option<String>,
    pub address: String,
    pub property_type: Option<String>,
    pub bedrooms: Option<String>,
    pub bathrooms: Option<String>,
    pub land_size: Option<String>,
    pub floor_size: Option<String>,
}

// Scraping result status
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScrapingResult {
    pub job_id: String,
    pub status: ScrapingStatus,
    pub properties_scraped: i32,
    pub errors: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ScrapingStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

// Property statistics for analytics
#[derive(Serialize, Deserialize, Debug)]
pub struct PropertyStats {
    pub total_properties: i64,
    pub properties_with_price: i64,
    pub average_price: Option<i64>,
    pub unique_cities: i64,
}
