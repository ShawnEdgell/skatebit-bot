use crate::types::{Context, Error};

/// Lists available mods for a specific game version branch.
#[poise::command(slash_command, prefix_command)]
pub async fn modlist(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Simple response for /modlist command!").await?;
    Ok(())
}