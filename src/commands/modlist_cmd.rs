use crate::types::{Context, Error, BOT_EMBED_COLOR};
use poise::serenity_prelude as serenity;
use poise::CreateReply;

/// Provides a link to the Skater XL working mod list.
#[poise::command(slash_command, prefix_command)]
pub async fn modlist(ctx: Context<'_>) -> Result<(), Error> {
    let mod_list_url = "https://skatebit.app/mods"; // Updated URL

    let embed = serenity::CreateEmbed::default()
        .title("🔗 Skater XL Mod List")
        .description(format!(
            "You can find the full, community-curated list of Skater XL script mods here:\n\n**[Click here to view the Mod List]({})**", // Made link text bold and a clear call to action
            mod_list_url 
        ))
        .color(BOT_EMBED_COLOR)
        .timestamp(serenity::Timestamp::now())
        .footer(serenity::CreateEmbedFooter::new(format!("Requested by {}", ctx.author().name)));

    let reply = CreateReply::default().embed(embed);
    ctx.send(reply).await?;

    Ok(())
}
