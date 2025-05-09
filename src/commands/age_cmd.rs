use crate::types::{Context, Error};
use poise::serenity_prelude as serenity;

/// Displays the account creation date of a user (or yourself).
#[poise::command(slash_command, prefix_command)]
pub async fn age(
    ctx: Context<'_>,
    user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let formatted_date = u.created_at().format("%B %e, %Y").to_string();

    ctx.say(format!("{}'s account was created on {}", u.name, formatted_date)).await?;
    Ok(())
}