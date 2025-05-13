// src/map_cache.rs
use crate::types::{ApiModioMap, GoApiMapsResponse}; // Use the new structs
use reqwest::Client;
use tracing::{info, warn, error, instrument}; // Added instrument for consistency if you use it elsewhere
use anyhow::{anyhow, Context, Result}; // Using anyhow for Result type and context

// This function will be called by your scheduler and during initial bot setup.
// It now fetches all maps in one go from your new Go API.
#[instrument(skip(http_client))] // Skips logging the http_client instance itself
pub async fn load_maps_from_go_api(
    http_client: &Client,
    api_maps_url: &str, // e.g., "https://api.skatebit.app/api/v1/skaterxl/maps"
) -> Result<Vec<ApiModioMap>> { // Return a Result with Vec<ApiModioMap> or an anyhow::Error
    info!(url = %api_maps_url, "Map Cache: Starting map data fetch from self-hosted Go API...");

    let response = http_client.get(api_maps_url)
        .timeout(std::time::Duration::from_secs(30)) // Add a timeout for the request
        .send()
        .await
        .context(format!("Map Cache: HTTP request to {} failed", api_maps_url))?;

    if !response.status().is_success() {
        let status = response.status();
        // Attempt to get error body for better debugging
        let error_body = response.text().await.unwrap_or_else(|e| format!("Failed to read error body: {}",e));
        error!(
            url = %api_maps_url,
            status = %status,
            body = %error_body,
            "Map Cache: Failed to fetch maps from Go API. Non-success status."
        );
        return Err(anyhow!("Map Cache: Go API request to {} returned status {}", api_maps_url, status));
    }

    // Try to parse the JSON response
    let api_response = response.json::<GoApiMapsResponse>().await
        .context(format!("Map Cache: Failed to parse JSON response from {}", api_maps_url))?;

    // Optional: Validate itemType if you want to be extra sure
    if api_response.item_type != "maps" {
        warn!(
            expected = "maps",
            got = %api_response.item_type,
            url = %api_maps_url,
            "Map Cache: Unexpected itemType received from Go API."
        );
        // You might choose to error here, or proceed if items look correct
    }

    info!(
        count = api_response.items.len(),
        last_updated = %api_response.last_updated, // This is a string from Go API
        "Map Cache: Successfully fetched and parsed map data from Go API."
    );

    Ok(api_response.items)
}