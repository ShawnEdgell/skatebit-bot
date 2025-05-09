use serde::Deserialize;
use std::{collections::HashMap, fmt, str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use reqwest::Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModVersionBranch {
    Alpha,
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
pub struct ModioMap {
    pub name: String,
    pub summary: String,
    pub profile_url: String,
    pub logo: ModioLogo,
    pub submitted_by: ModioUser,
    pub modfile: Option<ModioModfile>,
    pub tags: Option<Vec<ModioTag>>,
}
#[derive(Debug, Clone, Deserialize)] pub struct ModioLogo { pub original: String, pub thumb_1280x720: Option<String> }
#[derive(Debug, Clone, Deserialize)] pub struct ModioUser { pub username: String }
#[derive(Debug, Clone, Deserialize)] pub struct ModioModfile { pub download: ModioDownload, pub filesize: Option<u64> }
#[derive(Debug, Clone, Deserialize)] pub struct ModioDownload { pub binary_url: String }
#[derive(Debug, Clone, Deserialize)] pub struct ModioTag { pub name: String }
#[derive(Deserialize)] pub struct ModioPage { #[serde(rename = "maps")] pub map_entries: Vec<ModioMap> }

#[derive(Debug, Clone)]
pub struct Data {
    pub map_cache: Arc<RwLock<Vec<ModioMap>>>,
    pub http_client: Arc<Client>,
    pub mod_cache: Arc<RwLock<HashMap<String, Vec<ModEntry>>>>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            map_cache: Arc::new(RwLock::new(Vec::new())),
            http_client: Arc::new(Client::new()),
            mod_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

pub type Error = anyhow::Error;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub const BOT_EMBED_COLOR: u32 = 0x1eaeef;