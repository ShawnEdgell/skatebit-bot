use serenity::{
    async_trait,
    model::{
        application::interaction::Interaction,
        gateway::Ready,
        application::command::Command,
        application::interaction::InteractionResponseType,
        application::interaction::MessageFlags,
    },
    client::{Context, EventHandler},
};

use crate::commands::{modlist, modsearch, ping, map};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("✅ Bot is connected!");

        // --- Global Command Cleanup ---
        match Command::get_global_application_commands(&ctx.http).await {
            Ok(old_commands) => {
                if !old_commands.is_empty() {
                    println!("🧹 Removing {} old global command(s)...", old_commands.len());
                    for cmd in old_commands {
                        if let Err(e) = Command::delete_global_application_command(&ctx.http, cmd.id).await {
                            println!("❌ Failed to delete global command '{}': {:?}", cmd.name, e);
                        }
                    }
                    println!("🧹 Old global commands removed.");
                } else {
                     println!("ℹ️ No old global commands found to remove.");
                }
            }
            Err(e) => {
                 println!("⚠️ Could not fetch global commands to check for removal: {:?}", e);
            }
        }


        // --- Global Command Registration ---
        if let Err(e) = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|cmd| ping::register(cmd))
                .create_application_command(|cmd| modlist::register(cmd))
                .create_application_command(|cmd| modsearch::register(cmd))
                .create_application_command(|cmd| map::register(cmd))
        }).await {
            println!("❌ Failed to register global commands: {:?}", e);
        } else {
            println!("🌍 Global commands registered successfully.");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(cmd) => {
                match cmd.data.name.as_str() {
                    "ping" => {
                        ping::run(&ctx, &cmd).await;
                    }
                    "modlist" => {
                        if let Err(e) = modlist::run(&ctx, &cmd).await {
                            eprintln!("❌ Error running initial /modlist command: {:?}", e);
                            if let Err(err_resp) = cmd.create_interaction_response(&ctx.http, |r| {
                                r.kind(InteractionResponseType::ChannelMessageWithSource)
                                 .interaction_response_data(|m| {
                                     m.flags(MessageFlags::EPHEMERAL)
                                      .content("Sorry, I couldn't start the modlist process. Please try again later.")
                                 })
                            }).await {
                                eprintln!("❌ Failed to send error response for /modlist: {:?}", err_resp);
                            }
                        }
                    }
                    "modsearch" => {
                        if let Err(e) = modsearch::run(&ctx, &cmd).await {
                            eprintln!("❌ Error running modsearch: {:?}", e);
                        }
                    }
                    "map" => {
                        if let Err(e) = map::run(&ctx, &cmd).await {
                            eprintln!("❌ Error running map: {:?}", e);
                        }
                    }
                    _ => {
                        eprintln!("⚠️ Unhandled application command: {}", cmd.data.name);
                    }
                }
            }
            Interaction::MessageComponent(comp_interaction) => {
                match comp_interaction.data.custom_id.as_str() {
                    "modlist_version_select" => {
                        let selected_value = if let Some(value) = comp_interaction.data.values.get(0) {
                            value.clone()
                        } else {
                            eprintln!("❌ Modlist version select: No value selected by user {}.", comp_interaction.user.id);
                            if let Err(err_resp) = comp_interaction.create_interaction_response(&ctx.http, |r| {
                                r.kind(InteractionResponseType::ChannelMessageWithSource)
                                 .interaction_response_data(|m| m.content("You didn't select a version! Please try selecting an option from the menu.").flags(MessageFlags::EPHEMERAL))
                            }).await {
                                eprintln!("❌ Failed to send 'no selection' response: {:?}", err_resp);
                            }
                            return;
                        };

                        if let Err(e) = modlist::handle_version_selection_and_send_list(&ctx, &comp_interaction, &selected_value).await {
                            eprintln!("❌ Error in modlist::handle_version_selection_and_send_list for user {}: {:?}", comp_interaction.user.id, e);
                            if let Err(err_resp) = comp_interaction.create_followup_message(&ctx.http, |f| {
                                f.content("😥 Sorry, an error occurred while fetching that mod list. Please try again or select another option.")
                                 .flags(MessageFlags::EPHEMERAL)
                            }).await {
                                eprintln!("❌ Failed to send followup error message for modlist selection: {:?}", err_resp);
                            }
                        }
                    }
                    _ => {
                        eprintln!("⚠️ Unhandled component interaction custom_id: {}", comp_interaction.data.custom_id);
                    }
                }
            }
            Interaction::Autocomplete(auto) => {
                match auto.data.name.as_str() {
                    "modsearch" => {
                        if let Err(e) = modsearch::autocomplete(&ctx, &auto).await {
                            eprintln!("❌ Error handling modsearch autocomplete: {:?}", e);
                        }
                    }
                    "map" => {
                        if let Err(e) = map::autocomplete(&ctx, &auto).await {
                            eprintln!("❌ Error handling map autocomplete: {:?}", e);
                        }
                    }
                    _ => {
                        eprintln!("⚠️ Unhandled autocomplete command: {}", auto.data.name);
                    }
                }
            }
            _ => {
                eprintln!("⚠️ Unhandled interaction type");
            }
        }
    }
}