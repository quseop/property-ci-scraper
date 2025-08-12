use actix_web::web::Json;
use sqlx::PgPool;
use crate::models::property::{Property, PropertyNew};

#[derive(Clone)]
pub struct PropertyRepo {
    pub pool: PgPool
}

impl PropertyRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get All Properties From The Properties Database
    pub async fn find_all_properties(&self) -> Result<Vec<Property>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM properties")
            .fetch_all(&self.pool)
            .await
    }

    /// Find Property By ID
    pub async fn find_property_by_id(&self, id: String) -> Result<Property, sqlx::Error> {
        sqlx::query_as("SELECT * FROM properties WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    /// Create Property
    pub async fn create_property(&self, property: Json<PropertyNew>) -> Result<Property, sqlx::Error> {

        let property = Property::from(&property);

        sqlx::query_as("INSERT INTO properties(
            id, title, price, address, province, city, suburb,
            property_type, bedrooms, bathrooms, garage_spaces,
            land_size, floor_size, source_url,
            latitude, longitude)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING * ")
            .bind(&property.id)
            .bind(&property.title)
            .bind(&property.price)
            .bind(&property.address)
            .bind(&property.province)
            .bind(&property.city)
            .bind(&property.suburb)
            .bind(&property.property_type)
            .bind(&property.bedrooms)
            .bind(&property.bathrooms)
            .bind(&property.garage_spaces)
            .bind(&property.land_size)
            .bind(&property.floor_size)
            .bind(&property.source_url)
            .bind(&property.latitude)
            .bind(&property.longitude)
            .fetch_one(&self.pool)
            .await
    }

    /// Update Property By ID
    pub async fn update_property_by_id(&self, id: String, property: Json<PropertyNew>) -> Result<Property, sqlx::Error> {
        let property = Property::new_with_id(id, &property);
   
        sqlx::query_as("
            UPDATE properties
            SET title = $1, price = $2, address = $3, province = $4, city = $5, suburb = $6,
            property_type = $7, bedrooms = $8, bathrooms = $9, garage_spaces = $10,
            land_size = $11, floor_size = $12, source_url = $13,
            latitude = $14, longitude = $15
            WHERE id = $16
            ")
            .bind(&property.title)
            .bind(&property.price)
            .bind(&property.address)
            .bind(&property.province)
            .bind(&property.city)
            .bind(&property.suburb)
            .bind(&property.property_type)
            .bind(&property.bedrooms)
            .bind(&property.bathrooms)
            .bind(&property.garage_spaces)
            .bind(&property.land_size)
            .bind(&property.floor_size)
            .bind(&property.source_url)
            .bind(&property.latitude)
            .bind(&property.longitude)
            .bind(&property.id)
            .fetch_one(&self.pool)
            .await
    }

    /// Delete Property By ID
    pub async fn delete_property_by_id(&self, id: &str) -> Result<Property, sqlx::Error> {
        todo!()
    }

    /// Flush DB
    pub async fn flush_db(&self) -> Result<(), sqlx::Error> {
        todo!()
    }
}