use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::debug;

use crate::network::protocol::{
    read_varint,
    write_varint,
    ByteWritable,
    NBTBuilder,
    PacketReader,
    PacketWriter,
};

pub struct ConfigurationHandler;

impl ConfigurationHandler {
    /// Handle the Configuration phase after login
    /// Sends required registry data and finish configuration packet
    pub async fn handle_configuration(stream: &mut TcpStream) -> Result<()> {
        debug!("[CONFIG] Starting configuration phase");

        // Send required Registry Data packets
        // These define the game registries that client and server must agree on
        Self::send_registry_data(stream).await?;

        debug!("[CONFIG] Sending Finish Configuration");
        Self::send_finish_configuration(stream).await?;

        debug!("[CONFIG] Waiting for Acknowledge Finish Configuration");
        Self::read_acknowledge_finish_configuration(stream).await?;

        debug!("[CONFIG] Configuration phase complete");
        Ok(())
    }

    /// Send Registry Data packets for critical registries
    /// Registry Data packet structure (Protocol ID: 0x07 in Configuration state):
    /// - Registry ID (Identifier): The registry name (e.g., "minecraft:dimension_type")
    /// - Entries (Prefixed Array):
    ///   - Entry ID (Identifier): The entry name (e.g., "minecraft:overworld")
    ///   - Data (Prefixed Optional NBT): Entry data in NBT format (or null if from known packs)
    /// 
    /// LOCATION OF PROBLEM
    /// FIND FIX
    async fn send_registry_data(stream: &mut TcpStream) -> Result<()> {
        // Send minimal required registries for basic functionality
        // For a full server, you'd need to send ALL synchronized registries
        let registries = vec![
            ("minecraft:dimension_type", Self::get_dimension_type_registry()), // Problem child #1
            ("minecraft:damage_type", Self::get_damage_type_registry()), // Problem child #2
        ];

        for (registry_id, entries) in registries { // Problem child #3
            Self::send_single_registry(stream, registry_id, &entries).await?;
        }

        debug!("[CONFIG] Registry Data packets sent");
        Ok(())
    }

    /// Send a single Registry Data packet
    /// 
    /// Problem child #4
    async fn send_single_registry(
        stream: &mut TcpStream,
        registry_id: &str,
        entries: &[(String, Vec<u8>)],
    ) -> Result<()> {
        let mut writer = PacketWriter::new();

        for (entry_id, nbt_data) in entries {
            // Debug logging
            tracing::debug!("[CONFIG] Entry ID: {:?}", entry_id);
            tracing::debug!("[CONFIG] Entry ID bytes: {:?}", entry_id.as_bytes());
            tracing::debug!("[CONFIG] Entry ID len: {}", entry_id.len());
            // Write Entry ID (as an Identifier)
            writer.write_string(entry_id);
        }

        // Write Registry ID (as an Identifier)
        writer.write_string(registry_id);

        // Write Entries array
        writer.write_varint(entries.len() as i32); // Array length

        for (entry_id, nbt_data) in entries {
            // Write Entry ID (as an Identifier)
            writer.write_string(entry_id);

            // Write Data (Prefixed Optional NBT)
            // Write length followed by the NBT data
            writer.write_varint(nbt_data.len() as i32);
            writer.write_bytes(nbt_data);
        }

        let packet_data = writer.finish();
        let packet_id = write_varint(0x07); // Registry Data packet ID

        // Write packet: [length][id][data]
        let mut frame = Vec::new();
        frame.extend_from_slice(&write_varint((packet_id.len() + packet_data.len()) as i32));
        frame.extend_from_slice(&packet_id);
        frame.extend_from_slice(&packet_data);

        stream.write_all(&frame).await?;
        stream.flush().await?;
        debug!("[CONFIG] Sent registry data for: {} ({} entries)", registry_id, entries.len());

        Ok(())
    }

    /// Get the dimension_type registry entries with proper NBT data
    fn get_dimension_type_registry() -> Vec<(String, Vec<u8>)> {
        vec![
            (
                "minecraft:overworld".to_string(),
                NBTBuilder::dimension_compound("overworld", 384, -64, true, false, false, true, 1.0),
            ),
            (
                "minecraft:the_nether".to_string(),
                NBTBuilder::dimension_compound("the_nether", 256, 0, false, true, true, false, 8.0),
            ),
            (
                "minecraft:the_end".to_string(),
                NBTBuilder::dimension_compound("the_end", 256, 0, false, false, false, false, 1.0),
            ),
        ]
    }

    /// Get the damage_type registry entries with proper NBT data
    fn get_damage_type_registry() -> Vec<(String, Vec<u8>)> {
        vec![
            (
                "minecraft:generic".to_string(),
                NBTBuilder::damage_type_compound("generic", "when_caused_by_living_non_player", 0.0),
            ),
            (
                "minecraft:player_attack".to_string(),
                NBTBuilder::damage_type_compound("player_attack", "when_caused_by_living_non_player", 0.1),
            ),
            (
                "minecraft:player_knockback".to_string(),
                NBTBuilder::damage_type_compound("player_knockback", "when_caused_by_living_non_player", 0.1),
            ),
            (
                "minecraft:world_border".to_string(),
                NBTBuilder::damage_type_compound("world_border", "always", 0.0),
            ),
            (
                "minecraft:falling".to_string(),
                NBTBuilder::damage_type_compound("falling", "when_caused_by_living_non_player", 0.1),
            ),
            (
                "minecraft:suffocation".to_string(),
                NBTBuilder::damage_type_compound("suffocation", "always", 0.0),
            ),
            ("minecraft:drowning".to_string(), NBTBuilder::damage_type_compound("drowning", "always", 0.0)),
            ("minecraft:starving".to_string(), NBTBuilder::damage_type_compound("starving", "always", 0.0)),
            (
                "minecraft:falling_anvil".to_string(),
                NBTBuilder::damage_type_compound("falling_anvil", "when_caused_by_living_non_player", 0.1),
            ),
        ]
    }

    async fn send_finish_configuration(stream: &mut TcpStream) -> Result<()> {
        // Finish Configuration packet (0x03 in Configuration state)
        let packet_id = write_varint(0x03);

        // This packet has no payload, just packet ID
        let mut frame = Vec::new();
        frame.extend_from_slice(&write_varint(packet_id.len() as i32));
        frame.extend_from_slice(&packet_id);

        stream.write_all(&frame).await?;
        stream.flush().await?;

        debug!("[CONFIG] Finish Configuration sent");
        Ok(())
    }

    async fn read_acknowledge_finish_configuration(stream: &mut TcpStream) -> Result<()> {
        // Client may send optional packets before Acknowledge Finish Configuration
        // Valid packets in Configuration state (serverbound):
        // 0x00 = Client Information
        // 0x01 = Serverbound Plugin Message
        // 0x02 = Serverbound Known Packs
        // 0x03 = Acknowledge Finish Configuration

        loop {
            let mut length_buf = [0u8; 5];

            // Read packet length
            let mut bytes_read = 0;
            loop {
                let n = stream.read(&mut length_buf[bytes_read..bytes_read + 1]).await?;
                if n == 0 {
                    return Err(anyhow!("Connection closed during acknowledge finish configuration"));
                }

                if length_buf[bytes_read] & 0x80 == 0 {
                    bytes_read += 1;
                    break;
                }
                bytes_read += 1;
                if bytes_read >= 5 {
                    return Err(anyhow!("Packet length too long"));
                }
            }

            let packet_length = read_varint(&mut std::io::Cursor::new(&length_buf[..bytes_read]))? as usize;

            // Read packet data
            let mut packet_data = vec![0u8; packet_length];
            stream.read_exact(&mut packet_data).await?;

            let mut reader = PacketReader::new(&packet_data);
            let packet_id = reader.read_varint()?;

            match packet_id {
                0x00 => {
                    // Client Information - optional, skip it
                    debug!("[CONFIG] Received Client Information (0x00)");
                }
                0x01 => {
                    // Serverbound Plugin Message - optional, skip it
                    debug!("[CONFIG] Received Serverbound Plugin Message (0x01)");
                }
                0x02 => {
                    // Serverbound Known Packs - optional, skip it
                    debug!("[CONFIG] Received Serverbound Known Packs (0x02)");
                }
                0x03 => {
                    // Acknowledge Finish Configuration - this is what we're waiting for
                    debug!("[CONFIG] Acknowledge Finish Configuration received");
                    return Ok(());
                }
                _ => {
                    return Err(anyhow!("Unexpected packet in Configuration state: {:#x}", packet_id));
                }
            }
        }
    }
}
