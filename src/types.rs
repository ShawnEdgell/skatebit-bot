use poise::ChoiceParameter;
use serde::Deserialize;
use std::{collections::HashMap, fmt, str::FromStr, sync::Arc}; // Keep FromStr for ModVersionBranch
use tokio::sync::RwLock;
use reqwest::Client as ReqwestClient;
use deadpool_redis::{Pool, Config as DeadpoolRedisConfig, Runtime as DeadpoolRuntime};

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

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioUser {
    pub id: i32,
    pub username: String,
    #[serde(rename = "profile_url")]
    pub profile_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioLogo {
    pub filename: String,
    pub original: String,
    #[serde(rename = "thumb_320x180")]
    pub thumb_320x180: Option<String>,
    #[serde(rename = "thumb_640x360")]
    pub thumb_640x360: Option<String>,
    #[serde(rename = "thumb_1280x720")]
    pub thumb_1280x720: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioDownload {
    #[serde(rename = "binary_url")]
    pub binary_url: String,
    #[serde(rename = "date_expires")]
    pub date_expires: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioModfile {
    pub id: i32,
    pub filename: Option<String>,
    pub version: Option<String>,
    pub filesize: Option<i64>,
    pub download: ApiModioDownload,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioTag {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioStats {
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
pub struct ApiModioImage {
    pub filename: String,
    pub original: String,
    #[serde(rename = "thumb_320x180")]
    pub thumb_320x180: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioMedia {
    pub images: Option<Vec<ApiModioImage>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiModioMap {
    pub id: i32,
    #[serde(rename = "game_id")]
    pub game_id: i32,
    pub name: String,
    #[serde(rename = "name_id")]
    pub name_id: String,
    pub summary: String,
    #[serde(rename = "description_plaintext")]
    pub description: String,
    #[serde(rename = "profile_url")]
    pub profile_url: String,
    #[serde(rename = "submitted_by")]
    pub submitted_by: ApiModioUser,
    #[serde(rename = "date_added")]
    pub date_added: i64,
    #[serde(rename = "date_updated")]
    pub date_updated: i64,
    #[serde(rename = "date_live")]
    pub date_live: i64,
    pub logo: ApiModioLogo,
    pub modfile: Option<ApiModioModfile>,
    pub tags: Option<Vec<ApiModioTag>>,
    pub stats: ApiModioStats,
    pub media: Option<ApiModioMedia>,
}

#[derive(Clone)] // Removed Debug derive for now
pub struct Data {
    pub http_client: Arc<ReqwestClient>,
    pub mod_cache: Arc<RwLock<HashMap<String, Vec<ModEntry>>>>,
    pub redis_pool: Arc<Pool>,
}

// Manual implementation of Debug for Data
impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Data")
            .field("http_client", &self.http_client) // Arc<ReqwestClient> is Debug
            .field("mod_cache", &self.mod_cache)     // Arc<RwLock<...>> is Debug if inner is Debug
            .field("redis_pool", &"<Redis Pool>") // Placeholder for non-Debug Pool
            .finish()
    }
}


impl Data {
    pub fn new(redis_url: &str) -> Result<Self, Error> { // Changed AppError to Error
        let cfg = DeadpoolRedisConfig::from_url(redis_url);
        let pool = cfg.create_pool(Some(DeadpoolRuntime::Tokio1))
            .map_err(|e| anyhow::anyhow!("Failed to create Redis pool: {}", e))?;
        
        Ok(Self {
            http_client: Arc::new(ReqwestClient::new()),
            mod_cache: Arc::new(RwLock::new(HashMap::new())),
            redis_pool: Arc::new(pool),
        })
    }
}

pub type Error = anyhow::Error;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub const BOT_EMBED_COLOR: u32 = 0x1eaeef;

// Constants for mod.io tags, matching Go API's repository/modio packages
pub const MAP_TAG: &str = "Map";
pub const SCRIPT_MOD_TAG: &str = "Script";
