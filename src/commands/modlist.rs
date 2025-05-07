use crate::utils::mod_format::format_mod_entry;
use crate::utils::mod_fetch::{resolve_version, fetch_mods};

use serenity::builder::CreateApplicationCommand;
use serenity::model::{
    application::interaction::{
        application_command::ApplicationCommandInteraction,
        message_component::MessageComponentInteraction,
        InteractionResponseType,
        MessageFlags,
    },
};
use serenity::client::Context;
use tokio::time::Duration;

const MAX_MODS_PER_EMBED: usize = 15;
const EMBED_COLOR: u32 = 0x1eaeef;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("modlist")
        .description("Sends you a DM to select and receive a mod list.")
}

pub async fn run(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    interaction
        .create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| {
                    msg.flags(MessageFlags::EPHEMERAL)
                        .content("📬 I've sent you a DM where you can select the mod list version!")
                })
        })
        .await?;

    let dm_channel_result = interaction.user.create_dm_channel(&ctx.http).await;
    let dm_channel = match dm_channel_result {
        Ok(channel) => channel,
        Err(e) => {
            eprintln!("Failed to create DM channel for {}: {:?}", interaction.user.id, e);
            interaction.edit_original_interaction_response(&ctx.http, |msg| {
                msg.content("⚠️ I couldn't send you a DM. Please ensure your DMs are open for this server.")
                   .components(|c| c)
            }).await?;
            return Ok(());
        }
    };

    dm_channel
        .send_message(&ctx.http, |msg| {
            msg.content("Please select the mod list version you'd like to view:")
                .components(|comp_builder| {
                    comp_builder.create_action_row(|action_row| {
                        action_row.create_select_menu(|select_menu| {
                            select_menu
                                .custom_id("modlist_version_select")
                                .placeholder("Choose a version...")
                                .options(|opt_builder| {
                                    opt_builder.create_option(|opt| {
                                        opt.label("Alpha Version")
                                            .value("alpha")
                                            .description("View the Alpha mod list (~23 mods)")
                                    });
                                    opt_builder.create_option(|opt| {
                                        opt.label("Beta Version")
                                            .value("beta")
                                            .description("View the Beta mod list (~13 mods)")
                                    });
                                    opt_builder.create_option(|opt| {
                                        opt.label("Public Version")
                                            .value("public")
                                            .description("View the Public mod list (~13 mods)")
                                    })
                                })
                        })
                    })
                })
        })
        .await?;
    Ok(())
}

pub async fn handle_version_selection_and_send_list(
    ctx: &Context,
    component_interaction: &MessageComponentInteraction,
    selected_version_value: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut original_message = component_interaction.message.clone();
    original_message.edit(&ctx.http, |edit_msg| {
        edit_msg.content(format!("⏳ Fetching the {} mod list for you...", selected_version_value))
                .components(|c| c)
    }).await?;

    let version_code = resolve_version(selected_version_value)
        .ok_or_else(|| format!("Internal error: Invalid version value received: {}", selected_version_value))?;

    let list_label = match selected_version_value {
        "alpha" => "Alpha Version",
        "beta" => "Beta Version",
        "public" => "Public Version",
        _ => "Selected Version",
    };

    let mods = fetch_mods(version_code).await?;
    let dm_channel_id = component_interaction.channel_id;

    if mods.is_empty() {
        dm_channel_id
            .say(&ctx.http, format!("ℹ️ No mods found for the {} list.", list_label))
            .await?;
        return Ok(());
    }

    let total_mods = mods.len();
    let mut embed_count = 0;
    let num_pages = (total_mods as f32 / MAX_MODS_PER_EMBED as f32).ceil() as usize;

    for (chunk_index, mod_chunk) in mods.chunks(MAX_MODS_PER_EMBED).enumerate() {
        embed_count += 1;
        let mut description = String::new();
        for (item_index, mod_entry) in mod_chunk.iter().enumerate() {
            description.push_str(&format_mod_entry(mod_entry));
            if item_index < mod_chunk.len() - 1 {
                description.push_str("\n\n---\n\n");
            }
        }

        if description.is_empty() { continue; }
        if description.len() > 4096 {
            eprintln!("Warning: Embed description for a chunk is too long. Truncating.");
            description.truncate(4090);
            description.push_str("...");
        }

        let title = format!("{} Mod List (Page {} of {})", list_label, embed_count, num_pages);

        dm_channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(&title)
                 .description(&description)
                 .color(EMBED_COLOR)
                 .footer(|f| f.text(format!("{} mods in this part | Total: {}", mod_chunk.len(), total_mods)))
            })
        }).await?;

        if chunk_index < (num_pages - 1) {
            tokio::time::sleep(Duration::from_millis(600)).await;
        }
    }

    dm_channel_id.say(&ctx.http, format!("☑️ End of {} mod list.", list_label)).await?;
    Ok(())
}