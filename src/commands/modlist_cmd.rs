// src/commands/modlist_cmd.rs
use crate::types::{Context, Error};

/// Provides a link to the working mod list.
#[poise::command(slash_command, prefix_command)]
pub async fn modlist(ctx: Context<'_>) -> Result<(), Error> {
    let response_text = "You can find the full list of mods here: https://skatebit.vercel.app/mods";
    ctx.say(response_text).await?;
    Ok(())
}
