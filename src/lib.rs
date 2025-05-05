// src/lib.rs
use serenity::prelude::GatewayIntents;
use serenity::Client;
use dotenvy::dotenv;
use std::env;

mod handler;
mod commands;

use handler::Handler;

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected DISCORD_TOKEN in env");
    println!("Starting with token {}…", &token[..5]);

    let mut client = Client::builder(&token, GatewayIntents::all())
        .event_handler(Handler)
        .await?;
    client.start().await?;
    Ok(())
}
