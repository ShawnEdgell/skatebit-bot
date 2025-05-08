use crate::{
    types::ModEntry,
    utils::{
        mod_format::format_mod_entry,
        mod_fetch::{resolve_version, fetch_mods},
        interaction::get_str_option,
        autocomplete::basic_autocomplete,
        constants::BOT_EMBED_COLOR,
    },
};

use serenity::{
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed,
        CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    model::{
        application::{CommandInteraction, CommandOptionType, CommandDataOptionValue},
    },
    client::Context,
};
use std::error::Error;

pub fn register() -> CreateCommand {
    CreateCommand::new("modsearch")
        .description("Search for a mod by title within a specific version branch")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "version",
                "Which version: alpha, beta, public",
            )
            .add_string_choice("Alpha", "alpha")
            .add_string_choice("Beta", "beta")
            .add_string_choice("Public", "public")
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "query",
                "Keyword to search in mod titles",
            )
            .set_autocomplete(true)
            .required(true),
        )
}

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let version_alias = get_str_option(interaction, "version").ok_or("Missing version option")?;
    let code = resolve_version(version_alias).ok_or_else(|| format!("Invalid version alias: {}", version_alias))?;
    let query = get_str_option(interaction, "query")
        .unwrap_or_default()
        .to_lowercase();

    let mods: Vec<ModEntry> = fetch_mods(code).await?;

    let response_message: CreateInteractionResponseMessage;

    if let Some(entry) = mods.into_iter().find(|m| m.title.to_lowercase().contains(&query)) {
        let desc = format_mod_entry(&entry);
        let embed = CreateEmbed::new()
            .title(&entry.title)
            .description(&desc)
            .color(BOT_EMBED_COLOR);
        response_message = CreateInteractionResponseMessage::new().add_embed(embed);
    } else {
        response_message = CreateInteractionResponseMessage::new()
            .content("No matching mod found in the specified version.")
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
    let version_alias_opt = interaction.data.options.iter().find_map(|opt| {
        if opt.name == "version" {
            if let CommandDataOptionValue::String(s_val) = &opt.value {
                Some(s_val.as_str())
            } else { None }
        } else { None }
    });

    let version_alias = version_alias_opt.unwrap_or("public");
    let code = resolve_version(version_alias).unwrap_or("12104");

    basic_autocomplete(ctx, interaction, "query", move |current_query_input| {
        let query_owned = current_query_input.to_string().to_lowercase();
        async move {
            match fetch_mods(code).await {
                Ok(mods_data) => Ok(mods_data
                    .into_iter()
                    .filter(|m| m.title.to_lowercase().contains(&query_owned))
                    .map(|m| (m.title.clone(), m.title.clone()))
                    .take(25)
                    .collect()),
                Err(e) => {
                    eprintln!("Error fetching mods for autocomplete: {:?}", e);
                    Ok(Vec::new())
                }
            }
        }
    })
    .await
}