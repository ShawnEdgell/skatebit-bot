use crate::{
    types::Data,
    mod_utils,
};
use std::{collections::HashMap, sync::Arc}; // Removed env
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, warn, error};
use anyhow::{Context as AnyhowContext, Result as AnyhowResult};

pub async fn initialize_and_start_scheduler(app_data: Arc<Data>) -> AnyhowResult<()> {
    let sched = JobScheduler::new().await
        .context("Failed to create new JobScheduler")?;
    
    let data_for_job = app_data.clone();
    let job = Job::new_async("0 30 0,6,12,18 * * *", move |_uuid, _l| {
        let job_data_clone = data_for_job.clone();
        Box::pin(async move {
            info!("Scheduled Task: Starting slug-based mod cache refresh...");
            
            let slugs_to_fetch = ["1228", "12104"];
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
                warn!("Scheduled Task: No new slug-based mods loaded, slug-based mod cache not updated.");
            }
            info!("Scheduled Task: Finished.");
        })
    })?;

    sched.add(job).await.context("Failed to add slug-based mod cache refresh job")?;
    sched.start().await.context("Failed to start slug-based mod cache refresh scheduler")?;
    info!("Slug-based mod cache refresh scheduler started. Job scheduled for '0 30 0,6,12,18 * * *' (UTC).");
    
    Ok(())
}
