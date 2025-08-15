use actix_web::web::Json;
use sqlx::PgPool;
use crate::models::property::{Property, PropertyNew, PropertyStats};

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
        sqlx::query_as("DELETE FROM properties WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    /// Find properties by city
    pub async fn find_properties_by_city(&self, city: &str) -> Result<Vec<Property>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM properties WHERE city = $1 ORDER BY scraped_at DESC")
            .bind(city)
            .fetch_all(&self.pool)
            .await
    }

    /// Find properties by price range
    pub async fn find_properties_by_price_range(
        &self, 
        min_price: Option<i64>, 
        max_price: Option<i64>
    ) -> Result<Vec<Property>, sqlx::Error> {
        let mut query = "SELECT * FROM properties WHERE 1=1".to_string();
        let mut params = Vec::new();
        let mut param_count = 0;

        if let Some(min) = min_price {
            param_count += 1;
            query.push_str(&format!(" AND price >= ${}", param_count));
            params.push(min);
        }

        if let Some(max) = max_price {
            param_count += 1;
            query.push_str(&format!(" AND price <= ${}", param_count));
            params.push(max);
        }

        query.push_str(" ORDER BY scraped_at DESC");

        let mut sql_query = sqlx::query_as::<Property>(&query);
        for param in params {
            sql_query = sql_query.bind(param);
        }

        sql_query.fetch_all(&self.pool).await
    }

    /// Find properties by property type
    pub async fn find_properties_by_type(&self, property_type: &str) -> Result<Vec<Property>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM properties WHERE property_type = $1 ORDER BY scraped_at DESC")
            .bind(property_type)
            .fetch_all(&self.pool)
            .await
    }

    /// Get property statistics
    pub async fn get_property_stats(&self) -> Result<PropertyStats, sqlx::Error> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM properties")
            .fetch_one(&self.pool)
            .await?;

        let with_price: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM properties WHERE price IS NOT NULL")
            .fetch_one(&self.pool)
            .await?;

        let avg_price: (Option<f64>,) = sqlx::query_as("SELECT AVG(price::float) FROM properties WHERE price IS NOT NULL")
            .fetch_one(&self.pool)
            .await?;

        let unique_cities: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT city) FROM properties")
            .fetch_one(&self.pool)
            .await?;

        Ok(PropertyStats {
            total_properties: total.0,
            properties_with_price: with_price.0,
            average_price: avg_price.0.map(|p| p as i64),
            unique_cities: unique_cities.0,
        })
    }

    /// Get recent properties (last N days)
    pub async fn find_recent_properties(&self, days: i32) -> Result<Vec<Property>, sqlx::Error> {
        sqlx::query_as(
            "SELECT * FROM properties 
             WHERE scraped_at >= NOW() - INTERVAL '%d days' 
             ORDER BY scraped_at DESC"
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await
    }

    /// Check if property exists by URL
    pub async fn property_exists(&self, source_url: &str) -> Result<bool, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM properties WHERE source_url = $1")
            .bind(source_url)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 > 0)
    }

    /// Bulk insert properties (for efficient scraping)
    pub async fn bulk_create_properties(&self, properties: Vec<PropertyNew>) -> Result<i64, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let mut inserted = 0i64;

        for property_data in properties {
            let property = Property::from(&property_data);
            
            let result = sqlx::query(
                "INSERT INTO properties(
                    id, title, price, address, province, city, suburb,
                    property_type, bedrooms, bathrooms, garage_spaces,
                    land_size, floor_size, source_url,
                    latitude, longitude)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                ON CONFLICT (source_url) DO NOTHING"
            )
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
            .execute(&mut *tx)
            .await;

            if let Ok(result) = result {
                inserted += result.rows_affected() as i64;
            }
        }

        tx.commit().await?;
        Ok(inserted)
    }

    /// Flush DB (delete all properties - use with caution!)
    pub async fn flush_db(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM properties")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}