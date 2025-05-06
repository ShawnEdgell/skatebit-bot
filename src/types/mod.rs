use serde::Deserialize;

#[derive(Deserialize, Clone)]
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

#[derive(Deserialize, Clone)]
pub struct DownloadLink {
    pub url: String,
    pub label: String,
}
