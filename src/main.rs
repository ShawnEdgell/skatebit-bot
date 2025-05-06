// main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    skatebit_bot::start().await
}