use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::client::Context;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("ping").description("Replies with pong")
}

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) {
    let _ = interaction.create_interaction_response(&ctx.http, |r| {
        r.kind(serenity::model::prelude::InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m.content("🏓 Pong!"))
    }).await;
}
