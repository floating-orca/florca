use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::from_filename(".env.local").ok();
    dotenvy::dotenv().ok();

    configure_tracing();

    let state = florca_engine::init().await?;

    florca_engine::serve(state).await?;

    Ok(())
}

fn configure_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("INFO")),
        )
        .init();
}
