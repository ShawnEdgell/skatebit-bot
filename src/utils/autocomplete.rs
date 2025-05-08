use serenity::{
    builder::{
        CreateAutocompleteResponse,
        CreateInteractionResponse,
    },
    client::Context,
    model::application::{
        CommandInteraction,
        CommandDataOptionValue,
    },
    all::AutocompleteChoice,
};
use std::{error::Error, future::Future};

pub async fn basic_autocomplete<F, Fut>(
    ctx: &Context,
    interaction: &CommandInteraction,
    option_name: &str,
    handler: F,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    F: FnOnce(&str) -> Fut,
    Fut: Future<Output = Result<Vec<(String, String)>, Box<dyn Error + Send + Sync>>>,
{
    let current_input = interaction
        .data
        .options
        .iter()
        .find(|opt| opt.name == option_name)
        .and_then(|opt| {
            if let CommandDataOptionValue::String(s) = &opt.value {
                Some(s.as_str())
            } else { None }
        })
        .unwrap_or("");

    let suggestions = handler(current_input).await?;

    let mut choices_vec = Vec::new();

    for (name, value) in suggestions {
        if choices_vec.len() < 25 {
            choices_vec.push(AutocompleteChoice::new(name, value));
        } else {
            eprintln!("Warning: Truncated autocomplete choices exceeding limit of 25.");
            break;
        }
    }

    let response_builder = CreateAutocompleteResponse::new().set_choices(choices_vec);

    let full_response = CreateInteractionResponse::Autocomplete(response_builder);

    interaction
        .create_response(&ctx.http, full_response)
        .await?;

    Ok(())
}