use serenity::builder::CreateActionRow;
use serenity::model::application::component::ButtonStyle;

pub fn pagination_buttons(row: &mut CreateActionRow) -> &mut CreateActionRow {
    row.create_button(|b| b.custom_id("prev").label("⏪ Prev").style(ButtonStyle::Primary))
       .create_button(|b| b.custom_id("next").label("Next ⏩").style(ButtonStyle::Primary))
}
