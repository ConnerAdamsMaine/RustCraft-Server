use std::fmt::{Debug, Display};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, info};

use crate::chunk::ChunkStorage;
use crate::consts::{CHUNK_SEED, GAMELOOP_SLEEP_TICK, WORLD_PATH};
use crate::core::game_loop::GameLoop;
use crate::core::thread_pool::ChunkGenThreadPool;
use crate::error_tracker::{ErrorKey, ErrorTracker};
use crate::player::PlayerData;
use crate::terrain::ChunkGenerator;

pub struct MinecraftServer {
    listener:       TcpListener,
    game_loop:      Arc<tokio::sync::Mutex<GameLoop>>,
    chunk_storage:  ChunkStorage,
    error_tracker:  Arc<ErrorTracker>,
    chunk_gen_pool: Arc<ChunkGenThreadPool>,
}

impl MinecraftServer {
    pub async fn new<A>(addr: A, error_tracker: Arc<ErrorTracker>) -> Result<Self>
    where
        A: ToSocketAddrs + Display + Debug,
    {
        let listener = TcpListener::bind(&addr).await?;
        info!("[STARTUP] Server listening on {}", addr);

        // Initialize thread pools
        let chunk_gen_pool = Arc::new(ChunkGenThreadPool::new());

        // Create chunk generator and storage with the pool
        let chunk_gen = Arc::new(ChunkGenerator::new::<u64>(CHUNK_SEED));
        let chunk_storage = ChunkStorage::new(chunk_gen, chunk_gen_pool.clone())?;

        Ok(Self {
            listener,
            game_loop: Arc::new(tokio::sync::Mutex::new(GameLoop::new())),
            chunk_storage,
            error_tracker,
            chunk_gen_pool,
        })
    }

    pub async fn run(self) -> Result<()> {
        let game_loop = self.game_loop.clone();
        let chunk_storage = Arc::new(self.chunk_storage.clone());
        let error_tracker = self.error_tracker.clone();
        let chunk_gen_pool = self.chunk_gen_pool.clone();
        // let packet_logger = self.packet_logger.as_ref();

        // Start hit count reset task (runs every 5 minutes)
        self.chunk_storage.start_hit_reset_task();

        // Realistically; this should never happen due to generation
        // running on the constructor of `MinecraftServer::new()`, which itself constructs
        // a `ChunkStorage` that initializes the world folder.
        // The path (until a later refactor) is defined in `consts::WORLD_PATH`.
        if !Path::new(&WORLD_PATH).exists() {
            error!("[STARTUP] World directory does not exist after initialization!");
            error!(
                "[STARTUP] This should never happen unless you've deleted the world folder while the server is setting up."
            );
            error!("What have you done???");
            panic!();
        }

        self.chunk_gen_pool.signal_init_complete();

        // Spawn game loop task (main thread for game loop and logging)
        tokio::spawn(async move {
            loop {
                let mut gl = game_loop.lock().await;
                if let Err(e) = gl.tick() {
                    error!("[GAMELOOP] Tick failed: {}", e);
                }
                drop(gl);
                tokio::time::sleep(tokio::time::Duration::from_millis(GAMELOOP_SLEEP_TICK)).await;
            }
        });

        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    info!("[CONNECTION] New connection from {}", addr);
                    let chunk_storage = Arc::clone(&chunk_storage);
                    let error_tracker = error_tracker.clone();
                    let chunk_gen_pool = chunk_gen_pool.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(
                            //
                            socket,
                            Arc::clone(&chunk_storage).as_ref().clone(),
                            error_tracker,
                            chunk_gen_pool,
                        )
                        .await
                        {
                            error!("[CLIENT] Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("[NETWORK] Accept error: {}", e);
                    let key = ErrorKey::new("NETWORK", "accept_failed");
                    if self.error_tracker.record_error(key) {
                        error!("[SHUTDOWN] Initiating safe shutdown due to critical errors");
                        return Ok(());
                    }
                }
            }
        }
    }
}

async fn handle_client(
    socket: TcpStream,
    chunk_storage: ChunkStorage,
    error_tracker: Arc<ErrorTracker>,
    chunk_gen_pool: Arc<ChunkGenThreadPool>,
) -> Result<()> {
    let player = PlayerData::new(socket).await?;
    player
        .handle(chunk_storage, error_tracker, chunk_gen_pool)
        .await?;
    Ok(())
}
