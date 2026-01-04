mod server;
mod protocol;
mod player;
mod world;
mod game_loop;
mod chunk;
mod chunk_generator;
mod noise;
mod terrain_gen;
mod region;
mod cache;
mod chunk_storage;
mod login;
mod join_game;
mod error_tracker;
mod thread_pool;
mod chunk_protocol;
mod chunk_sender;
mod movement_handler;

use anyhow::Result;
use tracing_subscriber;
use std::sync::Arc;

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
