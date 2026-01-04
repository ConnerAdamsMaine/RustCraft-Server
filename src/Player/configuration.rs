use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::debug;

use crate::network::protocol::{read_varint, write_varint, ByteWritable, PacketReader, PacketWriter};

pub struct ConfigurationHandler;

impl ConfigurationHandler {
    /// Handle the Configuration phase after login
    /// Sends required registry data and finish configuration packet
    pub async fn handle_configuration(stream: &mut TcpStream) -> Result<()> {
        debug!("[CONFIG] Starting configuration phase");

        // Send Registry Data packet - minimal required registries
        // For 1.21, we need at least: damage_type, biome, dimension_type, trim_material, trim_pattern
        Self::send_registry_data(stream).await?;

        debug!("[CONFIG] Sending Finish Configuration");
        Self::send_finish_configuration(stream).await?;

        debug!("[CONFIG] Waiting for Acknowledge Finish Configuration");
        Self::read_acknowledge_finish_configuration(stream).await?;

        debug!("[CONFIG] Configuration phase complete");
        Ok(())
    }

    async fn send_registry_data(stream: &mut TcpStream) -> Result<()> {
        // Registry Data packet (0x07 in Configuration state)
        // This is a complex packet with NBT data
        // For now, send a minimal version with empty registries
        
        let mut writer = PacketWriter::new();
        
        // Registry ID (string)
        writer.write_string("minecraft:root");
        
        // Has entries (boolean) - true
        writer.write_bool(true);
        
        // Count (varint) - 0 for empty
        writer.write_varint(0);
        
        let packet_data = writer.finish();
        let packet_id = write_varint(0x07);

        let mut frame = Vec::new();
        frame.extend_from_slice(&write_varint((packet_id.len() + packet_data.len()) as i32));
        frame.extend_from_slice(&packet_id);
        frame.extend_from_slice(&packet_data);

        stream.write_all(&frame).await?;
        stream.flush().await?;

        debug!("[CONFIG] Registry Data sent");
        Ok(())
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
        let mut length_buf = [0u8; 5];

        // Read packet length
        let mut bytes_read = 0;
        loop {
            let n = stream
                .read(&mut length_buf[bytes_read..bytes_read + 1])
                .await?;
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

        let packet_length =
            read_varint(&mut std::io::Cursor::new(&length_buf[..bytes_read]))? as usize;

        // Read packet data
        let mut packet_data = vec![0u8; packet_length];
        stream.read_exact(&mut packet_data).await?;

        let mut reader = PacketReader::new(&packet_data);
        let packet_id = reader.read_varint()?;

        // Acknowledge Finish Configuration should be 0x03 (serverbound)
        if packet_id != 0x03 {
            return Err(anyhow!(
                "Expected Acknowledge Finish Configuration packet (0x03), got {:#x}",
                packet_id
            ));
        }

        debug!("[CONFIG] Acknowledge Finish Configuration received");
        Ok(())
    }
}
