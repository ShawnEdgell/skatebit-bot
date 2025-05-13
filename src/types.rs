// src/types.rs
use poise::ChoiceParameter;
use serde::Deserialize;
use std::{collections::HashMap, fmt, str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use reqwest::Client;
// If you want to parse the "lastUpdated" string from the API into a chrono::DateTime:
// use chrono::{DateTime, Utc};

// --- These types seem related to your specific script mod fetching by slug ---
// --- and will remain UNCHANGED for now as per your clarification.         ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, ChoiceParameter)]
pub enum ModVersionBranch {
    #[name = "Alpha"]
    Alpha,
    #[name = "Beta/Public"]
    BetaPublic,
}

impl FromStr for ModVersionBranch {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "alpha" => Ok(Self::Alpha),
            "beta" | "public" | "beta/public" => Ok(Self::BetaPublic),
            _ => Err(format!("Unknown version branch: '{}'. Use Alpha or Beta/Public.", s)),
        }
    }
}

impl fmt::Display for ModVersionBranch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alpha => write!(f, "Alpha"),
            Self::BetaPublic => write!(f, "Beta/Public"),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModEntry {
    pub title: String,
    pub author: Option<String>,
    #[serde(rename = "workingVersion")]
    pub working_version: Option<String>,
    #[serde(rename = "gameVersion")]
    pub game_version: Option<String>,
    pub keybind: Option<String>,
    pub features: Option<Vec<String>>,
    pub note: Option<String>,
    #[serde(rename = "downloadLinks")]
    pub download_links: Option<Vec<DownloadLink>>,
}
#[derive(Deserialize, Clone, Debug)] pub struct DownloadLink { pub url: String, pub label: String }

// --- End of unchanged script mod types ---

// --- NEW/UPDATED Structs for deserializing MAP data FROM YOUR GO API ---
// These structs need to match the JSON structure produced by your Go API's 
// `modio.Mod` struct (from modio-api-go/internal/modio/types.go)

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioUser { // Corresponds to Go's ModioUser
    pub id: i32,
    pub username: String,
    #[serde(rename = "profile_url")] // Matches json:"profile_url" in Go
    pub profile_url: Option<String>, // Your Go struct had this as non-optional, ensure consistency
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioLogo { // Corresponds to Go's ModioLogo
    pub filename: String,
    pub original: String,
    #[serde(rename = "thumb_320x180")]
    pub thumb_320x180: Option<String>,
    // Add these if your Go API's ModioLogo struct includes them and they are sent
    #[serde(rename = "thumb_640x360")]
    pub thumb_640x360: Option<String>,
    #[serde(rename = "thumb_1280x720")]
    pub thumb_1280x720: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioDownload { // Corresponds to embedded Download in Go's ModioModfile
    #[serde(rename = "binary_url")]
    pub binary_url: String, // Assuming always present from Go API
    #[serde(rename = "date_expires")]
    pub date_expires: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioModfile { // Corresponds to Go's ModioModfile
    pub id: i32,
    pub filename: Option<String>, // Your previous Rust struct had Option, matching here
    pub version: Option<String>,  // Your previous Rust struct had Option, matching here
    pub filesize: Option<i64>,    // Your previous Rust struct had Option<u64>, Go has i64. i64 is fine.
    pub download: ApiModioDownload, // Assuming download struct itself is always present
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioTag { // Corresponds to Go's ModioTag
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioStats { // Corresponds to Go's ModioStats
    #[serde(rename = "downloads_total")]
    pub downloads_total: i32,
    #[serde(rename = "subscribers_total")]
    pub subscribers_total: i32,
    #[serde(rename = "ratings_positive")]
    pub ratings_positive: i32,
    #[serde(rename = "ratings_negative")]
    pub ratings_negative: i32,
    #[serde(rename = "ratings_display_text")]
    pub ratings_display_text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioImage { // Corresponds to Go's ModioImage
    pub filename: String,
    pub original: String,
    #[serde(rename = "thumb_320x180")]
    pub thumb_320x180: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioMedia { // Corresponds to Go's ModioMedia
    // Ensure your Go API sends "images" as the key if this field exists
    pub images: Option<Vec<ApiModioImage>>, 
}

// This is the primary struct for a map item received from your Go API's "items" array.
// It replaces your old `ModioMap` for the purpose of consuming the new Go API.
// Fields here MUST align with the JSON produced by your Go `modio.Mod` struct.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioMap { // Changed name to distinguish from old ModioMap if it was different
    pub id: i32,
    #[serde(rename = "game_id")] // Matches Go `json:"game_id"`
    pub game_id: i32,
    pub name: String,
    #[serde(rename = "name_id")] // Matches Go `json:"name_id"`
    pub name_id: String,
    pub summary: String, // If these can be null from Go API, make them Option<String>
    #[serde(rename = "description_plaintext")] // Matches Go `json:"description_plaintext"`
    pub description: String, // If these can be null from Go API, make them Option<String>
    #[serde(rename = "profile_url")] // Matches Go `json:"profile_url"`
    pub profile_url: String,
    #[serde(rename = "submitted_by")] // Matches Go `json:"submitted_by"`
    pub submitted_by: ApiModioUser,
    #[serde(rename = "date_added")] // Matches Go `json:"date_added"`
    pub date_added: i64,
    #[serde(rename = "date_updated")] // Matches Go `json:"date_updated"`
    pub date_updated: i64,
    #[serde(rename = "date_live")] // Matches Go `json:"date_live"`
    pub date_live: i64,
    pub logo: ApiModioLogo,
    // Your Go Mod struct has Modfile as non-optional. If it can be null from Mod.io,
    // your Go struct should be *ModioModfile, and then this Rust field should be Option<ApiModioModfile>.
    // For now, assuming Go API always sends a modfile object (even if its fields are empty/null).
    // If it can be truly null from Go, then this needs to be Option<ApiModioModfile>.
    // Let's make it Option<> here to be safe, matching your original ModioMap.
    pub modfile: Option<ApiModioModfile>, 
    pub tags: Option<Vec<ApiModioTag>>, // Matches Go `json:"tags"` and your previous Option<>
    pub stats: ApiModioStats,           // Matches Go `json:"stats"`
    pub media: Option<ApiModioMedia>,   // Matches Go `json:"media"`, make Option<>
}

// This struct is for the overall response from your Go API's MAPS endpoint
#[derive(Deserialize, Debug)]
pub struct GoApiMapsResponse {
    #[serde(rename = "itemType")]
    pub item_type: String,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
    #[serde(rename = "count")] // Expect "count" from the JSON
    pub count: usize,          // Store it in a field named 'count'
    pub items: Vec<ApiModioMap>,
}

// You might also want one for scripts if the Go API endpoint for scripts returns the same overall structure
// but with item_type: "scripts" and items being the script mods (which might also use ApiModioMap/ModItem struct)
// #[derive(Deserialize, Debug)]
// pub struct GoApiScriptsResponse { ... same fields ... }


// --- Updated Data struct for Poise ---
#[derive(Debug, Clone)]
pub struct Data {
    pub map_cache: Arc<RwLock<Vec<ApiModioMap>>>, // Now stores the new map struct type
    pub http_client: Arc<Client>,
    pub mod_cache: Arc<RwLock<HashMap<String, Vec<ModEntry>>>>, // This remains UNCHANGED (for slug-based mods)
}

impl Data {
    pub fn new() -> Self {
        Self {
            map_cache: Arc::new(RwLock::new(Vec::new())),
            http_client: Arc::new(Client::new()),
            mod_cache: Arc::new(RwLock::new(HashMap::new())), // Stays as is
        }
    }
}

pub type Error = anyhow::Error;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub const BOT_EMBED_COLOR: u32 = 0x1eaeef;