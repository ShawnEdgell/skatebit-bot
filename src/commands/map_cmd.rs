use crate::types::{Context, Error, ModioMap, BOT_EMBED_COLOR};
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use tracing::{info, warn};

async fn map_name_autocomplete(
    ctx: Context<'_>,
    partial: &str,
) -> Vec<String> {
    let partial_lowercase = partial.to_lowercase();
    let map_cache_guard = ctx.data().map_cache.read().await;

    map_cache_guard
        .iter()
        .filter(move |map_entry: &&ModioMap| map_entry.name.to_lowercase().contains(&partial_lowercase))
        .map(|map_entry: &ModioMap| map_entry.name.clone())
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

    let map_cache_guard = ctx.data().map_cache.read().await;
    let search_lowercase = search.to_lowercase();

    let found_map: Option<&ModioMap> = map_cache_guard
        .iter()
        .find(|m| m.name.to_lowercase() == search_lowercase);

    let reply = if let Some(entry) = found_map {
        info!(map_name = %entry.name, "Map found in cache");

        let author = &entry.submitted_by.username;

        let download_link = entry
            .modfile
            .as_ref()
            .map(|mf| mf.download.binary_url.as_str())
            .unwrap_or("N/A");

        let download_field_value = if download_link == "N/A" {
            "No download link provided".to_string()
        } else {
            format!("[Download Map]({})", download_link)
        };

        let size = entry
            .modfile
            .as_ref()
            .and_then(|mf| mf.filesize.map(|s| format!("{:.2} MB", s as f64 / (1024.0 * 1024.0))))
            .unwrap_or_else(|| "Unknown size".to_string());

        let tags = entry
            .tags
            .as_ref()
            .filter(|tags_vec| !tags_vec.is_empty())
            .map(|tags_vec| tags_vec.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_else(|| "No tags".to_string());

        let image_url = entry.logo.thumb_1280x720.as_deref().unwrap_or(&entry.logo.original);

        let embed = serenity::CreateEmbed::default()
            .title(&entry.name)
            .url(&entry.profile_url)
            .description(&entry.summary)
            .color(BOT_EMBED_COLOR)
            .image(image_url)
            .field("Author", author, true)
            .field("Size", &size, true)
            .field("Tags", tags, false)
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