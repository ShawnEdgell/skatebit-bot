use crate::{
    types::ModioMap,
    utils::{
        autocomplete::basic_autocomplete,
        constants::BOT_EMBED_COLOR,
        interaction::get_str_option,
    },
};

use serenity::{
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed,
        CreateInteractionResponse, CreateInteractionResponseMessage
    },
    client::Context,
    model::application::{
        CommandInteraction, CommandOptionType,
    },
};
use std::{error::Error, sync::Arc};
use tokio::sync::RwLock;
use tokio::time::Duration;
use once_cell::sync::Lazy;

#[derive(serde::Deserialize)]
struct ModioPage {
    maps: Vec<ModioMap>,
}

static MOD_CACHE: Lazy<Arc<RwLock<Vec<ModioMap>>>> = Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

pub fn register() -> CreateCommand {
    CreateCommand::new("map")
        .description("Search maps from mod.io")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "search",
                "Search for a map by title",
            )
            .set_autocomplete(true)
            .required(true),
        )
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
                if page_data.maps.is_empty() && page_num > 1 {
                    last_successfully_fetched_page = page_num -1;
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
    } else if last_successfully_fetched_page > 0 && total_maps_loaded_this_run == 0 {
        println!(
            "ℹ️ Map cache check completed. No new maps found after checking {} pages. Cache remains unchanged.",
            last_successfully_fetched_page
        );
    } else if last_successfully_fetched_page == 0 && total_maps_loaded_this_run == 0 {
        eprintln!("❌ Map cache loading failed: Could not retrieve or parse the initial map page(s). Ensure the source is available.");
    }

    Ok(())
}

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let query = get_str_option(interaction, "search")
        .unwrap_or("")
        .to_lowercase();

    let cache = MOD_CACHE.read().await;
    let response_message: CreateInteractionResponseMessage;

    if let Some(entry) = cache.iter().find(|m| m.name.to_lowercase().contains(&query)) {
        let author = entry.submitted_by.username.clone();
        let download_link = entry
            .modfile
            .as_ref()
            .map(|mf| mf.download.binary_url.clone())
            .unwrap_or_else(|| "No download link".to_string());

        let size = entry
            .modfile
            .as_ref()
            .and_then(|mf| mf.filesize.map(|s| format!("{:.2} MB", s as f64 / (1024.0 * 1024.0))))
            .unwrap_or_else(|| "Unknown size".to_string());

        let tags = entry
            .tags
            .as_ref()
            .filter(|tags_vec| !tags_vec.is_empty())
            .map(|tags_vec| tags_vec.iter().map(|t| t.name.clone()).collect::<Vec<_>>().join(", "))
            .unwrap_or_else(|| "No tags".to_string());

        let embed = CreateEmbed::new()
            .title(&entry.name)
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
            .color(BOT_EMBED_COLOR);
        response_message = CreateInteractionResponseMessage::new().add_embed(embed);
    } else {
        response_message = CreateInteractionResponseMessage::new()
            .content("❌ No map found for that query.")
            .ephemeral(true);
    }

    let response = CreateInteractionResponse::Message(response_message);
    interaction.create_response(&ctx.http, response).await?;
    Ok(())
}

pub async fn autocomplete(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    basic_autocomplete(ctx, interaction, "search", |current_search_input| {
        let query_owned = current_search_input.to_string().to_lowercase();
        let cache_clone = Arc::clone(&MOD_CACHE);
        async move {
            let data = cache_clone.read().await;
            Ok(data
                .iter()
                .filter(|m| m.name.to_lowercase().contains(&query_owned))
                .map(|m| (m.name.clone(), m.name.clone()))
                .take(25)
                .collect())
        }
    })
    .await
}