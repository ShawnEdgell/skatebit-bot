use crate::utils::mod_format::format_mod_entry;
use crate::utils::mod_fetch::{resolve_version, fetch_mods};
use crate::utils::components::pagination_buttons;
use crate::utils::interaction::{get_str_option, get_bool_option};

use serenity::builder::CreateApplicationCommand;
use serenity::model::{
    application::command::CommandOptionType,
    application::interaction::{
        application_command::ApplicationCommandInteraction,
        message_component::MessageComponentInteraction,
        InteractionResponseType,
        MessageFlags,
    },
    prelude::{MessageId, UserId},
};
use serenity::client::Context;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    static ref PAGINATED_DATA: Arc<Mutex<HashMap<(UserId, MessageId), (Vec<(String, String)>, usize)>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("modlist")
        .description("Fetch mod list by alias (alpha, beta, or public)")
        .create_option(|opt| {
            opt.name("version")
                .description("Which version: alpha, beta, or public")
                .kind(CommandOptionType::String)
                .required(true)
                .add_string_choice("alpha", "alpha")
                .add_string_choice("beta", "beta")
                .add_string_choice("public", "public")
        })
        .create_option(|opt| {
            opt.name("dm")
                .description("Send the full list via DM instead of ephemeral chat")
                .kind(CommandOptionType::Boolean)
                .required(false)
        })
}

pub async fn run(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input = get_str_option(interaction, "version").ok_or("Missing version")?;
    let version_code = resolve_version(input).ok_or("Invalid version alias")?;
    let label = match input {
        "alpha" => "Alpha",
        "beta" => "Beta",
        "public" => "Public",
        _ => "Mods",
    };
    let mods = fetch_mods(version_code).await?;
    let dm_flag = get_bool_option(interaction, "dm").unwrap_or(false);

    let chunk_size = 4;
    let mut pages = Vec::new();
    for (i, chunk) in mods.chunks(chunk_size).enumerate() {
        let desc = chunk.iter().map(format_mod_entry).collect::<Vec<_>>().join("\n\n");
        let title = if i == 0 {
            format!("{} Mod List", label)
        } else {
            format!("{} Mod List (Page {})", label, i + 1)
        };
        pages.push((title, desc));
    }

    let (title, desc) = &pages[0];

    if dm_flag {
        interaction.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| {
                    msg.flags(MessageFlags::EPHEMERAL)
                        .content("✅ Sending mod list via DM!")
                })
        }).await?;

        let dm = interaction.user.create_dm_channel(&ctx.http).await?;
        let sent = dm.send_message(&ctx.http, |m| {
            m.embed(|e| e.title(title).description(desc))
             .components(|c| c.create_action_row(pagination_buttons))
        }).await?;

        PAGINATED_DATA.lock().await.insert((interaction.user.id, sent.id), (pages, 0));
    } else {
        interaction.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| {
                    msg.flags(MessageFlags::EPHEMERAL)
                        .embed(|e| e.title(title).description(desc))
                        .components(|c| c.create_action_row(pagination_buttons))
                })
        }).await?;

        let msg = interaction.get_interaction_response(&ctx.http).await?;
        PAGINATED_DATA.lock().await.insert((interaction.user.id, msg.id), (pages, 0));
    }

    Ok(())
}

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
