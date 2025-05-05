use serenity::{
    async_trait,
    model::{
        application::interaction::Interaction,
        gateway::Ready,
        prelude::GuildId,
    },
};
use serenity::client::{Context, EventHandler};

use crate::commands::{modlist, ping};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("Bot is connected!");

        let guild_id = GuildId(663764960202981435);

        // Register slash commands
        let _ = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|cmd| ping::register(cmd))
                .create_application_command(|cmd| modlist::register(cmd))
        })
        .await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            // Slash commands
            Interaction::ApplicationCommand(command) => {
                match command.data.name.as_str() {
                    "ping" => {
                        ping::run(&ctx, &command).await;
                    }
                    "modlist" => {
                        if let Err(e) = modlist::run(&ctx, &command).await {
                            println!("Error running modlist command: {:?}", e);
                        }
                    }
                    _ => {}
                }
            }

            // Pagination buttons
            Interaction::MessageComponent(component) => {
                if let Err(err) = modlist::handle_pagination(&ctx, &component).await {
                    println!("Error handling pagination: {:?}", err);
                }
            }

            _ => {}
        }
    }
}
