// src/commands/map_cmd.rs
use crate::types::{Context, Error, ApiModioMap, BOT_EMBED_COLOR, ApiModioModfile, ApiModioTag};
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use tracing::{info, warn};

async fn map_name_autocomplete(
    ctx: Context<'_>,
    partial: &str,
) -> Vec<String> {
    let partial_lowercase = partial.to_lowercase();
    let map_cache_guard = ctx.data().map_cache.read().await; // This now contains Vec<ApiModioMap>

    map_cache_guard
        .iter()
        .filter(move |map_entry: &&ApiModioMap| map_entry.name.to_lowercase().contains(&partial_lowercase)) // UPDATED HERE
        .map(|map_entry: &ApiModioMap| map_entry.name.clone()) // UPDATED HERE
        .take(25)
        .collect()
}

/// Searches the cached mod.io map list by title.
#[poise::command(slash_command, prefix_command)]
pub async fn map(
    ctx: Context<'_>,
    #[description = "Map name to search for"]
    #[autocomplete = "map_name_autocomplete"]
    search: String,
) -> Result<(), Error> {
    info!(user = %ctx.author().name, query = %search, "Map command received");

    let map_cache_guard = ctx.data().map_cache.read().await; // This is Vec<ApiModioMap>
    let search_lowercase = search.to_lowercase();

    let found_map: Option<&ApiModioMap> = map_cache_guard // UPDATED HERE
        .iter()
        .find(|m| m.name.to_lowercase() == search_lowercase);

    let reply = if let Some(entry) = found_map { // entry is now &ApiModioMap
        info!(map_name = %entry.name, "Map found in cache");

        // Ensure these field accesses match your ApiModioMap and its sub-structs
        let author = &entry.submitted_by.username; // Assuming ApiModioUser has username

        let download_link = entry
            .modfile
            .as_ref()
            .map(|mf: &ApiModioModfile| mf.download.binary_url.as_str()) // Option<&str>
            .unwrap_or("N/A");

        let download_field_value = if download_link == "N/A" {
            "No download link provided".to_string()
        } else {
            format!("[Download Map]({})", download_link)
        };

        let size_mb = entry
            .modfile
            .as_ref()
            .and_then(|mf: &ApiModioModfile| mf.filesize) // filesize is Option<i64>
            .map(|s| format!("{:.2} MB", s as f64 / (1024.0 * 1024.0)))
            .unwrap_or_else(|| "Unknown size".to_string());
        
        let tags_str = entry
            .tags
            .as_ref()
            .filter(|tags_vec| !tags_vec.is_empty())
            .map(|tags_vec| tags_vec.iter().map(|t: &ApiModioTag| t.name.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_else(|| "No tags".to_string());

        // Assuming ApiModioLogo has thumb_1280x720 (Option<String>) and original (String)
        let image_url = entry.logo.thumb_1280x720.as_deref()
            .unwrap_or_else(|| entry.logo.original.as_str());


        let embed = serenity::CreateEmbed::default()
            .title(&entry.name)
            .url(&entry.profile_url) // Assuming profile_url is directly in ApiModioMap
            .description(&entry.summary)
            .color(BOT_EMBED_COLOR)
            .image(image_url)
            .field("Author", author, true)
            .field("Size", &size_mb, true) // Changed to size_mb
            .field("Tags", tags_str, false) // Changed to tags_str
            .field("Link", download_field_value, false)
            .timestamp(serenity::Timestamp::now())
            .footer(serenity::CreateEmbedFooter::new(format!("Source: mod.io | Requested by {}", ctx.author().name)));
        
        CreateReply::default().embed(embed)

    } else {
        warn!(query = %search, "Map not found in cache");
        CreateReply::default()
            .content(format!("❌ Map not found matching: '{}'", search))
            .ephemeral(true)
    };

    ctx.send(reply).await?;

    Ok(())
}