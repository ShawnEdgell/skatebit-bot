// src/lib.rs
pub mod commands;
pub mod types;
pub mod map_cache;
pub mod mod_utils;
pub mod scheduler;

use poise::serenity_prelude as serenity;
use serenity::model::id::GuildId; // Keep for iterating guild IDs
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

                // --- Cache Loading (remains the same) ---
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
                // --- End Cache Loading ---

                // --- Comprehensive Command Cleanup and Registration ---

                // 1. Clear ALL old global application commands
                info!("Attempting to clear ALL old global application commands...");
                if let Err(e) = poise::builtins::register_globally(ctx, &[] as &[poise::Command<Data, AppError>]).await {
                    warn!(error = %e, "Failed to clear all global commands. This might be okay if it's the first run or due to permissions/rate limits.");
                } else {
                    info!("Successfully cleared all old global commands.");
                }

                // 2. Clear old guild-specific commands from all guilds the bot is in
                info!("Attempting to clear old guild-specific commands from all servers...");
                let guild_ids: Vec<GuildId> = ready.guilds.iter().map(|g| g.id).collect();
                if guild_ids.is_empty() {
                    info!("Bot is not in any guilds, skipping guild command cleanup.");
                } else {
                    for guild_id in guild_ids {
                        info!(guild_id = %guild_id, "Attempting to clear commands for guild...");
                        if let Err(e) = poise::builtins::register_in_guild(ctx, &[] as &[poise::Command<Data, AppError>], guild_id).await {
                            warn!(error = %e, guild_id = %guild_id, "Failed to clear commands for guild. Bot might lack 'applications.commands' scope in this guild or hit rate limits.");
                        } else {
                            info!(guild_id = %guild_id, "Successfully cleared commands for guild.");
                        }
                    }
                    info!("Finished attempting to clear guild-specific commands.");
                }

                // 3. Register the current set of commands globally
                info!("Registering current application commands globally...");
                poise::builtins::register_globally(ctx, &framework_ref.options().commands).await
                    .map_err(|e| { error!(error = %e, "Failed global command registration"); e })
                    .context("Failed to register current commands globally")?;
                info!("Successfully registered current application commands globally.");
                // --- End Command Cleanup and Registration ---

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
