use crate::utils::mod_format::format_mod_entry;
use crate::utils::mod_fetch::{resolve_version, fetch_mods};
use crate::utils::interaction::get_str_option;

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
use std::error::Error;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("modsearch")
        .description("Search for a mod by title within a specific version branch")
        .create_option(|opt| {
            opt.name("version")
                .description("Which version: alpha, beta, public")
                .kind(CommandOptionType::String)
                .add_string_choice("alpha", "alpha")
                .add_string_choice("beta", "beta")
                .add_string_choice("public", "public")
                .required(true)
        })
        .create_option(|opt| {
            opt.name("query")
                .description("Keyword to search in mod titles")
                .kind(CommandOptionType::String)
                .set_autocomplete(true)
                .required(true)
        })
}

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let version_alias = get_str_option(command, "version").ok_or("Missing version")?;
    let code = resolve_version(version_alias).ok_or("Invalid version alias")?;

    let query = get_str_option(command, "query")
        .unwrap_or_default()
        .to_lowercase();

    let mods = fetch_mods(code).await?;

    if let Some(entry) = mods.into_iter()
        .find(|m| m.title.to_lowercase().contains(&query))
    {
        let desc = format_mod_entry(&entry);
        command.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| {
                    data.embed(|e| e.title(&entry.title).description(&desc))
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

pub async fn autocomplete(
    ctx: &Context,
    inter: &AutocompleteInteraction,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if inter.data.name != "modsearch" {
        return Ok(());
    }

    let version_alias = inter.data.options.iter()
        .find(|o| o.name == "version")
        .and_then(|o| o.value.as_ref())
        .and_then(|v| v.as_str())
        .unwrap_or("public");

    let code = resolve_version(version_alias).unwrap_or("12104");

    if let Some(opt) = inter.data.options.iter().find(|o| o.name == "query") {
        if let Some(prefix) = opt.value.as_ref().and_then(|v| v.as_str()) {
            let norm = prefix.to_lowercase();
            let mods = fetch_mods(code).await?;

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

    Ok(())
}
