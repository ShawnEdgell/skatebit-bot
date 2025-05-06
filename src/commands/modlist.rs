// src/commands/modlist.rs

use serenity::builder::CreateApplicationCommand;
use serenity::model::{
    application::command::CommandOptionType,
    application::interaction::{
        application_command::ApplicationCommandInteraction,
        message_component::MessageComponentInteraction,
        InteractionResponseType,
        MessageFlags,
    },
    application::component::ButtonStyle,
    prelude::{MessageId, UserId},
};
use serenity::client::Context;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize, Clone)]
struct ModEntry {
    title: String,
    author: Option<String>,
    #[serde(rename = "workingVersion")]
    working_version: Option<String>,
    keybind: Option<String>,
    features: Option<Vec<String>>,
    note: Option<String>,
    #[serde(rename = "downloadLinks")]
    download_links: Option<Vec<DownloadLink>>,
}

#[derive(Deserialize, Clone)]
struct DownloadLink {
    url: String,
    label: String,
}

lazy_static::lazy_static! {
    /// Stores pagination state: (user, message) -> (pages, current_index)
    static ref PAGINATED_DATA: Arc<Mutex<HashMap<(UserId, MessageId), (Vec<(String, String)>, usize)>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Register the /modlist command (version alias + optional DM flag)
pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("modlist")
       .description("Fetch mod list by alias (alpha, beta, or public)")
       // version option
       .create_option(|opt| {
           opt.name("version")
              .description("Which version: alpha, beta, or public")
              .kind(CommandOptionType::String)
              .required(true)
       })
       // dm flag
       .create_option(|opt| {
           opt.name("dm")
              .description("Send the full list via DM instead of ephemeral chat")
              .kind(CommandOptionType::Boolean)
              .required(false)
       })
}

/// Map alias to API version strings
fn resolve_version(input: &str) -> Option<(&'static str, &'static str)> {
    match input.to_lowercase().as_str() {
        "alpha" => Some(("v1.2.2.8", "1228")),
        "beta"  => Some(("v1.2.2.8", "1228")),
        "public"=> Some(("v1.2.10.4", "12104")),
        _ => None,
    }
}

/// Handle /modlist invocation and send paginated embed
pub async fn run(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse version alias
    let input = interaction.data.options.iter()
        .find(|o| o.name == "version").unwrap()
        .value.as_ref().unwrap().as_str().unwrap();
    let (label, version) = resolve_version(input)
        .ok_or("Invalid version alias")?;

    // Parse DM flag
    let dm_flag = interaction.data.options.iter()
        .find(|o| o.name == "dm")
        .and_then(|o| o.value.as_ref())
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Fetch mod data
    let url = format!("https://skatebit-api.vercel.app/api/mods/{}", version);
    let mods: Vec<ModEntry> = Client::new()
        .get(&url)
        .send().await?
        .json().await?;

    // Build pages of embeds
    let chunk_size = 4;
    let mut pages = Vec::new();
    for (i, chunk) in mods.chunks(chunk_size).enumerate() {
        let mut desc = String::new();
        for m in chunk {
            let author = m.author.as_deref().unwrap_or("Unknown");
            let ver = m.working_version.as_deref().unwrap_or("N/A");
            let keybind = m.keybind.as_deref().unwrap_or("None");
            let features = m.features.as_ref().map(|f| f.join(", ")).unwrap_or_default();
            let note = m.note.as_deref().unwrap_or("");
            let note_line = if note.is_empty() { String::new() } else { format!("Note: {}", note) };
            let downloads = m.download_links.as_ref().map(|links|
                links.iter().map(|l| format!("[{}]({})", l.label, l.url)).collect::<Vec<_>>().join(" | ")
            ).unwrap_or_default();

            desc.push_str(&format!(
                "**{}**\nby {}\nVersion: {}\nKeybind: {}\nFeatures: {}\n{}\n{}\n\n",
                m.title, author, ver, keybind, features, note_line, downloads
            ));
        }
        let title_text = if i == 0 {
            format!("{} Mod List", label)
        } else {
            format!("{} Mod List (Page {})", label, i + 1)
        };
        pages.push((title_text, desc));
    }

    // Send initial embed
    let (title, desc) = &pages[0];
    if dm_flag {
        // Acknowledge then DM
        interaction.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| msg.flags(MessageFlags::EPHEMERAL).content("✅ Sending mod list via DM!"))
        }).await?;
        let dm = interaction.user.create_dm_channel(&ctx.http).await?;
        let sent = dm.send_message(&ctx.http, |m| m
            .embed(|e| e.title(title).description(desc))
            .components(|c| c.create_action_row(|row| {
                row.create_button(|b| b.custom_id("prev").label("⏪ Prev").style(ButtonStyle::Primary))
                   .create_button(|b| b.custom_id("next").label("Next ⏩").style(ButtonStyle::Primary))
            })))
            .await?;
        PAGINATED_DATA.lock().await.insert((interaction.user.id, sent.id), (pages, 0));
    } else {
        // Ephemeral in-channel
        interaction.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| msg.flags(MessageFlags::EPHEMERAL)
                    .embed(|e| e.title(title).description(desc))
                    .components(|c| c.create_action_row(|row| {
                        row.create_button(|b| b.custom_id("prev").label("⏪ Prev").style(ButtonStyle::Primary))
                           .create_button(|b| b.custom_id("next").label("Next ⏩").style(ButtonStyle::Primary))
                    }))
                )
        }).await?;
        let msg = interaction.get_interaction_response(&ctx.http).await?;
        PAGINATED_DATA.lock().await.insert((interaction.user.id, msg.id), (pages, 0));
    }

    Ok(())
}

/// Handle Prev/Next button clicks to update the embed
pub async fn handle_pagination(
    ctx: &Context,
    component: &MessageComponentInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut data = PAGINATED_DATA.lock().await;
    let key = (component.user.id, component.message.id);
    if let Some((pages, idx)) = data.get_mut(&key) {
        match component.data.custom_id.as_str() {
            "prev" if *idx > 0 => *idx -= 1,
            "next" if *idx + 1 < pages.len() => *idx += 1,
            _ => {}
        }
        let (title, desc) = &pages[*idx];
        component.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|m| m.embed(|e| e.title(title).description(desc)))
        }).await?;
    } else {
        component.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| m.content("❌ Pagination data not found."))
        }).await?;
    }
    Ok(())
}