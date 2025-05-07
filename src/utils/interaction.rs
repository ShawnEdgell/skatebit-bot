// src/utils/interaction.rs
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

pub fn get_str_option<'a>(command: &'a ApplicationCommandInteraction, name: &str) -> Option<&'a str> {
    command.data.options.iter()
        .find(|o| o.name == name)
        .and_then(|o| o.value.as_ref())
        .and_then(|v| v.as_str())
}