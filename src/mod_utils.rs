use crate::types::{ModEntry, Error as AppError, ModVersionBranch};
use reqwest::Client;
use tracing::{info, warn, error};
use anyhow::anyhow;

pub fn resolve_version_slug(branch: ModVersionBranch) -> &'static str {
    match branch {
        ModVersionBranch::Alpha => "1228",
        ModVersionBranch::BetaPublic => "12104",
    }
}

pub async fn fetch_mods_for_version(
    http_client: &Client,
    version_slug: &str
) -> Result<Vec<ModEntry>, AppError> {
    let url = format!("https://skatebit-api.vercel.app/api/mods/{}", version_slug);
    info!(url = %url, version = %version_slug, "Fetching mods...");

    let response = http_client.get(&url).send().await
        .map_err(|e| { warn!(error = %e, url = %url, "Failed to send request for mods"); e })?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_else(|_| "Failed to get error body".to_string());
        warn!(%url, %status, body = %text, "Received non-success status when fetching mods");
        return Err(anyhow!("API returned status {} for version {}", status, version_slug));
    }

    let mods = response.json::<Vec<ModEntry>>().await
        .map_err(|e| { error!(error = %e, url = %url, "Failed to parse JSON mod data"); e })?;

    info!(count = mods.len(), version = %version_slug, "Successfully fetched and parsed mods.");
    Ok(mods)
}

pub fn format_mod_entry(mod_entry: &ModEntry) -> String {
    let author = mod_entry.author.as_deref().unwrap_or("Unknown");
    let version = mod_entry.working_version.as_deref().unwrap_or("N/A");
    let game_version = mod_entry.game_version.as_deref().unwrap_or("N/A");
    let keybind = mod_entry.keybind.as_deref().unwrap_or("None");
    let features = mod_entry.features.as_ref().map(|f| if f.is_empty() { "N/A".to_string() } else { f.join(", ") }).unwrap_or_else(|| "N/A".to_string());
    let note = mod_entry.note.as_deref().unwrap_or("");
    let note_line = if note.is_empty() { String::new() } else { format!("**Note:** {}\n", note) };
    let downloads = mod_entry.download_links.as_ref().filter(|links| !links.is_empty()).map(|links| { links.iter().map(|l| format!("[{}]({})", l.label, l.url)).collect::<Vec<_>>().join(" | ") }).map(|s| format!("Links: {}", s)).unwrap_or_else(|| "".to_string());

    format!(
        "**Author:** {}\n**Mod Version:** {}\n**Game Version:** {}\n**Keybind:** {}\n**Features:** {}\n{}{}",
        author, version, game_version, keybind, features, note_line, downloads
    ).trim().to_string()
}