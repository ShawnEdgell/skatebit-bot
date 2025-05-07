use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        application::command::Command,
        application::interaction::Interaction,
        gateway::Ready,
    },
};

use crate::commands::{map, modlist, modsearch, ping};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("✅ Bot is connected!");

        // Optional: Clear old global commands (useful when renaming or removing commands)
        if let Ok(old_commands) = Command::get_global_application_commands(&ctx.http).await {
            for cmd in old_commands {
                let _ = Command::delete_global_application_command(&ctx.http, cmd.id).await;
            }
            println!("🧹 Removed old global commands.");
        }

        // Register global application commands
        if let Err(e) = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|cmd| ping::register(cmd))
                .create_application_command(|cmd| modlist::register(cmd))
                .create_application_command(|cmd| modsearch::register(cmd))
                .create_application_command(|cmd| map::register(cmd))
        }).await {
            println!("❌ Failed to register global commands: {:?}", e);
        } else {
            println!("🌐 Global commands registered.");
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
                            println!("❌ Error running modlist: {:?}", e);
                        }
                    }
                    "modsearch" => {
                        if let Err(e) = modsearch::run(&ctx, &cmd).await {
                            println!("❌ Error running modsearch: {:?}", e);
                        }
                    }
                    "map" => {
                        if let Err(e) = map::run(&ctx, &cmd).await {
                            println!("❌ Error running map: {:?}", e);
                        }
                    }
                    _ => {}
                }
            }
            Interaction::MessageComponent(comp) => {
                if let Err(e) = modlist::handle_pagination(&ctx, &comp).await {
                    println!("❌ Error handling pagination: {:?}", e);
                }
            }
            Interaction::Autocomplete(auto) => {
                match auto.data.name.as_str() {
                    "modsearch" => {
                        if let Err(e) = modsearch::autocomplete(&ctx, &auto).await {
                            println!("❌ Error handling modsearch autocomplete: {:?}", e);
                        }
                    }
                    "map" => {
                        if let Err(e) = map::autocomplete(&ctx, &auto).await {
                            println!("❌ Error handling map autocomplete: {:?}", e);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
