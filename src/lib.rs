pub mod commands;
pub mod types;
pub mod map_cache;
pub mod mod_utils;
pub mod scheduler;

use poise::serenity_prelude as serenity;
// GuildId is no longer needed here for command registration
use std::{collections::HashMap, env, sync::Arc};
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

    // GUILD_ID is no longer strictly needed for global command registration
    // If you use it for other purposes, you can keep its loading logic.
    // For now, I'm commenting it out as it's not used in this file anymore.
    /*
    let guild_id_str = env::var("GUILD_ID")
        .map_err(|e| { error!("Missing GUILD_ID: {}", e); e })
        .context("Missing GUILD_ID in the environment")?;
    let guild_id_val = guild_id_str.parse::<u64>()
        .map_err(|e| { error!("GUILD_ID not a valid u64: '{}', {}", guild_id_str, e); e })
        .with_context(|| format!("GUILD_ID is not a valid u64 integer: '{}'", guild_id_str))?;
    let _guild_id = serenity::model::id::GuildId::new(guild_id_val); // Marked as unused if only for registration
    */

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let app_data = Arc::new(Data::new());
    let app_data_for_scheduler = app_data.clone();

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
        .setup(move |ctx, ready, framework_ref| {
            let data_for_setup = app_data.clone();
            Box::pin(async move {
                info!("Logged in as {}! (User ID: {})", ready.user.name, ready.user.id);

                // Initial cache loading (remains the same)
                match map_cache::load_maps_from_remote(&data_for_setup.http_client).await {
                    Ok(maps) => {
                        let map_count = maps.len();
                        *data_for_setup.map_cache.write().await = maps;
                        info!("Initial map cache populated with {} maps.", map_count);
                    }
                    Err(e) => { error!(error = %e, "Failed initial map cache load during setup."); }
                }

                info!("Populating initial mod cache for defined versions...");
                let slugs_to_fetch = ["1228", "12104"];
                let mut mod_cache_map = HashMap::new();
                let mut total_mods_loaded = 0;
                let mut versions_loaded_count = 0;
                for slug in slugs_to_fetch {
                    match mod_utils::fetch_mods_for_version(&data_for_setup.http_client, slug).await {
                        Ok(mods) => {
                            info!(count = mods.len(), slug = slug, "Successfully fetched mods for slug.");
                            total_mods_loaded += mods.len();
                            if !mods.is_empty() { versions_loaded_count += 1; }
                            mod_cache_map.insert(slug.to_string(), mods);
                        }
                        Err(e) => { error!(error = %e, slug = slug, "Failed initial mod fetch for slug during setup."); }
                    }
                }
                *data_for_setup.mod_cache.write().await = mod_cache_map;
                info!(
                    total_mods = total_mods_loaded,
                    versions_attempted = slugs_to_fetch.len(),
                    versions_loaded = versions_loaded_count,
                    "Initial mod cache population complete."
                );

                // Register commands globally
                // This will overwrite any previous global command set with the current one.
                info!("Registering application commands globally...");
                poise::builtins::register_globally(ctx, &framework_ref.options().commands).await
                    .map_err(|e| { error!(error = %e, "Failed global command registration"); e })
                    .context("Failed to register commands globally")?;
                info!("Successfully registered application commands globally.");

                Ok((*data_for_setup).clone())
            })
        })
        .build();

    scheduler::initialize_and_start_scheduler(app_data_for_scheduler).await?;

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
