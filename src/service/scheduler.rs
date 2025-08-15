use crate::models::property::{ScrapingJob, ScrapingResult};
use crate::service::scraper::PropertyScraper;
use tokio_cron_scheduler::{Job, JobScheduler};
use std::sync::Arc;
use log::{info, error, warn};
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone)]
pub struct ScrapingScheduler {
    scheduler: Arc<JobScheduler>,
    jobs: Arc<RwLock<HashMap<String, ScrapingJob>>>,
    scraper: PropertyScraper,
    results: Arc<RwLock<HashMap<String, ScrapingResult>>>,
}

impl ScrapingScheduler {
    pub async fn new(scraper: PropertyScraper) -> Result<Self> {
        let scheduler = JobScheduler::new().await?;
        
        Ok(Self {
            scheduler: Arc::new(scheduler),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            scraper,
            results: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        self.scheduler.start().await?;
        info!("Scraping scheduler started");
        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) -> Result<()> {
        self.scheduler.shutdown().await?;
        info!("Scraping scheduler stopped");
        Ok(())
    }

    /// Add a new scraping job
    pub async fn add_job(&self, mut job: ScrapingJob) -> Result<String> {
        if job.id.is_empty() {
            job.id = Uuid::new_v4().to_string();
        }

        let job_id = job.id.clone();
        let cron_expression = job.schedule.clone();
        
        // Create a closure for the job execution
        let scraper = self.scraper.clone();
        let job_clone = job.clone();
        let results = self.results.clone();
        
        let scheduled_job = Job::new_async(&cron_expression, move |_uuid, _l| {
            let scraper = scraper.clone();
            let job = job_clone.clone();
            let results = results.clone();
            
            Box::pin(async move {
                info!("Executing scheduled scraping job: {}", job.name);
                
                match scraper.run_scraping_job(&job).await {
                    Ok(result) => {
                        info!(
                            "Scraping job '{}' completed: {} properties scraped", 
                            job.name, 
                            result.properties_scraped
                        );
                        
                        // Store the result
                        let mut results_guard = results.write().await;
                        results_guard.insert(job.id.clone(), result);
                    }
                    Err(e) => {
                        error!("Scraping job '{}' failed: {}", job.name, e);
                        
                        let failed_result = ScrapingResult {
                            job_id: job.id.clone(),
                            status: crate::models::property::ScrapingStatus::Failed,
                            properties_scraped: 0,
                            errors: vec![e.to_string()],
                            started_at: Utc::now(),
                            completed_at: Some(Utc::now()),
                        };
                        
                        let mut results_guard = results.write().await;
                        results_guard.insert(job.id.clone(), failed_result);
                    }
                }
            })
        })?;

        self.scheduler.add(scheduled_job).await?;
        
        // Store the job configuration
        let mut jobs_guard = self.jobs.write().await;
        jobs_guard.insert(job_id.clone(), job);
        
        info!("Added scraping job: {} with schedule: {}", job_id, cron_expression);
        Ok(job_id)
    }

    /// Remove a scraping job
    pub async fn remove_job(&self, job_id: &str) -> Result<()> {
        // Remove from scheduler (this is tricky with tokio-cron-scheduler)
        // For now, we'll mark it as inactive
        let mut jobs_guard = self.jobs.write().await;
        if let Some(job) = jobs_guard.get_mut(job_id) {
            job.active = false;
            info!("Deactivated scraping job: {}", job_id);
        } else {
            warn!("Job {} not found", job_id);
        }
        
        Ok(())
    }

    /// Get all configured jobs
    pub async fn get_jobs(&self) -> Vec<ScrapingJob> {
        let jobs_guard = self.jobs.read().await;
        jobs_guard.values().cloned().collect()
    }

    /// Get job by ID
    pub async fn get_job(&self, job_id: &str) -> Option<ScrapingJob> {
        let jobs_guard = self.jobs.read().await;
        jobs_guard.get(job_id).cloned()
    }

    /// Get recent scraping results
    pub async fn get_results(&self, limit: Option<usize>) -> Vec<ScrapingResult> {
        let results_guard = self.results.read().await;
        let mut results: Vec<ScrapingResult> = results_guard.values().cloned().collect();
        
        // Sort by completion time (most recent first)
        results.sort_by(|a, b| {
            let a_time = a.completed_at.unwrap_or(a.started_at);
            let b_time = b.completed_at.unwrap_or(b.started_at);
            b_time.cmp(&a_time)
        });
        
        if let Some(limit) = limit {
            results.truncate(limit);
        }
        
        results
    }

    /// Get result for a specific job
    pub async fn get_job_result(&self, job_id: &str) -> Option<ScrapingResult> {
        let results_guard = self.results.read().await;
        results_guard.get(job_id).cloned()
    }

    /// Run a job immediately (manual trigger)
    pub async fn run_job_now(&self, job_id: &str) -> Result<ScrapingResult> {
        let job = {
            let jobs_guard = self.jobs.read().await;
            jobs_guard.get(job_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("Job {} not found", job_id))?
        };

        info!("Manually triggering scraping job: {}", job.name);
        let result = self.scraper.run_scraping_job(&job).await?;

        // Store the result
        let mut results_guard = self.results.write().await;
        results_guard.insert(job_id.to_string(), result.clone());

        Ok(result)
    }

    /// Update job configuration
    pub async fn update_job(&self, job_id: &str, updated_job: ScrapingJob) -> Result<()> {
        let mut jobs_guard = self.jobs.write().await;
        
        if jobs_guard.contains_key(job_id) {
            jobs_guard.insert(job_id.to_string(), updated_job);
            info!("Updated scraping job: {}", job_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Job {} not found", job_id))
        }
    }

    /// Get scheduler statistics
    pub async fn get_stats(&self) -> HashMap<String, i64> {
        let jobs_guard = self.jobs.read().await;
        let results_guard = self.results.read().await;
        
        let total_jobs = jobs_guard.len() as i64;
        let active_jobs = jobs_guard.values().filter(|job| job.active).count() as i64;
        let total_runs = results_guard.len() as i64;
        let successful_runs = results_guard.values()
            .filter(|result| matches!(result.status, crate::models::property::ScrapingStatus::Completed))
            .count() as i64;
        
        let mut stats = HashMap::new();
        stats.insert("total_jobs".to_string(), total_jobs);
        stats.insert("active_jobs".to_string(), active_jobs);
        stats.insert("total_runs".to_string(), total_runs);
        stats.insert("successful_runs".to_string(), successful_runs);
        
        stats
    }

    /// Clean up old results (keep only the last N results per job)
    pub async fn cleanup_results(&self, keep_per_job: usize) -> Result<()> {
        let mut results_guard = self.results.write().await;
        
        // Group results by job_id
        let mut job_results: HashMap<String, Vec<(String, ScrapingResult)>> = HashMap::new();
        
        for (key, result) in results_guard.iter() {
            job_results
                .entry(result.job_id.clone())
                .or_default()
                .push((key.clone(), result.clone()));
        }
        
        // For each job, keep only the most recent N results
        for (_, mut results) in job_results {
            if results.len() > keep_per_job {
                // Sort by completion time (most recent first)
                results.sort_by(|a, b| {
                    let a_time = a.1.completed_at.unwrap_or(a.1.started_at);
                    let b_time = b.1.completed_at.unwrap_or(b.1.started_at);
                    b_time.cmp(&a_time)
                });
                
                // Remove old results
                for (key, _) in results.iter().skip(keep_per_job) {
                    results_guard.remove(key);
                }
            }
        }
        
        info!("Cleaned up old scraping results");
        Ok(())
    }
}

/// Helper function to create common cron schedules
pub struct CronSchedules;

impl CronSchedules {
    pub const DAILY: &'static str = "0 0 2 * * *";           // 2 AM daily
    pub const HOURLY: &'static str = "0 0 * * * *";          // Every hour
    pub const WEEKLY: &'static str = "0 0 2 * * 0";          // 2 AM on Sunday
    pub const TWICE_DAILY: &'static str = "0 0 2,14 * * *";  // 2 AM and 2 PM
    
    /// Create a custom cron for every N hours
    pub fn every_n_hours(hours: u8) -> String {
        format!("0 0 */{} * * *", hours)
    }
    
    /// Create a cron for specific time daily
    pub fn daily_at(hour: u8, minute: u8) -> String {
        format!("0 {} {} * * *", minute, hour)
    }
}
