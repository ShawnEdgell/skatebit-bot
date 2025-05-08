use serenity::model::application::{
    CommandInteraction,
    CommandDataOption,
    CommandDataOptionValue,
};

pub fn get_str_option<'a>(
    interaction: &'a CommandInteraction,
    name: &str,
) -> Option<&'a str> {
    interaction
        .data
        .options
        .iter()
        .find_map(|opt: &CommandDataOption| {
            if opt.name == name {
                if let CommandDataOptionValue::String(s_val) = &opt.value {
                    Some(s_val.as_str())
                } else {
                    None
                }
            } else {
                None
            }
        })
}