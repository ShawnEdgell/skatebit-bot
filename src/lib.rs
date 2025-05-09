pub mod commands;
pub mod types;
pub mod map_cache;
pub mod mod_utils;

use poise::serenity_prelude as serenity;
use serenity::model::id::GuildId;
use std::{collections::HashMap, env};
use dotenvy::dotenv;
use types::{Data, Error as AppError};
use anyhow::{Context as AnyhowContext, Result as AnyhowResult};
use tracing::{info, warn, error};

pub async fn run() -> AnyhowResult<()> {
    match dotenv() {
        Ok(_) => info!(".env file loaded successfully."),
        Err(e) => {
            warn!("Failed to load .env file: {}. Will try using environment variables directly.", e);
        }
    }

    let token = env::var("DISCORD_TOKEN")
        .map_err(|e| { error!("Missing DISCORD_TOKEN: {}", e); e })
        .context("Missing DISCORD_TOKEN in the environment")?;

    let guild_id_str = env::var("GUILD_ID")
        .map_err(|e| { error!("Missing GUILD_ID: {}", e); e })
        .context("Missing GUILD_ID in the environment")?;

    let guild_id_val = guild_id_str.parse::<u64>()
        .map_err(|e| { error!("GUILD_ID not a valid u64: '{}', {}", guild_id_str, e); e })
        .with_context(|| format!("GUILD_ID is not a valid u64 integer: '{}'", guild_id_str))?;
    let guild_id = GuildId::new(guild_id_val);

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let app_data = Data::new();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::ping_cmd::ping(),
                commands::age_cmd::age(),
                commands::map_cmd::map(),
                commands::modlist_cmd::modlist(),
                commands::mod_cmd::modsearch(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let data_for_setup = app_data.clone();
            Box::pin(async move {
                info!("Logged in as {}! (User ID: {})", ready.user.name, ready.user.id);

                info!("Attempting to clear old global application commands...");
                poise::builtins::register_globally(ctx, &[] as &[poise::Command<Data, AppError>]).await
                    .map_err(|e| { warn!(error = %e, "Failed attempt to clear global commands"); e })
                    .context("Failed to clear global commands")
                    .ok();

                let maps = map_cache::load_maps_from_remote(&data_for_setup.http_client).await
                     .context("Failed to populate map cache during setup")?;
                *data_for_setup.map_cache.write().await = maps;
                info!("Map cache populated with {} maps.", data_for_setup.map_cache.read().await.len());


                info!("Populating mod cache for defined versions...");
                let slugs_to_fetch = ["1228", "12104"];
                let mut mod_cache_map = HashMap::new();
                let mut total_mods_loaded = 0;
                let mut versions_loaded_count = 0;

                for slug in slugs_to_fetch {
                    let mods = mod_utils::fetch_mods_for_version(&data_for_setup.http_client, slug).await
                       .with_context(|| format!("Failed to fetch mods for slug '{}' during setup", slug))?;

                    info!(count = mods.len(), slug = slug, "Successfully fetched mods for slug.");
                    total_mods_loaded += mods.len();
                    if !mods.is_empty() { versions_loaded_count += 1; }
                    mod_cache_map.insert(slug.to_string(), mods);
                }
                *data_for_setup.mod_cache.write().await = mod_cache_map;
                info!(
                    total_mods = total_mods_loaded,
                    versions_attempted = slugs_to_fetch.len(),
                    versions_loaded = versions_loaded_count,
                    "Mod cache population complete."
                );

                info!(guild_id = %guild_id, "Registering commands in guild...");
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id).await
                    .map_err(|e| { error!(guild_id = %guild_id, error = %e, "Failed guild command registration"); e })
                    .context("Failed to register commands in guild")?;
                info!(guild_id = %guild_id, "Successfully registered commands in guild.");

                Ok(data_for_setup)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .map_err(|e| { error!(error = %e, "Fatal error creating Discord client"); e })
        .context("Fatal error creating Discord client")?;

    info!("Starting bot connection to Discord...");
    client.start_autosharded().await
        .map_err(|e| { error!(error = %e, "Client runtime error"); e })
        .context("Discord client stopped unexpectedly")?;

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, AppError>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            error!(error = ?error, "Framework setup error");
        },
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!(error = ?error, command = %ctx.command().name, "Error executing command");
             let _ = ctx.say("Oops, an internal error occurred!").await;
        },
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                 error!(error = ?e, "Error occurred while handling another error");
            }
        }
    }
}