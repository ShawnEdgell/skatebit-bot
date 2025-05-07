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

#[derive(Debug, Clone, Deserialize)]
pub struct ModioMap {
    // pub id: u32,
    pub name: String,
    pub summary: String,
    pub profile_url: String,
    pub logo: ModioLogo,
    pub submitted_by: ModioUser,
    pub modfile: Option<ModioModfile>,
    pub tags: Option<Vec<ModioTag>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModioLogo {
    // pub filename: String,
    pub original: String,
    // pub thumb_320x180: String,
    // pub thumb_640x360: Option<String>,
    pub thumb_1280x720: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModioUser {
    // pub id: u32,
    pub username: String,
    // pub profile_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModioModfile {
    pub download: ModioDownload,
    pub filesize: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModioDownload {
    pub binary_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModioTag {
    pub name: String,
}
