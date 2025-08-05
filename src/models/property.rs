use chrono::Utc;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Deserialize)]
pub struct PropertyNew {
    pub title: String,
    pub price: Option<i64>,
    pub address: String,
    pub province: String,
    pub city: String,
    pub suburb: Option<String>,
    pub property_type: String, // residential, commercial, industrial, etc.
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub garage_spaces: Option<i32>,
    pub land_size: Option<f64>, // in square meters
    pub floor_size: Option<f64>, // in square meters
    pub scraped_at: chrono::DateTime<Utc>,
    pub source_url: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Property {
    pub id: String,
    pub title: String,
    pub price: Option<i64>,
    pub address: String,
    pub province: String,
    pub city: String,
    pub suburb: Option<String>,
    pub property_type: String, // residential, commercial, industrial, etc.
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub garage_spaces: Option<i32>,
    pub land_size: Option<f64>, // in square meters
    pub floor_size: Option<f64>, // in square meters
    pub scraped_at: chrono::DateTime<Utc>,
    pub source_url: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Property {
    pub fn new(
        id: String,
        title: String,
        price: Option<i64>,
        address: String,
        province: String,
        city: String,
        suburb: Option<String>,
        property_type: String,
        bedrooms: Option<i32>,
        bathrooms: Option<i32>,
        garage_spaces: Option<i32>,
        land_size: Option<f64>,
        floor_size: Option<f64>,
        source_url: String,
        latitude: Option<f64>,
        longitude: Option<f64>,
    ) -> Self {
        Self {
            id,
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
            scraped_at: Utc::now(),
            source_url,
            latitude,
            longitude,
        }
    }
}