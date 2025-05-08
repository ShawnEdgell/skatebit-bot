use serenity::{
    prelude::GatewayIntents,
    Client,
};
use dotenvy::dotenv;
use std::env;
use tokio::time::{sleep, Duration};

mod handler;
mod commands;
mod utils;
mod types;

use handler::Handler;

pub async fn start() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in env");
    println!("Starting with token {}…", if token.len() >= 5 { &token[..5] } else { "INVALID_TOKEN_TOO_SHORT" });

    println!("📦 Loading map cache...");
    if let Err(e) = crate::commands::map::load_cache().await {
        eprintln!("❌ Failed to initially load map cache: {:?}", e);
    } else {
        println!("✅ Initial map cache loaded.");
    }

    tokio::spawn(async {
        loop {
            sleep(Duration::from_secs(60 * 60 * 6)).await;
            println!("🔁 Refreshing map cache...");
            if let Err(e) = crate::commands::map::load_cache().await {
                eprintln!("❌ Failed to refresh map cache: {:?}", e);
            } else {
                println!("✅ Map cache refreshed.");
            }
        }
    });

    let mut intents = GatewayIntents::GUILDS;
    intents |= GatewayIntents::DIRECT_MESSAGES;

    println!("ℹ️ Using intents: {:?}", intents);

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating Serenity client");

    if let Err(why) = client.start().await {
        eprintln!("❌ Client error during startup: {:?}", why);
        return Err(Box::new(why));
    }

    Ok(())
}