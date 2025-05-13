// src/lib.rs
pub mod commands;
pub mod types;
pub mod map_cache;
pub mod mod_utils;
pub mod scheduler;

use poise::serenity_prelude as serenity;
use std::{collections::HashMap, env, sync::Arc, time::Duration}; // Added std::time::Duration
use dotenvy::dotenv;
use types::{Data, Error as AppError};
use anyhow::{Context as AnyhowContext, Result as AnyhowResult};
use tracing::{info, warn, error, instrument};

pub async fn run() -> AnyhowResult<()> {
    match dotenv() {
        Ok(_) => info!(".env file loaded successfully for bot."),
        Err(e) => {
            warn!("Failed to load .env file for bot: {}. Relying on system environment variables.", e);
        }
    }

    let token = env::var("DISCORD_TOKEN")
        .context("DISCORD_TOKEN environment variable not set")?;

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
            let commands_to_register: &[poise::Command<Data, AppError>] = &framework_ref.options().commands;
            
            Box::pin(async move {
                info!("Bot logged in as {} (User ID: {})", ready.user.name, ready.user.id);
                info!("Connected to {} guilds.", ready.guilds.len());

                // --- Initial Map Cache Loading (from new Go API) WITH RETRY ---
                let go_api_base_url = env::var("GO_MODIO_API_BASE_URL")
                    .unwrap_or_else(|_| {
                        warn!("GO_MODIO_API_BASE_URL not set, defaulting to live https://api.skatebit.app");
                        "https://api.skatebit.app".to_string()
                    });
                let maps_api_endpoint = format!("{}/api/v1/skaterxl/maps", go_api_base_url);

                info!("Initial Setup: Attempting to fetch map cache from self-hosted Go API ({})", maps_api_endpoint);
                
                let mut initial_maps_loaded_successfully = false;
                for attempt in 1..=3 { // Try up to 3 times
                    match map_cache::load_maps_from_go_api(&data_for_setup.http_client, &maps_api_endpoint).await {
                        Ok(maps_from_api) => {
                            if !maps_from_api.is_empty() {
                                let map_count = maps_from_api.len();
                                *data_for_setup.map_cache.write().await = maps_from_api;
                                info!(attempt, map_count, "Initial Setup: Map cache populated from Go API.");
                                initial_maps_loaded_successfully = true;
                                break; // Success, exit retry loop
                            } else {
                                warn!(attempt, "Initial Setup: Go API returned empty map list. API might still be initializing. Retrying soon...");
                            }
                        }
                        Err(e) => { 
                            error!(error = ?e, attempt, "Initial Setup: Failed map cache load from Go API.");
                            // Decide if you want to retry on error or just log. For now, we'll retry.
                        }
                    }
                    if attempt < 3 { // Don't sleep after the last attempt
                        let sleep_duration = Duration::from_secs(5 * attempt as u64); // Increase delay: 5s, 10s
                        info!("Initial Setup: Waiting {:?} before retrying map fetch.", sleep_duration);
                        tokio::time::sleep(sleep_duration).await;
                    }
                }
                if !initial_maps_loaded_successfully {
                    error!("Initial Setup: FAILED to load maps from Go API after multiple attempts. Bot may have stale/no map data for this session.");
                }
                // --- End Initial Map Cache Loading ---

                // --- Initial Slug-based Mod Cache Loading (UNCHANGED) ---
                info!("Initial Setup: Populating slug-based mod cache...");
                // ... (your existing slug-based mod cache logic remains here) ...
                let slugs_to_fetch = ["1228", "12104"];
                let mut mod_cache_map_for_setup = HashMap::new();
                let mut total_mods_loaded_setup = 0;
                let mut versions_loaded_count_setup = 0;

                for slug in slugs_to_fetch {
                    match mod_utils::fetch_mods_for_version(&data_for_setup.http_client, slug).await {
                        Ok(mods) => {
                            info!(count = mods.len(), slug, "Initial Setup: Fetched slug-based mods for slug.");
                            total_mods_loaded_setup += mods.len();
                            if !mods.is_empty() { versions_loaded_count_setup += 1; }
                            mod_cache_map_for_setup.insert(slug.to_string(), mods);
                        }
                        Err(e) => { 
                            error!(error = ?e, slug, "Initial Setup: Failed slug-based mod fetch for slug.");
                        }
                    }
                }
                if total_mods_loaded_setup > 0 || versions_loaded_count_setup > 0 {
                    *data_for_setup.mod_cache.write().await = mod_cache_map_for_setup;
                    info!( total_mods = total_mods_loaded_setup, versions_attempted = slugs_to_fetch.len(), versions_loaded = versions_loaded_count_setup, "Initial Setup: Slug-based mod cache population complete.");
                } else {
                    warn!("Initial Setup: No slug-based mods loaded, mod cache might be empty or fetch failed.");
                }
                // --- End Initial Slug-based Mod Cache Loading ---
                
                // --- THOROUGH COMMAND CLEANUP AND REGISTRATION ---
                info!("Starting thorough command cleanup and registration...");
                // ... (your existing command registration logic from the "skatebit_bot_lib_rs_command_cleanup" artifact) ...
                // 1. Clear commands from all guilds
                if !ready.guilds.is_empty() {
                    info!("Attempting to clear old guild-specific commands from all {} servers...", ready.guilds.len());
                    for guild_status in &ready.guilds {
                        let guild_id = guild_status.id;
                        info!(guild_id = %guild_id, "Attempting to clear commands for guild...");
                        if let Err(e) = poise::builtins::register_in_guild(ctx, &[] as &[poise::Command<Data, AppError>], guild_id).await {
                            warn!(error = %e, guild_id = %guild_id, "Failed to clear commands for guild.");
                        } else {
                            info!(guild_id = %guild_id, "Successfully cleared commands for guild.");
                        }
                    }
                    info!("Finished attempting to clear guild-specific commands.");
                } else {
                    info!("Bot is not in any guilds, skipping guild command cleanup.");
                }

                // 2. Clear ALL old global application commands
                info!("Attempting to clear ALL old global application commands...");
                if let Err(e) = poise::builtins::register_globally(ctx, &[] as &[poise::Command<Data, AppError>]).await {
                    warn!(error = %e, "Failed to clear all global commands.");
                } else {
                    info!("Successfully cleared all old global commands.");
                }
            
                // 3. Register the current set of commands globally
                info!("Registering current application commands globally...");
                poise::builtins::register_globally(ctx, commands_to_register).await
                    .context("Failed to register current commands globally during setup")?;
                info!("Successfully registered current application commands globally.");
                // --- End Thorough Command Cleanup and Registration ---

                Ok((*data_for_setup).clone())
            })
        })
        .build();

    scheduler::initialize_and_start_scheduler(app_data_for_scheduler).await
        .context("Failed to initialize and start the scheduler")?;

    info!("Building Serenity client...");
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .map_err(|e| { error!(error = %e, "Fatal: Error creating Discord client"); e })
        .context("Fatal error creating Discord client")?;

    info!("Starting Discord bot connection...");
    client.start_autosharded().await
        .map_err(|e| { error!(error = %e, "Fatal: Discord client runtime error"); e })
        .context("Discord client stopped unexpectedly")?;

    Ok(())
}

#[instrument(skip(error))]
async fn on_error(error: poise::FrameworkError<'_, Data, AppError>) {
    // ... (your on_error function remains the same) ...
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            error!(error = ?error, "Poise Framework setup error");
        },
        poise::FrameworkError::Command { error, ctx, .. } => {
            let command_name = ctx.command().qualified_name.clone();
            error!(error = ?error, command = %command_name, "Error executing command");
            if let Err(e) = ctx.say("Oops, an internal error occurred while running that command!").await {
                error!(error = ?e, "Failed to send error message to Discord");
            }
        },
        other_error => {
            if let Err(e) = poise::builtins::on_error(other_error).await {
                 error!(error = ?e, "Error occurred while poise was handling another error");
            }
        }
    }
}
