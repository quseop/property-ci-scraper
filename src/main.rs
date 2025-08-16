mod repository;
mod models;
mod controller;
mod service;

use actix_web::{
    middleware::Logger,
    web::{self, ServiceConfig},
};

use shuttle_actix_web::ShuttleActixWeb;
use sqlx::PgPool;
use log::info;

// Import existing controllers
use crate::controller::controller::{get_all_properties, get_property_by_id, post_property, put_property, AppState};

// Import new controllers
use crate::controller::scraping_controller::{
    ScrapingAppState, get_scraping_jobs, create_scraping_job, get_scraping_job,
    delete_scraping_job, run_scraping_job, get_scraping_results, get_job_results,
    get_scraping_stats, get_property_stats, export_properties, export_ml_dataset,
    get_export_stats, search_properties, get_recent_properties, create_sample_job
};

// Import services
use crate::service::scraper::PropertyScraper;
use crate::service::scheduler::ScrapingScheduler;
use crate::service::export::DataExportService;
use crate::repository::property_repo::PropertyRepo;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    // Logging is already initialized by shuttle runtime
    info!("Starting Property CI Scraper service");
    
    // Run database migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create repository
    let repository = PropertyRepo::new(pool);
    
    // Create services
    let scraper = PropertyScraper::new(repository.clone());
    let export_service = DataExportService::new(repository.clone());
    
    // Create and start scheduler
    let scheduler = ScrapingScheduler::new(scraper.clone())
        .await
        .expect("Failed to create scheduler");
    
    scheduler.start()
        .await
        .expect("Failed to start scheduler");
    
    info!("Scheduler started successfully");
    
    // Create application states
    let basic_state = web::Data::new(AppState { 
        repository: repository.clone() 
    });
    
    let scraping_state = web::Data::new(ScrapingAppState {
        repository: repository.clone(),
        scraper,
        scheduler: web::Data::new(scheduler),
        export_service,
    });

    let config = move |cfg: &mut ServiceConfig| {
        cfg
            // Basic CRUD endpoints for properties
            .service(
                web::scope("/properties")
                    .wrap(Logger::default())
                    .service(get_all_properties)
                    .service(get_property_by_id)
                    .service(post_property)
                    .service(put_property)
                    .app_data(basic_state)
            )
            // Advanced property endpoints
            .service(
                web::scope("/api/v1")
                    .wrap(Logger::default())
                    // Property search and stats
                    .service(search_properties)
                    .service(get_recent_properties)
                    .service(get_property_stats)
                    
                    // Scraping job management
                    .service(get_scraping_jobs)
                    .service(create_scraping_job)
                    .service(get_scraping_job)
                    .service(delete_scraping_job)
                    .service(run_scraping_job)
                    .service(create_sample_job)
                    
                    // Scraping results and stats
                    .service(get_scraping_results)
                    .service(get_job_results)
                    .service(get_scraping_stats)
                    
                    // Data export endpoints
                    .service(export_properties)
                    .service(export_ml_dataset)
                    .service(get_export_stats)
                    
                    .app_data(scraping_state)
            )
            // Health check endpoint
            .route("/health", web::get().to(health_check))
            // API documentation endpoint
            .route("/", web::get().to(api_info));
    };

    Ok(config.into())
}

/// Health check endpoint
async fn health_check() -> actix_web::Result<web::Json<serde_json::Value>> {
    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "property-ci-scraper",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// API information endpoint
async fn api_info() -> actix_web::Result<web::Json<serde_json::Value>> {
    Ok(web::Json(serde_json::json!({
        "service": "Property CI Scraper",
        "version": "0.1.0",
        "description": "A comprehensive Rust web service for property data scraping, storage, and ML dataset generation",
        "endpoints": {
            "health": "GET /health",
            "properties": {
                "list_all": "GET /properties",
                "get_by_id": "GET /properties/{id}",
                "create": "POST /properties",
                "update": "PUT /properties/{id}",
                "search": "GET /api/v1/properties/search",
                "recent": "GET /api/v1/properties/recent",
                "stats": "GET /api/v1/properties/stats"
            },
            "scraping": {
                "jobs": "GET /api/v1/scraping/jobs",
                "create_job": "POST /api/v1/scraping/jobs",
                "get_job": "GET /api/v1/scraping/jobs/{id}",
                "delete_job": "DELETE /api/v1/scraping/jobs/{id}",
                "run_job": "POST /api/v1/scraping/jobs/{id}/run",
                "create_sample": "POST /api/v1/scraping/jobs/sample",
                "results": "GET /api/v1/scraping/results",
                "job_results": "GET /api/v1/scraping/jobs/{id}/results",
                "stats": "GET /api/v1/scraping/stats"
            },
            "export": {
                "export_data": "POST /api/v1/export",
                "ml_dataset": "POST /api/v1/export/ml-dataset",
                "stats": "GET /api/v1/export/stats"
            }
        },
        "features": [
            "Web scraping with configurable CSS selectors",
            "Scheduled scraping jobs with cron expressions",
            "Property data storage in PostgreSQL",
            "Advanced filtering and search",
            "ML dataset export (CSV, JSON, Parquet)",
            "Real-time scraping statistics",
            "Duplicate detection and handling"
        ]
    })))
}
