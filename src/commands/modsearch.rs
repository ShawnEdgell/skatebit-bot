// src/commands/modsearch.rs

use serenity::builder::CreateApplicationCommand;
use serenity::model::{
    application::command::CommandOptionType,
    application::interaction::{
        application_command::ApplicationCommandInteraction,
        autocomplete::AutocompleteInteraction,
        InteractionResponseType,
        MessageFlags,
    },
};
use serenity::client::Context;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;

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

/// Map alias to API code
fn resolve_version(alias: &str) -> Option<&'static str> {
    match alias.to_lowercase().as_str() {
        "alpha" => Some("1228"),
        "beta"  => Some("1228"),
        "public"=> Some("12104"),
        _ => None,
    }
}

/// Register /modsearch version:<alias> query:<keyword>
pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("modsearch")
       .description("Search for a mod by title within a specific version branch")

       // required version option with choices
       .create_option(|opt| {
           opt.name("version")
              .description("Which version branch to search: alpha, beta, public")
              .kind(CommandOptionType::String)
              .add_string_choice("alpha", "alpha")
              .add_string_choice("beta", "beta")
              .add_string_choice("public", "public")
              .required(true)
       })

       // required query with autocomplete
       .create_option(|opt| {
           opt.name("query")
              .description("Keyword to search in mod titles")
              .kind(CommandOptionType::String)
              .set_autocomplete(true)
              .required(true)
       })
}

/// Run /modsearch to fetch and display a single mod
pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Parse version alias
    let version_alias = command.data.options.iter()
        .find(|o| o.name == "version").unwrap()
        .value.as_ref().unwrap().as_str().unwrap();
    let code = resolve_version(version_alias)
        .ok_or("Invalid version alias")?;

    // Parse query
    let query = command.data.options.iter()
        .find(|o| o.name == "query").unwrap()
        .value.as_ref().unwrap().as_str().unwrap()
        .to_lowercase();

    // Fetch mods for that version
    let url = format!("https://skatebit-api.vercel.app/api/mods/{}", code);
    let mods: Vec<ModEntry> = Client::new()
        .get(&url)
        .send().await?
        .json().await?;

    // Find first matching entry
    if let Some(entry) = mods.into_iter()
        .find(|m| m.title.to_lowercase().contains(&query))
    {
        let title_str = entry.title.clone();
        let mut desc = format!(
            "**Author:** {}\n**Version:** {}\n**Keybind:** {}",
            entry.author.as_deref().unwrap_or("Unknown"),
            entry.working_version.as_deref().unwrap_or("N/A"),
            entry.keybind.as_deref().unwrap_or("None"),
        );
        if let Some(features) = entry.features {
            desc.push_str(&format!("\n**Features:** {}", features.join(", ")));
        }
        if let Some(note) = entry.note.filter(|n| !n.is_empty()) {
            desc.push_str(&format!("\n**Note:** {}", note));
        }
        if let Some(links) = entry.download_links {
            let dl = links.iter()
                .map(|l| format!("[{}]({})", l.label, l.url))
                .collect::<Vec<_>>().join(" | ");
            desc.push_str(&format!("\n{}", dl));
        }
        command.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| {
                    data
                        .embed(|e| e.title(&title_str).description(&desc))
                })
        }).await?;
    } else {
        command.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| {
                    data.flags(MessageFlags::EPHEMERAL)
                        .content("No matching mod found in the specified version.")
                })
        }).await?;
    }

    Ok(())
}

/// Autocomplete handler for /modsearch
pub async fn autocomplete(
    ctx: &Context,
    inter: &AutocompleteInteraction,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if inter.data.name == "modsearch" {
        // Determine version alias in autocomplete context
        let version_alias = inter.data.options.iter()
            .find(|o| o.name == "version")
            .and_then(|o| o.value.as_ref())
            .and_then(|v| v.as_str())
            .unwrap_or("public");
        let code = resolve_version(version_alias).unwrap_or("12104");

        if let Some(opt) = inter.data.options.iter().find(|o| o.name == "query") {
            if let Some(prefix) = opt.value.as_ref().and_then(|v| v.as_str()) {
                let norm = prefix.to_lowercase();
                let url = format!("https://skatebit-api.vercel.app/api/mods/{}", code);
                let mods: Vec<ModEntry> = Client::new()
                    .get(&url)
                    .send().await?
                    .json().await?;
                inter.create_autocomplete_response(&ctx.http, |resp| {
                    let mut r = resp;
                    for m in mods.into_iter()
                        .filter(|m| m.title.to_lowercase().contains(&norm))
                        .take(25)
                    {
                        r = r.add_string_choice(m.title.clone(), m.title.clone());
                    }
                    r
                }).await?;
            }
        }
    }
    Ok(())
}