use serenity::{
    all::*,
    async_trait,
    client::{Context, EventHandler},
};

use crate::commands::{map, modlist, modsearch, ping};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("✅ Bot is connected!");

        match ctx.http.get_global_commands().await {
            Ok(old_commands) => {
                if !old_commands.is_empty() {
                    println!("🧹 Removing {} old global command(s)...", old_commands.len());
                    for cmd_def in old_commands {
                        if let Err(e) = ctx.http.delete_global_command(cmd_def.id).await {
                            println!("❌ Failed to delete global command '{}': {:?}", cmd_def.name, e);
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

        let commands_to_register: Vec<CreateCommand> = vec![
            ping::register(),
            modlist::register(),
            modsearch::register(),
            map::register(),
        ];

        if let Err(e) = ctx.http.create_global_commands(&commands_to_register).await {
            println!("❌ Failed to register global commands: {:?}", e);
        } else {
            println!("🌍 Global commands registered successfully.");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(cmd_interaction) => {
                match cmd_interaction.data.name.as_str() {
                    "ping" => {
                        ping::run(&ctx, &cmd_interaction).await;
                    }
                    "modlist" => {
                        if let Err(e) = modlist::run(&ctx, &cmd_interaction).await {
                            eprintln!("❌ Error running initial /modlist command: {:?}", e);
                            let response_message = CreateInteractionResponseMessage::new()
                                .content("Sorry, I couldn't start the modlist process. Please try again later.")
                                .ephemeral(true);
                            let response = CreateInteractionResponse::Message(response_message);
                            if let Err(err_resp) = cmd_interaction.create_response(&ctx.http, response).await {
                                eprintln!("❌ Failed to send error response for /modlist: {:?}", err_resp);
                            }
                        }
                    }
                    "modsearch" => {
                           if let Err(e) = modsearch::run(&ctx, &cmd_interaction).await {
                                eprintln!("❌ Error running modsearch: {:?}", e);
                            }
                    }
                    "map" => {
                           if let Err(e) = map::run(&ctx, &cmd_interaction).await {
                                eprintln!("❌ Error running map: {:?}", e);
                            }
                    }
                    _ => {
                        eprintln!("⚠️ Unhandled application command: {}", cmd_interaction.data.name);
                    }
                }
            }
            Interaction::Component(comp_interaction) => {
                match comp_interaction.data.custom_id.as_str() {
                    "modlist_version_select" => {
                        let selected_string_values_vec = match &comp_interaction.data.kind {
                            ComponentInteractionDataKind::StringSelect { values } => values,
                            _ => {
                                eprintln!("❌ Modlist version select: Expected StringSelect kind, got: {:?}", comp_interaction.data.kind);
                                let err_msg = CreateInteractionResponseMessage::new()
                                    .content("⚠️ Internal error: Unexpected component type.")
                                    .ephemeral(true);
                                let err_resp = CreateInteractionResponse::Message(err_msg);
                                if let Err(e) = comp_interaction.create_response(&ctx.http, err_resp).await {
                                    eprintln!("❌ Failed to send component kind error response: {:?}", e);
                                }
                                return;
                            }
                        };

                        let selected_value = if let Some(value_from_vec) = selected_string_values_vec.get(0) {
                            value_from_vec.clone()
                        } else {
                            eprintln!("❌ Modlist version select: No value selected/found in values vector for user {}.", comp_interaction.user.id);
                            let response_message = CreateInteractionResponseMessage::new()
                                .content("You didn't select a version! Please try selecting an option from the menu.")
                                .ephemeral(true);
                            let response = CreateInteractionResponse::Message(response_message);
                            if let Err(err_resp) = comp_interaction.create_response(&ctx.http, response).await {
                                eprintln!("❌ Failed to send 'no selection' response: {:?}", err_resp);
                            }
                            return;
                        };

                        if let Err(e) = modlist::handle_version_selection_and_send_list(&ctx, &comp_interaction, &selected_value).await {
                            eprintln!("❌ Error in modlist::handle_version_selection_and_send_list for user {}: {:?}", comp_interaction.user.id, e);

                            let followup_message_builder = CreateInteractionResponseFollowup::new()
                                .content("😥 Sorry, an error occurred while fetching that mod list. Please try again or select another option.")
                                .ephemeral(true);

                            if let Err(err_resp) = comp_interaction
                                .create_followup(&ctx.http, followup_message_builder)
                                .await
                            {
                                eprintln!(
                                    "❌ Failed to send followup error message for modlist selection: {:?}",
                                    err_resp
                                );
                            }
                        }
                    }
                    _ => {
                        eprintln!("⚠️ Unhandled component interaction custom_id: {}", comp_interaction.data.custom_id);
                    }
                }
            }
            Interaction::Autocomplete(auto_cmd_interaction) => {
                   match auto_cmd_interaction.data.name.as_str() {
                       "modsearch" => {
                         if let Err(e) = modsearch::autocomplete(&ctx, &auto_cmd_interaction).await {
                                eprintln!("❌ Error handling modsearch autocomplete: {:?}", e);
                            }
                       }
                       "map" => {
                          if let Err(e) = map::autocomplete(&ctx, &auto_cmd_interaction).await {
                                eprintln!("❌ Error handling map autocomplete: {:?}", e);
                            }
                        }
                       _ => {
                            eprintln!("⚠️ Unhandled autocomplete command: {}", auto_cmd_interaction.data.name);
                       }
                   }
            }
            Interaction::Ping(_) => {}
            Interaction::Modal(modal_interaction) => {
                eprintln!("⚠️ Unhandled modal submission: custom_id: {}", modal_interaction.data.custom_id);
            }
            ref unhandled_interaction => {
                eprintln!("⚠️ Unhandled interaction type: {:?}", unhandled_interaction);
            }
        }
    }
}