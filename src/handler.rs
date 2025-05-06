use serenity::{
    async_trait,
    model::{
        application::interaction::Interaction,
        gateway::Ready,
        id::GuildId,
        application::command::Command,
    },
    client::{Context, EventHandler},
};

use crate::commands::{modlist, modsearch, ping};

const GUILD_ID: u64 = 663764960202981435;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("✅ Bot is connected!");

        let guild_id = GuildId(GUILD_ID);

        // Optional: Clear old global commands
        if let Ok(old_commands) = Command::get_global_application_commands(&ctx.http).await {
            for cmd in old_commands {
                let _ = Command::delete_global_application_command(&ctx.http, cmd.id).await;
            }
            println!("🧹 Removed old global commands.");
        }

        if let Err(e) = guild_id.set_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|cmd| ping::register(cmd))
                .create_application_command(|cmd| modlist::register(cmd))
                .create_application_command(|cmd| modsearch::register(cmd))
        }).await {
            println!("❌ Failed to register commands: {:?}", e);
        } else {
            println!("📦 Guild commands registered.");
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
                    _ => {}
                }
            }
            Interaction::MessageComponent(comp) => {
                if let Err(e) = modlist::handle_pagination(&ctx, &comp).await {
                    println!("❌ Error handling pagination: {:?}", e);
                }
            }
            Interaction::Autocomplete(auto) => {
                if let Err(e) = modsearch::autocomplete(&ctx, &auto).await {
                    println!("❌ Error handling autocomplete: {:?}", e);
                }
            }
            _ => {}
        }
    }
}
