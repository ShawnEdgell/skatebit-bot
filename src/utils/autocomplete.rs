use serenity::client::Context;
use serenity::model::application::interaction::autocomplete::AutocompleteInteraction;
use std::error::Error;

pub async fn basic_autocomplete<F, Fut>(
    ctx: &Context,
    inter: &AutocompleteInteraction,
    option_name: &str,
    handler: F,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    F: FnOnce(&str) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<(String, String)>, Box<dyn Error + Send + Sync>>>,
{
    let prefix = inter
        .data
        .options
        .iter()
        .find(|opt| opt.name == option_name)
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str())
        .unwrap_or("");

    let suggestions = handler(prefix).await?;

    inter.create_autocomplete_response(&ctx.http, |resp| {
        for (label, value) in suggestions {
            resp.add_string_choice(label, value);
        }
        resp
    }).await?;

    Ok(())
}
