// src/scheduler.rs
use crate::{
    types::Data, // Only Data is needed from types
    map_cache,   // Will contain load_maps_from_go_api
    mod_utils,   // For your existing slug-based mod fetching
};
use std::{collections::HashMap, env, sync::Arc}; // Added env for environment variables
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, warn, error};
use anyhow::{Context as AnyhowContext, Result as AnyhowResult};

pub async fn initialize_and_start_scheduler(app_data: Arc<Data>) -> AnyhowResult<()> {
    let sched = JobScheduler::new().await
        .context("Failed to create new JobScheduler")?;
    
    let data_for_job = app_data.clone();
    // Cron for 30 minutes past the hour, every 6 hours (00:30, 06:30, 12:30, 18:30 UTC)
    let job = Job::new_async("0 30 0,6,12,18 * * *", move |_uuid, _l| {
        let job_data_clone = data_for_job.clone();
        Box::pin(async move {
            info!("Scheduled Cache Refresh Task: Starting...");

            // --- Refactor Map Cache Refresh to use Go API ---
            info!("Scheduled Task: Refreshing map cache from self-hosted Go API...");
            
            // Get the Go API base URL from an environment variable
            // For local testing, this might be http://localhost:8083 (if your Go API runs on host port 8083)
            // For Docker Compose on VPS, this will be http://modio_api_service_name:INTERNAL_GO_API_PORT (e.g., http://modio_api:8000)
            let go_api_base_url = env::var("GO_MODIO_API_BASE_URL")
                .unwrap_or_else(|_| {
                    // Fallback if the env var isn't set - adjust this default as needed for safety
                    // For a scheduler, it should ideally always find the env var in production.
                    warn!("GO_MODIO_API_BASE_URL not set, attempting to use a default (e.g., https://api.skatebit.app). This might not be ideal for scheduled tasks.");
                    "https://api.skatebit.app".to_string() // Or a more suitable default/error handling
                });
            
            let maps_api_endpoint = format!("{}/api/v1/skaterxl/maps", go_api_base_url);

            match map_cache::load_maps_from_go_api(&job_data_clone.http_client, &maps_api_endpoint).await {
                Ok(maps) => {
                    let map_count = maps.len();
                    // Assuming map_cache in Data struct is now Arc<RwLock<Vec<ApiModioMap>>> or similar
                    *job_data_clone.map_cache.write().await = maps; 
                    info!(map_count, "Scheduled Task: Map cache refresh from Go API success.");
                }
                Err(e) => {
                    // Using ?e to get the full error chain from anyhow
                    error!(error = ?e, "Scheduled Task: Map cache refresh from Go API failed.");
                }
            }
            // --- End of Refactored Map Cache Refresh ---

            // --- Slug-based Mod Cache Refresh (This part remains unchanged) ---
            info!("Scheduled Task: Refreshing slug-based mod cache...");
            let slugs_to_fetch = ["1228", "12104"]; // Your existing slugs
            let mut new_mod_cache_map = HashMap::new();
            let mut total_mods_refreshed = 0;
            let mut versions_refreshed_count = 0;

            for slug in slugs_to_fetch {
                match mod_utils::fetch_mods_for_version(&job_data_clone.http_client, slug).await {
                    Ok(mods) => {
                        info!(count = mods.len(), slug, "Scheduled Task: Fetched slug-based mods for slug.");
                        total_mods_refreshed += mods.len();
                        if !mods.is_empty() { versions_refreshed_count += 1; }
                        new_mod_cache_map.insert(slug.to_string(), mods);
                    }
                    Err(e) => {
                        error!(error = ?e, slug, "Scheduled Task: Failed to fetch slug-based mods for slug.");
                    }
                }
            }

            if total_mods_refreshed > 0 || versions_refreshed_count > 0 {
                *job_data_clone.mod_cache.write().await = new_mod_cache_map;
                info!(
                    total_mods = total_mods_refreshed,
                    versions_loaded = versions_refreshed_count,
                    "Scheduled Task: Slug-based mod cache refresh complete."
                );
            } else {
                warn!("Scheduled Task: No new slug-based mods loaded, mod cache not updated.");
            }
            // --- End of Unchanged Slug-based Mod Cache Logic ---

            info!("Scheduled Cache Refresh Task: Finished.");
        })
    })?; // Handle Result from Job::new_async

    sched.add(job).await.context("Failed to add combined cache refresh job")?;

    // This starts the scheduler, which will then wait for the cron expression to trigger.
    sched.start().await.context("Failed to start cache refresh scheduler")?;
    info!("Cache refresh scheduler started. Map/Mod refresh job scheduled for '0 30 0,6,12,18 * * *' (UTC).");
    
    Ok(())
}