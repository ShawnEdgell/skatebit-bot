use serenity::{
    async_trait,
    model::{
        application::command::Command,
        application::interaction::Interaction,
        gateway::Ready,
    },
};
use serenity::client::{Context, EventHandler};

use crate::commands::{modlist, modsearch, ping};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("Bot is connected!");

        // === REGISTER GLOBAL COMMANDS ===
        // This will replace any existing global commands.
        let _ = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|cmd| ping::register(cmd))
                .create_application_command(|cmd| modlist::register(cmd))
                .create_application_command(|cmd| modsearch::register(cmd))
        })
        .await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            // Slash commands
            Interaction::ApplicationCommand(cmd) => {
                match cmd.data.name.as_str() {
                    "ping" => {
                        ping::run(&ctx, &cmd).await;
                    }
                    "modlist" => {
                        if let Err(e) = modlist::run(&ctx, &cmd).await {
                            println!("Error running modlist: {:?}", e);
                        }
                    }
                    "modsearch" => {
                        if let Err(e) = modsearch::run(&ctx, &cmd).await {
                            println!("Error running modsearch: {:?}", e);
                        }
                    }
                    _ => {}
                }
            }

            // Pagination buttons for /modlist
            Interaction::MessageComponent(comp) => {
                if let Err(e) = modlist::handle_pagination(&ctx, &comp).await {
                    println!("Error handling pagination: {:?}", e);
                }
            }

            // Autocomplete for /modsearch
            Interaction::Autocomplete(auto) => {
                if let Err(e) = modsearch::autocomplete(&ctx, &auto).await {
                    println!("Error handling autocomplete: {:?}", e);
                }
            }

            _ => {}
        }
    }
}
