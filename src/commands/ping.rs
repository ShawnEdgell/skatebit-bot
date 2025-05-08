use serenity::{
    builder::{
        CreateCommand,
        CreateInteractionResponse,
        CreateInteractionResponseMessage,
        CreateEmbed,
    },
    client::Context,
    model::application::CommandInteraction,
};
use crate::utils::constants::BOT_EMBED_COLOR;

pub fn register() -> CreateCommand {
    CreateCommand::new("ping")
        .description("A simple command to check if the bot is responding.")
}

pub async fn run(ctx: &Context, interaction: &CommandInteraction) {
    let embed = CreateEmbed::new()
        .title("🏓 Pong!")
        .color(BOT_EMBED_COLOR);

    let response_message = CreateInteractionResponseMessage::new().add_embed(embed);
    let response = CreateInteractionResponse::Message(response_message);

    if let Err(e) = interaction.create_response(&ctx.http, response).await {
        eprintln!("Error sending ping response: {:?}", e);
    }
}