use crate::types::ModioMap;
use crate::utils::autocomplete::basic_autocomplete;

use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::{
    application_command::ApplicationCommandInteraction,
    autocomplete::AutocompleteInteraction,
    InteractionResponseType, MessageFlags,
};

use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

#[derive(serde::Deserialize)]
struct ModioPage {
    maps: Vec<ModioMap>,
}

// Shared mod cache
static MOD_CACHE: Lazy<Arc<RwLock<Vec<ModioMap>>>> = Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("map")
        .description("Search maps from mod.io")
        .create_option(|opt| {
            opt.name("search")
                .description("Search for a map by title")
                .kind(serenity::model::application::command::CommandOptionType::String)
                .set_autocomplete(true)
                .required(true)
        })
}

pub async fn load_cache() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut all_maps = Vec::new();
    for page in 1..=12 {
        let url = format!("https://modio-cache.vercel.app/maps_v2/page_{}.json", page);
        let res = reqwest::get(&url).await?;
        if res.status().is_success() {
            let data: ModioPage = res.json().await?;
            all_maps.extend(data.maps);
        } else {
            break;
        }
    }
    *MOD_CACHE.write().await = all_maps;
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
            .and_then(|mf| Some(mf.download.binary_url.clone()))
            .unwrap_or_else(|| "No download link".to_string());

        let size = entry.modfile
            .as_ref()
            .and_then(|mf| mf.filesize.map(|s| format!("{:.2} MB", s as f64 / (1024.0 * 1024.0))))
            .unwrap_or("Unknown".to_string());

        let tags = entry.tags.as_ref()
            .map(|tags| tags.iter().map(|t| t.name.clone()).collect::<Vec<_>>().join(", "))
            .unwrap_or("No tags".to_string());

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
                            .field("Link", format!("[Download]({})", download_link), false)
                            .image(
                                entry
                                    .logo
                                    .thumb_1280x720
                                    .as_deref()
                                    .unwrap_or(&entry.logo.original)
                            )
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
        let prefix = prefix.to_string().to_lowercase();
        let cache = MOD_CACHE.clone();
        async move {
            let data = cache.read().await;
            Ok(data.iter()
                .filter(|m| m.name.to_lowercase().contains(&prefix))
                .map(|m| (m.name.clone(), m.name.clone()))
                .take(25)
                .collect())
        }
    }).await
}
