use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;

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
