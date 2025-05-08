use crate::{
    types::ModEntry,
    utils::{
        mod_format::format_mod_entry,
        mod_fetch::{resolve_version, fetch_mods},
        constants::BOT_EMBED_COLOR,
    },
};

use serenity::{
    all::*,
    client::Context,
};
use std::error::Error;
use tokio::time::Duration;

const MAX_MODS_PER_EMBED: usize = 15;

pub fn register() -> CreateCommand {
    CreateCommand::new("modlist")
        .description("Sends you a DM to select and receive a mod list.")
}

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let initial_response_msg = CreateInteractionResponseMessage::new()
        .content("📬 I'm attempting to send you a DM where you can select the mod list version!")
        .ephemeral(true);
    let initial_response = CreateInteractionResponse::Message(initial_response_msg);
    interaction.create_response(&ctx.http, initial_response).await?;

    let dm_channel = match interaction.user.create_dm_channel(&ctx.http).await {
        Ok(channel) => channel,
        Err(e) => {
            eprintln!("Failed to create DM channel for {}: {:?}", interaction.user.id, e);
            let edit_msg = EditInteractionResponse::new()
                .content("⚠️ I couldn't send you a DM. Please ensure your DMs are open for this server.")
                .components(vec![]);
            interaction.edit_response(&ctx.http, edit_msg).await?;
            return Ok(());
        }
    };

    let options = vec![
        CreateSelectMenuOption::new("Alpha Version", "alpha")
            .description("View the Alpha mod list (~23 mods)"),
        CreateSelectMenuOption::new("Beta Version", "beta")
            .description("View the Beta mod list (~13 mods)"),
        CreateSelectMenuOption::new("Public Version", "public")
            .description("View the Public mod list (~13 mods)"),
    ];
    let select_menu = CreateSelectMenu::new(
        "modlist_version_select",
        CreateSelectMenuKind::String { options },
    )
    .placeholder("Choose a version...");
    let action_row = CreateActionRow::SelectMenu(select_menu);

    let components_vec = vec![action_row];

    let dm_message_builder = CreateMessage::new()
        .content("Please select the mod list version you'd like to view:")
        .components(components_vec);

    dm_channel.send_message(&ctx.http, dm_message_builder).await?;
    Ok(())
}

pub async fn handle_version_selection_and_send_list(
    ctx: &Context,
    component_interaction: &ComponentInteraction,
    selected_version_value: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {

    let mut message_to_edit = component_interaction.message.clone();
    message_to_edit.edit(&ctx.http,
        EditMessage::new()
            .content(format!("⏳ Fetching the {} mod list for you...", selected_version_value))
            .components(vec![])
    ).await?;

    let version_code = resolve_version(selected_version_value)
        .ok_or_else(|| format!("Internal error: Invalid version value received: {}", selected_version_value))?;

    let list_label = match selected_version_value {
        "alpha" => "Alpha Version",
        "beta" => "Beta Version",
        "public" => "Public Version",
        _ => "Selected Version",
    };

    let mods: Vec<ModEntry> = fetch_mods(version_code).await?;
    let dm_channel_id: ChannelId = component_interaction.channel_id;

    if mods.is_empty() {
        dm_channel_id
            .send_message(&ctx.http, CreateMessage::new().content(format!(
                "ℹ️ No mods found for the {} list.",
                list_label
            )))
            .await?;
        return Ok(());
    }

    let total_mods = mods.len();
    let mut embed_count = 0;
    let num_pages = (total_mods as f32 / MAX_MODS_PER_EMBED as f32).ceil() as usize;

    for (chunk_index, mod_chunk) in mods.chunks(MAX_MODS_PER_EMBED).enumerate() {
        embed_count += 1;
        let mut description_content = String::new();
        for (item_index, mod_entry) in mod_chunk.iter().enumerate() {
            description_content.push_str(&format_mod_entry(mod_entry));
            if item_index < mod_chunk.len() - 1 {
                description_content.push_str("\n\n---\n\n");
            }
        }

        if description_content.is_empty() { continue; }
        if description_content.len() > 4096 {
            eprintln!("Warning: Embed description for a chunk is too long. Truncating.");
            description_content.truncate(4090);
            description_content.push_str("...");
        }

        let title = format!("{} Mod List (Page {} of {})", list_label, embed_count, num_pages);

        let embed = CreateEmbed::new()
            .title(&title)
            .description(&description_content)
            .color(BOT_EMBED_COLOR)
            .footer(CreateEmbedFooter::new(format!(
                "{} mods in this part | Total: {}",
                mod_chunk.len(),
                total_mods
            )));

        dm_channel_id.send_message(&ctx.http, CreateMessage::new().add_embed(embed)).await?;

        if chunk_index < (num_pages - 1) {
            tokio::time::sleep(Duration::from_millis(600)).await;
        }
    }

    dm_channel_id
        .send_message(&ctx.http, CreateMessage::new().content(format!(
            "☑️ End of {} mod list.",
            list_label
        )))
        .await?;
    Ok(())
}