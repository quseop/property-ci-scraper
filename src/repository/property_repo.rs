use actix_web::web::Json;
use sqlx::PgPool;
use crate::models::property::Property;

#[derive(Clone)]
pub struct PropertyRepo {
    pub pool: PgPool
}

impl PropertyRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get All Properties From The Database
    pub async fn find_all_properties(&self) -> Result<Vec<Property>, sqlx::Error> {
        todo!()
    }

    /// Find Property By ID
    pub async fn find_property_by_id(&self, id: &str) -> Result<Property, sqlx::Error> {
        todo!()
    }

    /// Create Property
    pub async fn create_property(&self, property: Json<Property>) -> Result<Property, sqlx::Error> {
        todo!()
    }

    /// Update Property By ID
    pub async fn update_property_by_id(&self, id: &str, property: Json<Property>) -> Result<Property, sqlx::Error> {
        todo!()
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