use crate::types::ModioMap;
use crate::utils::autocomplete::basic_autocomplete;
use crate::utils::constants::BOT_EMBED_COLOR;

use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::{
    application_command::ApplicationCommandInteraction,
    autocomplete::AutocompleteInteraction,
    InteractionResponseType, MessageFlags,
};
use serenity::model::application::command::CommandOptionType;

use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use once_cell::sync::Lazy;

#[derive(serde::Deserialize)]
struct ModioPage {
    maps: Vec<ModioMap>,
}

static MOD_CACHE: Lazy<Arc<RwLock<Vec<ModioMap>>>> = Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("map")
        .description("Search maps from mod.io")
        .create_option(|opt| {
            opt.name("search")
                .description("Search for a map by title")
                .kind(CommandOptionType::String)
                .set_autocomplete(true)
                .required(true)
        })
}

pub async fn load_cache() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut all_maps = Vec::new();
    let client = reqwest::Client::new();
    let base_url = "https://modio-cache.vercel.app/maps_v2/page_";
    let mut last_successfully_fetched_page = 0;
    let mut total_maps_loaded_this_run = 0;

    for page_num in 1..=100 {
        let url = format!("{}{}.json", base_url, page_num);

        let response = match client.get(&url).send().await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("❌ Request error for {}: {:?}", url, e);
                break;
            }
        };

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            break;
        }

        if !response.status().is_success() {
            eprintln!("❌ Failed to fetch {}: Status {}", url, response.status());
            break;
        }

        match response.json::<ModioPage>().await {
            Ok(page_data) => {
                if page_data.maps.is_empty() {
                    last_successfully_fetched_page = page_num;
                    break;
                }
                total_maps_loaded_this_run += page_data.maps.len();
                all_maps.extend(page_data.maps);
                last_successfully_fetched_page = page_num;
            }
            Err(e) => {
                eprintln!("❌ Failed to parse JSON for {}: {:?}", url, e);
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    if total_maps_loaded_this_run > 0 {
        let mut cache_writer = MOD_CACHE.write().await;
        *cache_writer = all_maps;
        println!(
            "🗺️ Map cache updated successfully: {} maps from {} pages.",
            total_maps_loaded_this_run, last_successfully_fetched_page
        );
    } else if last_successfully_fetched_page > 0 {
        println!(
            "ℹ️ Map cache check completed. No new maps found after checking {} pages. Cache remains unchanged or empty.",
            last_successfully_fetched_page
        );
    } else {
        eprintln!("❌ Map cache loading failed: Could not retrieve or parse the initial map page(s).");
    }

    Ok(())
}

pub async fn run(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let query = interaction.data.options.iter()
        .find(|o| o.name == "search")
        .and_then(|o| o.value.as_ref())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();

    let cache = MOD_CACHE.read().await;
    if let Some(entry) = cache.iter().find(|m| m.name.to_lowercase().contains(&query)) {
        let author = entry.submitted_by.username.clone();
        let download_link = entry.modfile
            .as_ref()
            .map(|mf| mf.download.binary_url.clone())
            .unwrap_or_else(|| "No download link".to_string());

        let size = entry.modfile
            .as_ref()
            .and_then(|mf| mf.filesize.map(|s| format!("{:.2} MB", s as f64 / (1024.0 * 1024.0))))
            .unwrap_or_else(|| "Unknown size".to_string());

        let tags = entry.tags.as_ref()
            .filter(|tags_vec| !tags_vec.is_empty())
            .map(|tags_vec| tags_vec.iter().map(|t| t.name.clone()).collect::<Vec<_>>().join(", "))
            .unwrap_or_else(|| "No tags".to_string());

        interaction.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| {
                    msg.embed(|e| {
                        e.title(&entry.name)
                            .description(&entry.summary)
                            .url(&entry.profile_url)
                            .field("Author", &author, true)
                            .field("Size", &size, true)
                            .field("Tags", &tags, false)
                            .field("Link", format!("[Download Map]({})", download_link), false)
                            .image(
                                entry.logo.thumb_1280x720.as_deref()
                                     .unwrap_or(&entry.logo.original)
                            )
                            .color(BOT_EMBED_COLOR)
                    })
                })
        }).await?;
    } else {
        interaction.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| {
                    msg.flags(MessageFlags::EPHEMERAL)
                        .content("❌ No map found for that query.")
                })
        }).await?;
    }
    Ok(())
}

pub async fn autocomplete(
    ctx: &Context,
    inter: &AutocompleteInteraction,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if inter.data.name != "map" {
        return Ok(());
    }
    basic_autocomplete(ctx, inter, "search", |prefix| {
        let prefix_owned = prefix.to_string().to_lowercase();
        let cache_clone = Arc::clone(&MOD_CACHE);
        async move {
            let data = cache_clone.read().await;
            Ok(data.iter()
                .filter(|m| m.name.to_lowercase().contains(&prefix_owned))
                .map(|m| (m.name.clone(), m.name.clone()))
                .take(25)
                .collect())
        }
    }).await
}