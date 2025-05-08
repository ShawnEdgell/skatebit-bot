use crate::types::ModEntry;

pub fn format_mod_entry(mod_entry: &ModEntry) -> String {
    let author = mod_entry.author.as_deref().unwrap_or("Unknown");
    let version = mod_entry.working_version.as_deref().unwrap_or("N/A");
    let game_version = mod_entry.game_version.as_deref().unwrap_or("N/A");
    let keybind = mod_entry.keybind.as_deref().unwrap_or("None");
    let features = mod_entry.features.as_ref().map(|f| f.join(", ")).unwrap_or_default();
    let note = mod_entry.note.as_deref().unwrap_or("");
    let note_line = if note.is_empty() {
        String::new()
    } else {
        format!("**Note:** {}\n", note)
    };
    let downloads = mod_entry.download_links.as_ref().map(|links| {
        links
            .iter()
            .map(|l| format!("[{}]({})", l.label, l.url))
            .collect::<Vec<_>>()
            .join(" | ")
    }).unwrap_or_default();

    format!(
        "**Title:** {}\n**Author:** {}\n**Mod Version:** {}\n**Game Version:** {}\n**Keybind:** {}\n**Features:** {}\n{}{}",
        mod_entry.title,
        author,
        version,
        game_version,
        keybind,
        features,
        note_line,
        downloads
    )
}