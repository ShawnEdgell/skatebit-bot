// src/commands/ping.rs
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use crate::utils::constants::BOT_EMBED_COLOR; // Assuming you have this for color

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("ping")
        .description("A simple command to check if the bot is responding.") // Updated description
}

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) {
    if let Err(e) = interaction.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| {
                m.embed(|e| {
                    e.title("🏓 Pong!")
                     // Optional: Add a minimal description if desired
                     // e.description("Bot is online and responding.")
                     .color(BOT_EMBED_COLOR) // Use your theme color
                })
            })
    }).await {
        eprintln!("Error sending ping response: {:?}", e);
    }
}