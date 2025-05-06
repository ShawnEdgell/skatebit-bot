// src/utils/mod_fetch.rs
use crate::types::ModEntry;
use reqwest::Client;

pub async fn fetch_mods(version_slug: &str) -> Result<Vec<ModEntry>, reqwest::Error> {
    let url = format!("https://skatebit-api.vercel.app/api/mods/{}", version_slug);
    Client::new().get(&url).send().await?.json().await
}

pub fn resolve_version(alias: &str) -> Option<&'static str> {
    match alias.to_lowercase().as_str() {
        "alpha" => Some("1228"),
        "beta" => Some("12104"),
        "public" => Some("12104"),
        _ => None,
    }
}
