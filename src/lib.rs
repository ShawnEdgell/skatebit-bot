// src/lib.rs
use serenity::prelude::GatewayIntents;
use serenity::Client;
use dotenvy::dotenv;
use std::env;
use tokio::time::{sleep, Duration};

mod handler;
mod commands;
mod utils;
mod types;

use handler::Handler;
use commands::map;

pub async fn start() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected DISCORD_TOKEN in env");
    println!("Starting with token {}…", &token[..5]);

    // Load map cache before bot starts
    println!("📦 Loading map cache...");
    map::load_cache().await?;

    // Refresh map cache every 6 hours
    tokio::spawn(async {
        loop {
            sleep(Duration::from_secs(60 * 60 * 6)).await;
            println!("🔁 Refreshing map cache...");
            if let Err(e) = map::load_cache().await {
                eprintln!("❌ Failed to refresh map cache: {:?}", e);
            } else {
                println!("✅ Map cache refreshed.");
            }
        }
    });

    let mut client = Client::builder(&token, GatewayIntents::all())
        .event_handler(Handler)
        .await?;

    client.start().await?;
    Ok(())
}
