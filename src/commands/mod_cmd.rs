use crate::{
    types::{Context, Error, ModVersionBranch, ModEntry, BOT_EMBED_COLOR},
    mod_utils,
};
use poise::{
    serenity_prelude::{self as serenity, CreateEmbedFooter},
    CreateReply,
};
use std::str::FromStr;
use tracing::{info, warn, error};

async fn mod_title_autocomplete(
    ctx: Context<'_>,
    partial: &str,
) -> Vec<String> {
    let mod_cache_guard = ctx.data().mod_cache.read().await;
    let mut suggestions = Vec::new();
    let partial_lowercase = partial.to_lowercase();

    for (slug, mods) in mod_cache_guard.iter() {
        // Determine branch enum variant from slug
        let version_enum = match slug.as_str() {
            "1228" => ModVersionBranch::Alpha,
            "12104" => ModVersionBranch::BetaPublic,
            _ => continue,
        };
        let branch_name = version_enum.to_string();

        for entry in mods {
            if entry.title.to_lowercase().contains(&partial_lowercase) {
                let display_title = if entry.title.len() > 75 {
                    format!("{}...", &entry.title[..72])
                } else {
                    entry.title.clone()
                };
                let suggestion = format!("{} - {}", display_title, branch_name);
                suggestions.push(suggestion[..suggestion.len().min(100)].to_string());
                if suggestions.len() >= 25 { break; }
            }
        }
        if suggestions.len() >= 25 { break; }
    }
    suggestions
}

/// Searches the working mod lists for a specific mod.
#[poise::command(slash_command, prefix_command, rename = "mod")]
pub async fn modsearch( 
    ctx: Context<'_>,
    #[description = "Mod Title (use autocomplete for version)"]
    #[autocomplete = "mod_title_autocomplete"]
    search: String,
) -> Result<(), Error> {
    info!(user = %ctx.author().name, search_term = %search, "Mod command received");

    let (target_title, target_branch_name) =
        if let Some(separator_index) = search.rfind(" - ") {
            let (title_part, branch_part) = search.split_at(separator_index);
            (title_part.trim(), branch_part[3..].trim())
        } else {
             warn!(user = %ctx.author().name, search_term = %search, "Search term missing ' - Branch' suffix");
             let reply = CreateReply::default()
                 .content("Please use autocomplete or format search as `Mod Title - Branch` (e.g., `SomeMod - Beta/Public`).")
                 .ephemeral(true);
             ctx.send(reply).await?;
             return Ok(());
        };

    let version_enum = match ModVersionBranch::from_str(target_branch_name) {
         Ok(v) => v,
         Err(_) => {
             warn!(user = %ctx.author().name, search_term = %search, parsed_branch = %target_branch_name, "Invalid branch name parsed from search query");
             let reply = CreateReply::default()
                 .content(format!("Invalid branch name '{}' found. Use autocomplete or 'Alpha', 'Beta', 'Public', 'Beta/Public'.", target_branch_name))
                 .ephemeral(true);
             ctx.send(reply).await?;
             return Ok(());
         }
    };

    let version_slug = mod_utils::resolve_version_slug(version_enum);

    let mod_cache_guard = ctx.data().mod_cache.read().await;

    let reply = if let Some(mods) = mod_cache_guard.get(version_slug) {
         info!(fetched_count = mods.len(), %target_title, version = %version_enum, "Using cached mods, proceeding to filter.");
        let matches: Vec<&ModEntry> = mods.iter().filter(|m| m.title.eq_ignore_ascii_case(target_title)).collect();
         info!(matched_count = matches.len(), %target_title, version = %version_enum, "Filtering complete.");
         if matches.is_empty() && !mods.is_empty() { warn!(query=%target_title, version=%version_enum, "Filter yielded no results"); }

        match matches.len() {
            0 => {
                warn!(query=%target_title, version = %version_enum, "Mod not found in cache");
                CreateReply::default()
                    .content(format!("❌ No mod found matching '{}' for version {}.", target_title, version_enum))
                    .ephemeral(true)
            }
            1 => {
                let entry = matches[0];
                info!(mod_title = %entry.title, version = %version_enum, "Found single mod match");
                let description = mod_utils::format_mod_entry(entry);
                let embed = serenity::CreateEmbed::default()
                    .title(&entry.title)
                    .description(description)
                    .color(BOT_EMBED_COLOR)
                    .footer(CreateEmbedFooter::new(format!("Version: {} | Requested by {}", version_enum, ctx.author().name))) // Use enum Display
                    .timestamp(serenity::Timestamp::now());
                CreateReply::default().embed(embed)
             }
            _ => {
                info!(count = matches.len(), query=%target_title, version = %version_enum, "Multiple exact matches found?");
                let list_limit = 5;
                let description = format!(
                    "Found {} mods matching '{}' exactly (unusual). Please check.\n\n**Matches:**\n{}",
                    matches.len(), target_title,
                    matches.iter().take(list_limit).map(|m| format!("- {}", m.title)).collect::<Vec<_>>().join("\n")
                );
                let embed = serenity::CreateEmbed::default()
                    .title("Multiple Exact Matches Found?")
                    .description(description)
                    .color(BOT_EMBED_COLOR)
                    .footer(CreateEmbedFooter::new(format!("Version: {} | Requested by {}", version_enum, ctx.author().name))) // Use enum Display
                    .timestamp(serenity::Timestamp::now());
                CreateReply::default().embed(embed)
             }
        }
    } else {
        error!(%version_slug, "Mod cache missing for required version slug!");
        CreateReply::default()
            .content(format!("Sorry, mod data for version {} is currently unavailable.", version_enum)) // Use enum Display
            .ephemeral(true)
    };

    ctx.send(reply).await?;

    Ok(())
}