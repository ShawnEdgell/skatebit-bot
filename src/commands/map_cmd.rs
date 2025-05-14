use crate::types::{Context, Error, ApiModioMap, BOT_EMBED_COLOR, MAP_TAG};
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use tracing::{info, warn, error};
use deadpool_redis::redis::AsyncCommands; // Corrected import for the trait

fn normalize_title_for_redis(title: &str) -> String {
    title.to_lowercase().trim().to_string()
}

async fn map_name_autocomplete(
    ctx: Context<'_>,
    partial: &str,
) -> Vec<String> {
    let mut final_suggestions = Vec::new();
    let limit = 25; // Discord's limit for autocomplete choices

    let mut redis_conn = match ctx.data().redis_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Autocomplete: Failed to get Redis connection: {}", e);
            return final_suggestions; // Return empty on error
        }
    };

    let redis_key = "mod_titles:map";
    let members_str: Vec<String> = if partial.is_empty() {
        // Fetch first `limit` members if partial is empty (default suggestions)
        info!("Autocomplete: Partial is empty, fetching default suggestions.");
        match redis_conn.zrange(redis_key, 0, (limit - 1) as isize).await {
            Ok(members) => members,
            Err(e) => {
                error!("Autocomplete: Redis ZRANGE error for default suggestions: {}", e);
                return final_suggestions;
            }
        }
    } else {
        // Existing logic for when partial is not empty
        let partial_normalized = normalize_title_for_redis(partial);
        let min_lex = format!("[{}", partial_normalized);
        let max_lex = format!("[{}{}", partial_normalized, std::char::from_u32(0xFF).unwrap_or('~'));
        
        match redis_conn.zrangebylex_limit(redis_key, min_lex.clone(), max_lex.clone(), 0, limit as isize).await {
            Ok(members) => members,
            Err(e) => {
                error!("Autocomplete: Redis ZRANGEBYLEX error: {}", e);
                return final_suggestions;
            }
        }
    };

    if members_str.is_empty() {
        return final_suggestions;
    }

    let mut mod_ids_to_fetch: Vec<String> = Vec::new();
    let mut member_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for member in members_str {
        if let Some(colon_idx) = member.rfind(':') {
            let id_part = member[colon_idx+1..].to_string();
            let mod_key_for_mget = format!("mod:{}", id_part);
            mod_ids_to_fetch.push(mod_key_for_mget.clone());
            member_map.insert(mod_key_for_mget, member.clone());
        }
    }

    if !mod_ids_to_fetch.is_empty() {
        match redis_conn.mget::<Vec<String>, Vec<Option<String>>>(mod_ids_to_fetch.clone()).await {
            Ok(mod_jsons) => {
                for (i, mod_json_opt) in mod_jsons.into_iter().enumerate() {
                    let original_member_key = &mod_ids_to_fetch[i]; // key used for member_map
                    if let Some(mod_json_str) = mod_json_opt { 
                        if let Ok(mod_data) = serde_json::from_str::<ApiModioMap>(&mod_json_str) {
                            let display_title = if mod_data.name.len() > 80 {
                                format!("{}...", &mod_data.name[..77])
                            } else {
                                mod_data.name.clone()
                            };
                            final_suggestions.push(format!("{} (ID: {})", display_title, mod_data.id));
                        } else {
                             // Deserialization failed, use fallback from member_map
                             if let Some(original_member) = member_map.get(original_member_key) {
                                if let Some(colon_idx) = original_member.rfind(':') {
                                    let title_part = &original_member[..colon_idx];
                                    let id_part = &original_member[colon_idx+1..];
                                    let display_title = if title_part.len() > 80 { format!("{}...", &title_part[..77]) } else { title_part.to_string() };
                                    final_suggestions.push(format!("{} (ID: {})", display_title, id_part));
                                }
                             }
                        }
                    } else {
                        // MGET returned None for this key, use fallback
                        if let Some(original_member) = member_map.get(original_member_key) {
                           if let Some(colon_idx) = original_member.rfind(':') {
                               let title_part = &original_member[..colon_idx];
                               let id_part = &original_member[colon_idx+1..];
                               let display_title = if title_part.len() > 80 { format!("{}...", &title_part[..77]) } else { title_part.to_string() };
                               final_suggestions.push(format!("{} (ID: {})", display_title, id_part));
                           }
                        }
                   }
                   if final_suggestions.len() >= limit { break; }
                }
            }
            Err(e) => {
                error!("Autocomplete: Redis MGET error for mod details: {}. Falling back to ZSET members.", e);
                for member_key_with_prefix in mod_ids_to_fetch { 
                    if let Some(original_member) = member_map.get(&member_key_with_prefix) {
                        if let Some(colon_idx) = original_member.rfind(':') {
                            let title_part = &original_member[..colon_idx];
                            let id_part = &original_member[colon_idx+1..];
                            let display_title = if title_part.len() > 80 { format!("{}...", &title_part[..77]) } else { title_part.to_string() };
                            final_suggestions.push(format!("{} (ID: {})", display_title, id_part));
                        }
                        if final_suggestions.len() >= limit { break; }
                    }
                }
            }
        }
    }
    
    final_suggestions.truncate(limit); 
    final_suggestions
}

/// Search for a Skater XL map by name.
#[poise::command(slash_command, prefix_command)]
pub async fn map(
    ctx: Context<'_>,
    #[description = "Map name or ID (use autocomplete for best results)"]
    #[autocomplete = "map_name_autocomplete"]
    search: String,
) -> Result<(), Error> {
    info!(user = %ctx.author().name, query = %search, "Map command received");

    let mut redis_conn = match ctx.data().redis_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Map Command: Failed to get Redis connection: {}", e);
            ctx.say("Sorry, I couldn't connect to the map database right now. Please try again later.").await?;
            return Ok(());
        }
    };

    let mut mod_id_to_fetch: Option<i32> = None;

    if let Some(start_idx) = search.rfind("(ID: ") {
        if let Some(end_idx) = search.rfind(')') {
            if start_idx < end_idx {
                let id_str = &search[start_idx + 5..end_idx];
                if let Ok(id) = id_str.parse::<i32>() {
                    mod_id_to_fetch = Some(id);
                }
            }
        }
    }

    let mut found_map_data: Option<ApiModioMap> = None;

    if let Some(id) = mod_id_to_fetch {
        let mod_key = format!("mod:{}", id);
        info!("Map Command: Attempting to fetch map by ID from Redis: {}", mod_key);
        match redis_conn.get::<&str, Option<String>>(&mod_key).await {
            Ok(Some(mod_json_str)) => { 
                match serde_json::from_str::<ApiModioMap>(&mod_json_str) {
                    Ok(map_item) => {
                        if map_item.tags.as_ref().map_or(false, |tags| tags.iter().any(|t| t.name == MAP_TAG)) {
                           found_map_data = Some(map_item);
                        } else {
                            info!("Map Command: Mod ID {} found but is not tagged as a Map.", id);
                        }
                    }
                    Err(e) => error!("Map Command: Failed to deserialize mod JSON from Redis for ID {}: {}", id, e),
                }
            }
            Ok(None) => info!("Map Command: No map found in Redis for ID: {}", id),
            Err(e) => error!("Map Command: Redis GET error for mod ID {}: {}", id, e),
        }
    } else {
        warn!("Map Command: No ID parsed from search term: '{}'. Attempting prefix search.", search);
        let normalized_search = normalize_title_for_redis(&search);
        let redis_key = "mod_titles:map";
        let min_lex = format!("[{}", normalized_search);
        let max_lex = format!("[{}{}", normalized_search, std::char::from_u32(0xFF).unwrap_or('~'));

        match redis_conn.zrangebylex_limit(redis_key, min_lex, max_lex, 0, 2).await {
            Ok(members_str) => {
                let members: Vec<String> = members_str;
                if members.len() == 1 {
                    if let Some(colon_idx) = members[0].rfind(':') {
                        let id_str = &members[0][colon_idx+1..];
                        if let Ok(id) = id_str.parse::<i32>() {
                            let mod_key = format!("mod:{}", id);
                            info!("Map Command: Single match from prefix search, fetching mod: {}", id);
                            match redis_conn.get::<&str, Option<String>>(&mod_key).await {
                                Ok(Some(mod_json_str)) => {
                                    if let Ok(map_item) = serde_json::from_str::<ApiModioMap>(&mod_json_str) {
                                        if map_item.tags.as_ref().map_or(false, |tags| tags.iter().any(|t| t.name == MAP_TAG)) {
                                            found_map_data = Some(map_item);
                                        }
                                    }
                                }
                                Ok(None) => info!("Map Command: Mod (from prefix search) not found in Redis for ID: {}", id),
                                Err(e) => error!("Map Command: Redis GET error for mod ID {} (from prefix search): {}", id, e),
                            }
                        }
                    }
                } else if members.len() > 1 {
                     info!("Map Command: Multiple potential matches for manual search: '{}'. Suggesting autocomplete.", search);
                } else {
                    info!("Map Command: No matches found for manual search: '{}'.", search);
                }
            }
            Err(e) => error!("Map Command: Redis ZRANGEBYLEX error for manual search: {}", e),
        }
    }

    let reply_message = if let Some(entry) = found_map_data {
        info!(map_name = %entry.name, map_id = entry.id, "Map found and processed");

        let author = &entry.submitted_by.username;
        let download_link = entry.modfile.as_ref().map(|mf| mf.download.binary_url.as_str()).unwrap_or("N/A");
        let download_field_value = if download_link == "N/A" { "No download link".to_string() } else { format!("[Download Map]({})", download_link) };
        let size_mb = entry.modfile.as_ref().and_then(|mf| mf.filesize).map(|s| format!("{:.2} MB", s as f64 / (1024.0 * 1024.0))).unwrap_or_else(|| "Unknown".to_string());
        let tags_str = entry.tags.as_ref().filter(|tv| !tv.is_empty()).map(|tv| tv.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_else(|| "None".to_string());
        let image_url = entry.logo.thumb_1280x720.as_deref().unwrap_or_else(|| entry.logo.original.as_str());

        let embed = serenity::CreateEmbed::default()
            .title(&entry.name)
            .url(&entry.profile_url)
            .description(&entry.summary)
            .color(BOT_EMBED_COLOR)
            .image(image_url)
            .field("Author", author, true)
            .field("Size", &size_mb, true)
            .field("Tags", tags_str, false)
            .field("Link", download_field_value, false)
            .timestamp(serenity::Timestamp::now())
            .footer(serenity::CreateEmbedFooter::new(format!("ID: {} | Source: mod.io | Requested by {}", entry.id, ctx.author().name)));
        
        CreateReply::default().embed(embed)
    } else {
        warn!(query = %search, "Final: Map not found or ambiguous");
        CreateReply::default()
            .content(format!("❌ Map not found matching: '{}'.\nTip: Use the autocomplete suggestions for best results, or make sure the name is exact.", search))
            .ephemeral(true)
    };

    ctx.send(reply_message).await?;
    Ok(())
}
