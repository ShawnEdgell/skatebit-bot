use crate::types::{Context, Error};

/// Responds with pong! Checks if the bot is responsive.
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("pong!").await?;
    Ok(())
}