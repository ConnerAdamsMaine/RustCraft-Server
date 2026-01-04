mod cache;
mod chunk;
mod chunk_generator;
mod chunk_protocol;
mod chunk_sender;
mod chunk_storage;
mod error_tracker;
mod game_loop;
mod join_game;
mod login;
mod movement_handler;
mod noise;
mod packet_logger;
mod player;
mod protocol;
mod region;
mod server;
mod terrain_gen;
mod thread_pool;
mod world;

use std::sync::Arc;

use anyhow::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with a custom format
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .compact()
        .init();

    let error_tracker = Arc::new(error_tracker::ErrorTracker::new());

    // Start the Minecraft server
    let server = server::MinecraftServer::new("127.0.0.1:25565", error_tracker.clone()).await?;
    server.run().await?;

    Ok(())
}
