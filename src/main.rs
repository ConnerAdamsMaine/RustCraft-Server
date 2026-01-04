use anyhow::Result;
use rustcraft::core::server::MinecraftServer;
use rustcraft::error_tracker::ErrorTracker;

#[cfg(feature = "dev-sdk")]
use crate::sdk::PacketLogger;

#[cfg(feature = "dev-sdk")]
pub static LOGGER: std::sync::LazyLock<PacketLogger> =
    std::sync::LazyLock::new(|| PacketLogger::new().expect("Failed to initialize PacketLogger"));

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with a custom format
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(true)
        .with_max_level(tracing::Level::DEBUG)
        .compact()
        .init();

    let error_tracker = std::sync::Arc::new(ErrorTracker::new());

    // Start the Minecraft server
    let server = MinecraftServer::new("127.0.0.1:25565", error_tracker.clone()).await?;
    server.run().await?;

    Ok(())
}
