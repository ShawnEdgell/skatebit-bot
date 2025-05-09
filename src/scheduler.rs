// src/scheduler.rs
use crate::{
    types::Data, // Only Data is needed from types
    map_cache,
    mod_utils,
};
use std::{collections::HashMap, sync::Arc};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, warn, error};
use anyhow::{Context as AnyhowContext, Result as AnyhowResult}; // Import AnyhowResult

pub async fn initialize_and_start_scheduler(app_data: Arc<Data>) -> AnyhowResult<()> {
    let sched = JobScheduler::new().await
        .context("Failed to create new JobScheduler")?;
    
    let data_for_job = app_data.clone();
    // Handle the Result from Job::new_async with `?`
    let job = Job::new_async("0 30 0,6,12,18 * * *", move |_uuid, _l| {
        let job_data_clone = data_for_job.clone();
        Box::pin(async move {
            info!("Scheduled Cache Refresh Task: Starting...");

            info!("Scheduled Task: Refreshing map cache...");
            match map_cache::load_maps_from_remote(&job_data_clone.http_client).await {
                Ok(maps) => {
                    let map_count = maps.len();
                    *job_data_clone.map_cache.write().await = maps;
                    info!(map_count, "Scheduled Task: Map cache refresh success.");
                }
                Err(e) => {
                    error!(error = %e, "Scheduled Task: Map cache refresh failed.");
                }
            }

            info!("Scheduled Task: Refreshing mod cache...");
            let slugs_to_fetch = ["1228", "12104"];
            let mut new_mod_cache_map = HashMap::new();
            let mut total_mods_refreshed = 0;
            let mut versions_refreshed_count = 0;

            for slug in slugs_to_fetch {
                match mod_utils::fetch_mods_for_version(&job_data_clone.http_client, slug).await {
                    Ok(mods) => {
                        info!(count = mods.len(), slug, "Scheduled Task: Fetched mods for slug.");
                        total_mods_refreshed += mods.len();
                        if !mods.is_empty() { versions_refreshed_count += 1; }
                        new_mod_cache_map.insert(slug.to_string(), mods);
                    }
                    Err(e) => {
                        error!(error = %e, slug, "Scheduled Task: Failed to fetch mods for slug.");
                    }
                }
            }

            if total_mods_refreshed > 0 || versions_refreshed_count > 0 {
                *job_data_clone.mod_cache.write().await = new_mod_cache_map;
                info!(
                    total_mods = total_mods_refreshed,
                    versions_loaded = versions_refreshed_count,
                    "Scheduled Task: Mod cache refresh complete."
                );
            } else {
                warn!("Scheduled Task: No new mods loaded, mod cache not updated.");
            }
            info!("Scheduled Cache Refresh Task: Finished.");
        })
    })?; // Handle Result from Job::new_async

    sched.add(job).await.context("Failed to add combined cache refresh job")?;

    sched.start().await.context("Failed to start cache refresh scheduler")?;
    info!("Cache refresh scheduler started. Refreshing at 30 mins past 00,06,12,18 UTC.");
    
    Ok(())
}
