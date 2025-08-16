use actix_web::{web, get, post, delete, HttpResponse, Result, error};
use actix_web::web::{Json, Path, Query};
use log::{info, error};
use std::collections::HashMap;

use crate::models::property::{
    PropertyQuery, ExportRequest, ScrapingJob, ScrapingJobRequest, PropertySelectors, 
    ScrapingResult, PropertyStats
};
use crate::service::scraper::PropertyScraper;
use crate::service::scheduler::{ScrapingScheduler, CronSchedules};
use crate::service::export::{DataExportService, ExportStats};
use crate::repository::property_repo::PropertyRepo;

#[derive(Clone)]
pub struct ScrapingAppState {
    pub repository: PropertyRepo,
    pub scraper: PropertyScraper,
    pub scheduler: web::Data<ScrapingScheduler>,
    pub export_service: DataExportService,
}

/// Get all scraping jobs
#[get("/scraping/jobs")]
pub async fn get_scraping_jobs(state: web::Data<ScrapingAppState>) -> Result<Json<Vec<ScrapingJob>>> {
    info!("Fetching all scraping jobs");
    
    let jobs = state.scheduler.get_jobs().await;
    Ok(Json(jobs))
}

/// Create a new scraping job
#[post("/scraping/jobs")]
pub async fn create_scraping_job(
    job_request: Json<ScrapingJobRequest>, 
    state: web::Data<ScrapingAppState>
) -> Result<HttpResponse> {
    info!("Creating new scraping job: {}", job_request.name);
    
    // Convert request to full job with server-generated fields
    let job = ScrapingJob::from_request(job_request.into_inner());
    
    match state.scheduler.add_job(job).await {
        Ok(job_id) => {
            info!("Successfully created scraping job with ID: {}", job_id);
            Ok(HttpResponse::Created().json(serde_json::json!({
                "job_id": job_id,
                "message": "Scraping job created successfully"
            })))
        }
        Err(e) => {
            error!("Failed to create scraping job: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to create job: {}", e)
            })))
        }
    }
}

/// Get a specific scraping job
#[get("/scraping/jobs/{job_id}")]
pub async fn get_scraping_job(
    path: Path<String>, 
    state: web::Data<ScrapingAppState>
) -> Result<HttpResponse> {
    let job_id = path.into_inner();
    info!("Fetching scraping job: {}", job_id);
    
    match state.scheduler.get_job(&job_id).await {
        Some(job) => Ok(HttpResponse::Ok().json(job)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Job {} not found", job_id)
        })))
    }
}

/// Delete a scraping job
#[delete("/scraping/jobs/{job_id}")]
pub async fn delete_scraping_job(
    path: Path<String>, 
    state: web::Data<ScrapingAppState>
) -> Result<HttpResponse> {
    let job_id = path.into_inner();
    info!("Deleting scraping job: {}", job_id);
    
    match state.scheduler.remove_job(&job_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Job {} deleted successfully", job_id)
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Failed to delete job: {}", e)
        })))
    }
}

/// Trigger a scraping job manually
#[post("/scraping/jobs/{job_id}/run")]
pub async fn run_scraping_job(
    path: Path<String>, 
    state: web::Data<ScrapingAppState>
) -> Result<HttpResponse> {
    let job_id = path.into_inner();
    info!("Manually triggering scraping job: {}", job_id);
    
    match state.scheduler.run_job_now(&job_id).await {
        Ok(result) => {
            info!("Job {} completed with {} properties scraped", 
                  job_id, result.properties_scraped);
            Ok(HttpResponse::Ok().json(result))
        }
        Err(e) => {
            error!("Failed to run job {}: {}", job_id, e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to run job: {}", e)
            })))
        }
    }
}

/// Get recent scraping results
#[get("/scraping/results")]
pub async fn get_scraping_results(
    query: Query<HashMap<String, String>>,
    state: web::Data<ScrapingAppState>
) -> Result<Json<Vec<ScrapingResult>>> {
    let limit = query.get("limit")
        .and_then(|l| l.parse().ok());
    
    info!("Fetching scraping results with limit: {:?}", limit);
    
    let results = state.scheduler.get_results(limit).await;
    Ok(Json(results))
}

/// Get result for a specific job
#[get("/scraping/jobs/{job_id}/results")]
pub async fn get_job_results(
    path: Path<String>, 
    state: web::Data<ScrapingAppState>
) -> Result<HttpResponse> {
    let job_id = path.into_inner();
    info!("Fetching results for job: {}", job_id);
    
    match state.scheduler.get_job_result(&job_id).await {
        Some(result) => Ok(HttpResponse::Ok().json(result)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("No results found for job {}", job_id)
        })))
    }
}

/// Get scraping statistics
#[get("/scraping/stats")]
pub async fn get_scraping_stats(state: web::Data<ScrapingAppState>) -> Result<Json<HashMap<String, i64>>> {
    info!("Fetching scraping statistics");
    
    let stats = state.scheduler.get_stats().await;
    Ok(Json(stats))
}

/// Get property statistics
#[get("/properties/stats")]
pub async fn get_property_stats(state: web::Data<ScrapingAppState>) -> Result<HttpResponse> {
    info!("Fetching property statistics");
    
    match state.repository.get_property_stats().await {
        Ok(stats) => Ok(HttpResponse::Ok().json(stats)),
        Err(e) => {
            error!("Failed to get property stats: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch statistics"
            })))
        }
    }
}

/// Export properties data
#[post("/export")]
pub async fn export_properties(
    export_request: Json<ExportRequest>,
    state: web::Data<ScrapingAppState>
) -> Result<HttpResponse> {
    let format = export_request.format.clone();
    let request = export_request.into_inner();
    info!("Exporting properties with format: {:?}", request.format);
    
    match state.export_service.export_data(request).await {
        Ok(data) => {
            let content_type = match format {
                crate::models::property::ExportFormat::Csv => "text/csv",
                crate::models::property::ExportFormat::Json => "application/json",
                crate::models::property::ExportFormat::Parquet => "application/octet-stream",
            };
            
            Ok(HttpResponse::Ok()
                .content_type(content_type)
                .body(data))
        }
        Err(e) => {
            error!("Failed to export data: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Export failed: {}", e)
            })))
        }
    }
}

/// Export ML-ready dataset
#[post("/export/ml-dataset")]
pub async fn export_ml_dataset(
    query: Query<PropertyQuery>,
    state: web::Data<ScrapingAppState>
) -> Result<HttpResponse> {
    info!("Exporting ML-ready dataset");
    
    let query_params = if query.city.is_some() || query.province.is_some() || 
                          query.min_price.is_some() || query.max_price.is_some() {
        Some(query.into_inner())
    } else {
        None
    };
    
    match state.export_service.export_ml_dataset(query_params).await {
        Ok(data) => {
            Ok(HttpResponse::Ok()
                .content_type("text/csv")
                .append_header(("Content-Disposition", "attachment; filename=ml_dataset.csv"))
                .body(data))
        }
        Err(e) => {
            error!("Failed to export ML dataset: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("ML dataset export failed: {}", e)
            })))
        }
    }
}

/// Get export statistics
#[get("/export/stats")]
pub async fn get_export_stats(state: web::Data<ScrapingAppState>) -> Result<HttpResponse> {
    info!("Fetching export statistics");
    
    match state.export_service.get_export_stats().await {
        Ok(stats) => Ok(HttpResponse::Ok().json(stats)),
        Err(e) => {
            error!("Failed to get export stats: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch export statistics"
            })))
        }
    }
}

/// Search properties with advanced filtering
#[get("/properties/search")]
pub async fn search_properties(
    query: Query<PropertyQuery>,
    state: web::Data<ScrapingAppState>
) -> Result<HttpResponse> {
    info!("Searching properties with filters: {:?}", *query);
    
    // For now, implement basic filtering - in a real app you'd build dynamic SQL
    if let Some(city) = &query.city {
        match state.repository.find_properties_by_city(city).await {
            Ok(properties) => Ok(HttpResponse::Ok().json(properties)),
            Err(e) => {
                error!("Failed to search properties by city: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Search failed"
                })))
            }
        }
    } else if query.min_price.is_some() || query.max_price.is_some() {
        match state.repository.find_properties_by_price_range(
            query.min_price, 
            query.max_price
        ).await {
            Ok(properties) => Ok(HttpResponse::Ok().json(properties)),
            Err(e) => {
                error!("Failed to search properties by price range: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Search failed"
                })))
            }
        }
    } else if let Some(property_type) = &query.property_type {
        match state.repository.find_properties_by_type(property_type).await {
            Ok(properties) => Ok(HttpResponse::Ok().json(properties)),
            Err(e) => {
                error!("Failed to search properties by type: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Search failed"
                })))
            }
        }
    } else {
        // Return all properties if no specific filters
        match state.repository.find_all_properties().await {
            Ok(properties) => Ok(HttpResponse::Ok().json(properties)),
            Err(e) => {
                error!("Failed to fetch all properties: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Search failed"
                })))
            }
        }
    }
}

/// Get recent properties
#[get("/properties/recent")]
pub async fn get_recent_properties(
    query: Query<HashMap<String, String>>,
    state: web::Data<ScrapingAppState>
) -> Result<HttpResponse> {
    let days = query.get("days")
        .and_then(|d| d.parse().ok())
        .unwrap_or(7); // Default to last 7 days
    
    info!("Fetching properties from last {} days", days);
    
    match state.repository.find_recent_properties(days).await {
        Ok(properties) => Ok(HttpResponse::Ok().json(properties)),
        Err(e) => {
            error!("Failed to fetch recent properties: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch recent properties"
            })))
        }
    }
}

/// Create a sample scraping job for testing
#[post("/scraping/jobs/sample")]
pub async fn create_sample_job(state: web::Data<ScrapingAppState>) -> Result<HttpResponse> {
    info!("Creating sample scraping job");
    
    let sample_job = ScrapingJob {
        id: String::new(), // Will be generated
        name: "Sample Property Site".to_string(),
        target_url: "https://example-property-site.com/listings".to_string(),
        selectors: PropertySelectors {
            title: "h2.property-title".to_string(),
            price: Some("span.price".to_string()),
            address: "div.address".to_string(),
            property_type: Some("span.type".to_string()),
            bedrooms: Some("span.bedrooms".to_string()),
            bathrooms: Some("span.bathrooms".to_string()),
            land_size: Some("span.land-size".to_string()),
            floor_size: Some("span.floor-size".to_string()),
        },
        schedule: CronSchedules::DAILY.to_string(),
        active: true,
        created_at: chrono::Utc::now(),
        last_run: None,
    };
    
    match state.scheduler.add_job(sample_job).await {
        Ok(job_id) => {
            info!("Successfully created sample scraping job with ID: {}", job_id);
            Ok(HttpResponse::Created().json(serde_json::json!({
                "job_id": job_id,
                "message": "Sample scraping job created successfully",
                "note": "This is a demo job with example selectors. Update the selectors and URL for real scraping."
            })))
        }
        Err(e) => {
            error!("Failed to create sample scraping job: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to create sample job: {}", e)
            })))
        }
    }
}
