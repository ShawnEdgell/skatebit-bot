use skatebit_bot::run;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    tracing::info!("Bot process starting...");

    if let Err(e) = run().await {
        tracing::error!(error.cause_chain = ?e, error.message = %e, "Error running bot");
        std::process::exit(1);
    }
    Ok(())
}