use crate::models::property::{Property, PropertyNew, PropertySelectors, ScrapingJob, ScrapingResult, ScrapingStatus};
use crate::repository::property_repo::PropertyRepo;
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use chrono::Utc;
use anyhow::{Result, anyhow};
use log::{info, warn, error};
use std::collections::HashMap;

#[derive(Clone)]
pub struct PropertyScraper {
    client: Client,
    repository: PropertyRepo,
}

impl PropertyScraper {
    pub fn new(repository: PropertyRepo) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, repository }
    }

    /// Execute a scraping job
    pub async fn run_scraping_job(&self, job: &ScrapingJob) -> Result<ScrapingResult> {
        info!("Starting scraping job: {} for URL: {}", job.name, job.target_url);
        
        let started_at = Utc::now();
        let mut errors = Vec::new();
        let mut properties_scraped = 0;

        match self.scrape_properties(&job.target_url, &job.selectors).await {
            Ok(properties) => {
                info!("Successfully scraped {} properties", properties.len());
                
                for property in properties {
                    match self.save_property(property).await {
                        Ok(_) => properties_scraped += 1,
                        Err(e) => {
                            warn!("Failed to save property: {}", e);
                            errors.push(e.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                error!("Scraping failed: {}", e);
                errors.push(e.to_string());
            }
        }

        let status = if errors.is_empty() && properties_scraped > 0 {
            ScrapingStatus::Completed
        } else if properties_scraped > 0 {
            ScrapingStatus::Completed // Partial success
        } else {
            ScrapingStatus::Failed
        };

        Ok(ScrapingResult {
            job_id: job.id.clone(),
            status,
            properties_scraped,
            errors,
            started_at,
            completed_at: Some(Utc::now()),
        })
    }

    /// Scrape properties from a given URL using CSS selectors
    pub async fn scrape_properties(
        &self,
        url: &str,
        selectors: &PropertySelectors,
    ) -> Result<Vec<PropertyNew>> {
        info!("Fetching HTML from: {}", url);
        
        let response = self.client
            .get(url)
            .send()
            .await?
            .text()
            .await?;

        let document = Html::parse_document(&response);
        let mut properties = Vec::new();

        // Find property containers (assume they're in a common parent)
        let property_containers = self.find_property_containers(&document, selectors)?;
        
        info!("Found {} property containers", property_containers.len());

        for container_html in property_containers {
            match self.extract_property_data(&container_html, selectors, url).await {
                Ok(Some(property)) => properties.push(property),
                Ok(None) => continue, // Skip incomplete properties
                Err(e) => warn!("Failed to extract property data: {}", e),
            }
        }

        Ok(properties)
    }

    /// Find individual property containers in the HTML
    fn find_property_containers(
        &self,
        document: &Html,
        _selectors: &PropertySelectors,
    ) -> Result<Vec<String>> {
        // Try to find a common parent container for properties
        // This is a heuristic approach - in practice, you'd configure this per site
        let container_selectors = vec![
            ".property-item",
            ".listing-item", 
            ".property-card",
            ".property",
            "[data-testid*='property']",
        ];

        for selector_str in container_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                let containers: Vec<String> = document
                    .select(&selector)
                    .map(|element| element.html())
                    .collect();
                
                if !containers.is_empty() {
                    info!("Using container selector: {}", selector_str);
                    return Ok(containers);
                }
            }
        }

        // Fallback: treat the entire document as one container
        warn!("No property containers found, using entire document");
        Ok(vec![document.html()])
    }

    /// Extract property data from a single container HTML
    async fn extract_property_data(
        &self,
        html: &str,
        selectors: &PropertySelectors,
        base_url: &str,
    ) -> Result<Option<PropertyNew>> {
        let fragment = Html::parse_fragment(html);

        // Extract required fields
        let title = self.extract_text(&fragment, &selectors.title)?;
        let address = self.extract_text(&fragment, &selectors.address)?;
        
        // Skip if required fields are missing
        if title.trim().is_empty() || address.trim().is_empty() {
            return Ok(None);
        }

        // Extract optional fields
        let price = self.extract_price(&fragment, &selectors.price).ok();
        let property_type = selectors.property_type
            .as_ref()
            .and_then(|s| self.extract_text(&fragment, s).ok())
            .unwrap_or_else(|| "unknown".to_string());
        
        let bedrooms = selectors.bedrooms
            .as_ref()
            .and_then(|s| self.extract_number(&fragment, s));
        
        let bathrooms = selectors.bathrooms
            .as_ref()
            .and_then(|s| self.extract_number(&fragment, s));
        
        let land_size = selectors.land_size
            .as_ref()
            .and_then(|s| self.extract_float(&fragment, s));
        
        let floor_size = selectors.floor_size
            .as_ref()
            .and_then(|s| self.extract_float(&fragment, s));

        // Infer location data from address
        let (province, city, suburb) = self.parse_address(&address);
        
        // Try to get coordinates (this would typically use a geocoding service)
        let (latitude, longitude) = self.geocode_address(&address).await.unwrap_or((None, None));

        Ok(Some(PropertyNew {
            title,
            price,
            address,
            province,
            city,
            suburb,
            property_type,
            bedrooms,
            bathrooms,
            garage_spaces: None, // Would need specific selector
            land_size,
            floor_size,
            source_url: base_url.to_string(),
            latitude,
            longitude,
        }))
    }

    /// Extract text content using CSS selector
    fn extract_text(&self, html: &Html, selector: &str) -> Result<String> {
        let selector = Selector::parse(selector)
            .map_err(|e| anyhow!("Invalid CSS selector '{}': {}", selector, e))?;
        
        html.select(&selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string())
            .ok_or_else(|| anyhow!("Element not found for selector: {:?}", selector))
    }

    /// Extract price from text, handling various formats
    fn extract_price(&self, html: &Html, selector_opt: &Option<String>) -> Result<i64> {
        let selector = selector_opt.as_ref()
            .ok_or_else(|| anyhow!("Price selector not provided"))?;
        
        let text = self.extract_text(html, selector)?;
        self.parse_price(&text)
    }

    /// Parse price from text string
    fn parse_price(&self, text: &str) -> Result<i64> {
        // Remove common currency symbols and separators
        let cleaned = text
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>();
        
        cleaned.parse::<i64>()
            .map_err(|_| anyhow!("Could not parse price from: {}", text))
    }

    /// Extract numeric value (for bedrooms, bathrooms, etc.)
    fn extract_number(&self, html: &Html, selector: &str) -> Option<i16> {
        self.extract_text(html, selector)
            .ok()?
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .ok()
    }

    /// Extract float value (for sizes)
    fn extract_float(&self, html: &Html, selector: &str) -> Option<f64> {
        let text = self.extract_text(html, selector).ok()?;
        
        // Extract numbers and decimal points
        let cleaned: String = text
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        
        cleaned.parse().ok()
    }

    /// Parse address into province, city, suburb components
    fn parse_address(&self, address: &str) -> (String, String, Option<String>) {
        // This is a simplified parser - in practice, you'd use a proper address parsing service
        let parts: Vec<&str> = address.split(',').map(|s| s.trim()).collect();
        
        match parts.len() {
            1 => ("Unknown".to_string(), parts[0].to_string(), None),
            2 => (parts[1].to_string(), parts[0].to_string(), None),
            3 => (parts[2].to_string(), parts[1].to_string(), Some(parts[0].to_string())),
            _ => {
                // Take last as province, second-to-last as city, first as suburb
                let province = parts.last().unwrap_or(&"Unknown").to_string();
                let city = parts.get(parts.len() - 2).unwrap_or(&"Unknown").to_string();
                let suburb = if parts.len() > 2 { Some(parts[0].to_string()) } else { None };
                (province, city, suburb)
            }
        }
    }

    /// Geocode address to get coordinates (mock implementation)
    async fn geocode_address(&self, _address: &str) -> Result<(Option<f64>, Option<f64>)> {
        // In a real implementation, you would call a geocoding service like Google Maps API
        // For now, return None to indicate coordinates are not available
        Ok((None, None))
    }

    /// Save property to database, handling duplicates
    async fn save_property(&self, property: PropertyNew) -> Result<()> {
        match self.repository.create_property(actix_web::web::Json(property)).await {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_error)) 
                if db_error.constraint() == Some("unique_property_url") => {
                // Property already exists, skip silently
                Ok(())
            }
            Err(e) => Err(anyhow!("Database error: {}", e)),
        }
    }

    /// Get scraping statistics
    pub async fn get_scraping_stats(&self) -> Result<HashMap<String, i64>> {
        // This would query the database for statistics
        // For now, return mock data
        let mut stats = HashMap::new();
        stats.insert("total_properties".to_string(), 0);
        stats.insert("properties_today".to_string(), 0);
        stats.insert("active_jobs".to_string(), 0);
        Ok(stats)
    }
}
