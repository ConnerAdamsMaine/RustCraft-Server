use std::io::Cursor;
use std::sync::Arc;

use anyhow::Result;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::chunk::{chunk_sender, ChunkStorage};
use crate::core::thread_pool::ChunkGenThreadPool;
use crate::error_tracker::{ErrorKey, ErrorTracker};
use crate::network::protocol::read_varint;
use crate::network::LoginHandler;
use crate::player::configuration::ConfigurationHandler;
use crate::player::join_game::JoinGameHandler;
use crate::player::movement_handler;
use crate::terrain::ChunkPos;

pub struct Player {
    pub uuid:      Uuid,
    pub username:  String,
    pub socket:    TcpStream,
    pub state:     PlayerState,
    pub x:         f64,
    pub y:         f64,
    pub z:         f64,
    loaded_chunks: std::collections::HashSet<ChunkPos>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    Handshake,
    Login,
    Play,
    Idle,
}

impl Player {
    pub async fn new(socket: TcpStream) -> Result<Self> {
        Ok(Self {
            uuid: Uuid::new_v4(),
            username: String::new(),
            socket,
            state: PlayerState::Handshake,
            x: 0.0,
            y: 64.0,
            z: 0.0,
            loaded_chunks: std::collections::HashSet::new(),
        })
    }

    pub async fn handle(
        mut self,
        _chunk_storage: ChunkStorage,
        error_tracker: Arc<ErrorTracker>,
        chunk_gen_pool: Arc<ChunkGenThreadPool>,
    ) -> Result<()> {
        tracing::debug!("[PLAYER] Player handler starting");

        // Wait for world initialization to complete (in blocking task to not block async runtime)
        tracing::debug!("[PLAYER] Waiting for world initialization...");
        let chunk_gen_pool_clone = Arc::clone(&chunk_gen_pool);
        tokio::task::spawn_blocking(move || {
            chunk_gen_pool_clone.wait_for_init_complete();
            tracing::info!("[PLAYER] World initialization complete, accepting players");
        })
        .await?;

        // Handle login flow
        tracing::debug!("[PLAYER] Creating LoginHandler");
        let mut login_handler = LoginHandler::from(self.socket); // new(self.socket);

        tracing::debug!("[PLAYER] Starting login flow");
        let player_login = match login_handler.handle_login().await {
            Ok(login) => {
                tracing::debug!("[PLAYER] Login successful");
                login
            }
            Err(e) => {
                tracing::error!("[LOGIN] Authentication failed: {}", e);
                let key = ErrorKey::new("LOGIN", format!("auth_failed: {}", e));
                error_tracker.record_error(key);
                return Err(e);
            }
        };

        tracing::debug!("[PLAYER] Extracting login info");
        self.uuid = player_login.uuid;
        self.username = player_login.username.clone();
        self.socket = login_handler.get_stream();
        self.state = PlayerState::Login;
        tracing::debug!("[PLAYER] Player state set to Login (awaiting configuration)");

        tracing::info!(
            "[PLAYER] '{}' ({}) joined at (x:{}, y:{}, z:{})",
            self.username,
            self.uuid,
            self.x,
            self.y,
            self.z
        );

        // Handle Configuration phase
        tracing::debug!("[PLAYER] Starting configuration phase");
        if let Err(e) = ConfigurationHandler::handle_configuration(&mut self.socket).await {
            tracing::error!("[PLAYER] Configuration phase failed for {}: {}", self.username, e);
            let key = ErrorKey::new("CONFIG", format!("config_failed: {}", e));
            error_tracker.record_error(key);
            return Err(e);
        }
        tracing::debug!("[PLAYER] Configuration phase complete");

        // Transition to Play state
        self.state = PlayerState::Play;
        tracing::debug!("[PLAYER] Player state set to Play");

        // Send join game packet
        tracing::debug!("[PLAYER] Sending Join Game packet");
        if let Err(e) = JoinGameHandler::send_join_game(&mut self.socket, 1, &self.username).await {
            tracing::error!("[PLAYER] Failed to send join game packet to {}: {}", self.username, e);
            let key = ErrorKey::new("JOIN_GAME", "send_failed");
            error_tracker.record_error(key);
            return Err(e);
        }
        tracing::debug!("[PLAYER] Join Game sent");

        // Send player info add packet
        tracing::debug!("[PLAYER] Sending Player Info Add packet");
        if let Err(e) =
            JoinGameHandler::send_player_info_add(&mut self.socket, self.uuid, &self.username).await
        {
            tracing::error!("[PLAYER] Failed to send player info to {}: {}", self.username, e);
            let key = ErrorKey::new("PLAYER_INFO", "send_failed");
            error_tracker.record_error(key);
            return Err(e);
        }
        tracing::debug!("[PLAYER] Player Info Add sent");

        // TODO: Load initial chunks around player and send to client
        // Currently disabled - chunk serialization needs fixing for 1.21.7 protocol
        // {
        //     let socket = &mut self.socket;
        //     if let Err(e) = Self::send_chunks_around_static(socket, &self.x, &self.y, &self.z, &chunk_storage, &mut self.loaded_chunks).await {
        //         tracing::error!("[CHUNK] Failed to load initial chunks for {}: {}", self.username, e);
        //         let key = ErrorKey::new("CHUNK", "load_failed");
        //         error_tracker.record_error(key);
        //         return Err(e);
        //     }
        // }

        tracing::info!("[PLAYER] {} ready to play at ({}, {}, {})", self.username, self.x, self.y, self.z);
        tracing::debug!("[PLAYER] Starting main game loop");

        // Main game loop for this player
        loop {
            // Try to read incoming packets from client
            {
                let socket = &mut self.socket;
                // let logger = &self.packet_logger;
                match Self::handle_incoming_packets_static(
                    socket,
                    &mut self.x,
                    &mut self.y,
                    &mut self.z,
                    // logger,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("[PLAYER] {} packet read error: {}", self.username, e);
                        return Err(e);
                    }
                }
            }

            // TODO: Update loaded chunks based on player position
            // Currently disabled - chunk serialization needs fixing for 1.21.7 protocol
            // if self.check_chunk_changed(&chunk_storage).await? {
            //     // Player moved - send new chunks
            //     let socket = &mut self.socket;
            //     if let Err(e) = Self::send_chunks_around_static(socket, &self.x, &self.y, &self.z, &chunk_storage, &mut self.loaded_chunks).await {
            //         tracing::warn!("[PLAYER] Failed to send chunks to {}: {}", self.username, e);
            //     }
            // }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }

    async fn check_chunk_changed(&self, _chunk_storage: &ChunkStorage) -> Result<bool> {
        // Check if player moved to a different chunk (simplified)
        // In practice, this would track the last known chunk position
        // For now, always return false since movement comes from client packets
        Ok(false)
    }

    async fn send_chunks_around_static(
        socket: &mut TcpStream,
        x: &f64,
        _y: &f64,
        z: &f64,
        chunk_storage: &ChunkStorage,
        loaded_chunks: &mut std::collections::HashSet<ChunkPos>,
    ) -> Result<()> {
        let chunk_x = (*x / 16.0) as i32;
        let chunk_z = (*z / 16.0) as i32;

        // Load a 5x5 chunk radius around player
        for cx in (chunk_x - 2)..=(chunk_x + 2) {
            for cz in (chunk_z - 2)..=(chunk_z + 2) {
                let pos = ChunkPos::new(cx, cz);
                if !loaded_chunks.contains(&pos) {
                    match chunk_storage.get_chunk(pos) {
                        Ok(chunk) => {
                            // Send chunk to client
                            if let Err(e) = chunk_sender::send_chunk(socket, &chunk).await {
                                tracing::warn!("[CHUNK] Failed to send chunk {}: {}", pos, e);
                            } else {
                                loaded_chunks.insert(pos);
                                tracing::debug!("[CHUNK] Sent chunk {}", pos);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("[CHUNK] Failed to load chunk {}: {}", pos, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_incoming_packets_static(
        socket: &mut TcpStream,
        x: &mut f64,
        y: &mut f64,
        z: &mut f64,
        // packet_logger: &PacketLogger,
    ) -> Result<()> {
        // Read packet length
        let mut length_bytes = [0u8; 5];
        let n = socket.read(&mut length_bytes).await?;

        if n == 0 {
            // Client disconnected
            tracing::warn!("[PACKET] Client disconnected (read 0 bytes)");
            return Err(anyhow::anyhow!("Client disconnected"));
        }

        tracing::trace!("[PACKET] Read {} bytes for packet header", n);

        // Parse varint length
        let mut cursor = Cursor::new(&length_bytes[..n]);
        let packet_length = match read_varint(&mut cursor) {
            Ok(len) => {
                tracing::trace!("[PACKET] Packet length: {}", len);
                len as usize
            }
            Err(e) => {
                tracing::trace!("[PACKET] Could not parse varint: {}, trying again later", e);
                return Ok(()); // Incomplete packet, try again later
            }
        };

        // Read packet data
        let mut packet_data = vec![0u8; packet_length];
        match socket.read_exact(&mut packet_data).await {
            Ok(_) => {
                tracing::trace!("[PACKET] Read packet data ({} bytes)", packet_length);

                // Log the full packet (length + data)
                let mut full_packet = length_bytes[..n].to_vec();
                full_packet.extend_from_slice(&packet_data);
                #[cfg(feature = "dev-sdk")]
                let _ = &crate::LOGGER.log_client_packet(&full_packet);

                // Parse packet ID
                let mut cursor = Cursor::new(&packet_data[..]);
                if let Ok(packet_id) = read_varint(&mut cursor) {
                    let pos = cursor.position() as usize;
                    let payload = &packet_data[pos..];

                    tracing::trace!(
                        "[PACKET] Packet ID: 0x{:02x}, payload: {} bytes",
                        packet_id,
                        payload.len()
                    );

                    // Handle movement packets
                    if let Ok(Some(movement)) = movement_handler::parse_movement_packet(packet_id, payload) {
                        match movement {
                            movement_handler::MovementPacket::Position(pos) => {
                                *x = pos.x;
                                *y = pos.y;
                                *z = pos.z;
                                tracing::debug!("[PLAYER] moved to ({:.2}, {:.2}, {:.2})", x, y, z);
                            }
                            movement_handler::MovementPacket::PositionAndLook(pos) => {
                                *x = pos.x;
                                *y = pos.y;
                                *z = pos.z;
                                tracing::debug!("[PLAYER] moved to ({:.2}, {:.2}, {:.2})", x, y, z);
                            }
                            movement_handler::MovementPacket::Look(_) => {
                                // Handle rotation only - no position update
                            }
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available, try again later
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Client disconnected gracefully
                tracing::debug!("[PACKET] Client disconnected (unexpected EOF)");
                return Err(anyhow::anyhow!("Client disconnected"));
            }
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }
}
