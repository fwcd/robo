use anyhow::Result;
use tracing::info;

fn bootstrap_tracing() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    bootstrap_tracing()?;

    info!("Hello, world!");

    Ok(())
}
