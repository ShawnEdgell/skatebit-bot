use crate::types::{ModioMap, ModioPage, Error as AppError};
use reqwest::Client;
use std::time::Duration;
use tracing::{info, warn, error};
use anyhow::anyhow;

pub async fn load_maps_from_remote(http_client: &Client) -> Result<Vec<ModioMap>, AppError> {
    let mut all_maps = Vec::new();
    let base_url = "https://modio-cache.vercel.app/maps_v2/page_";
    let mut last_successfully_fetched_page = 0;
    let mut total_maps_loaded_this_run = 0;
    let mut pages_checked = 0;

    info!("Starting map cache loading from remote...");

    for page_num in 1..=20 {
        pages_checked = page_num;
        let url = format!("{}{}.json", base_url, page_num);
        info!(page = page_num, url = %url, "Fetching map page...");

        let response = http_client.get(&url).send().await
            .map_err(|e| { warn!(error = %e, page = page_num, url = %url, "Request error during map cache loading"); e })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            info!(page = page_num, "Page not found. Assuming end of map pages.");
            last_successfully_fetched_page = page_num -1;
            break;
        }

        if !response.status().is_success() {
            let status = response.status();
            warn!(page = page_num, url = %url, status = %status, "Failed to fetch map page with non-success status. Stopping.");
            break;
        }

        let page_data = response.json::<ModioPage>().await
            .map_err(|e| { error!(error = %e, page = page_num, url = %url, "Failed to parse JSON for map page"); e })?;

        if page_data.map_entries.is_empty() {
            info!(page = page_num, "Received empty map list. Assuming end of map pages.");
            last_successfully_fetched_page = if page_num > 0 { page_num -1 } else { 0 };
            break;
        }
        total_maps_loaded_this_run += page_data.map_entries.len();
        all_maps.extend(page_data.map_entries);
        last_successfully_fetched_page = page_num;

        tokio::time::sleep(Duration::from_millis(150)).await;
    }

    if total_maps_loaded_this_run > 0 {
        info!(
            count = total_maps_loaded_this_run,
            pages = last_successfully_fetched_page,
            "Map cache updated successfully."
        );
    } else if pages_checked > 0 && last_successfully_fetched_page == 0 && all_maps.is_empty() {
         warn!(
            pages_checked = pages_checked,
            "Map cache loading: No maps found or error on first page(s)."
        );
    } else if pages_checked > 0 {
         info!(
            pages_checked = pages_checked,
            "Map cache: No new maps found or an issue occurred after the first page."
        );
    }

    if all_maps.is_empty() && last_successfully_fetched_page == 0 && pages_checked > 0 {
        return Err(anyhow!("Initial map cache loading failed: Could not retrieve or parse critical map page(s)."));
    }

    Ok(all_maps)
}