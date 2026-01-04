use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use tracing::debug;

pub struct PacketLogger {
    packet_dir: PathBuf,
    counter:    AtomicUsize,
}

impl PacketLogger {
    pub fn new() -> Result<Self> {
        let packet_dir = PathBuf::from("packets");

        // Create packets directory if it doesn't exist
        if !packet_dir.exists() {
            fs::create_dir_all(&packet_dir)?;
        }

        // Clear old packets on startup
        if packet_dir.exists() {
            for entry in fs::read_dir(&packet_dir)?.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "bin") {
                    let _ = fs::remove_file(path);
                }
            }
        }

        tracing::info!("[PACKET_LOG] Logger initialized, packets directory ready");

        Ok(Self {
            packet_dir,
            counter: AtomicUsize::new(0),
        })
    }

    pub fn log_client_packet(&self, data: &[u8]) -> Result<()> {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        let filename = format!("{:06}_client.bin", count);
        let path = self.packet_dir.join(&filename);

        fs::write(&path, data)?;
        debug!("[PACKET_LOG] Logged client packet: {} ({} bytes)", filename, data.len());

        Ok(())
    }

    pub fn log_server_packet(&self, data: &[u8]) -> Result<()> {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        let filename = format!("{:06}_server.bin", count);
        let path = self.packet_dir.join(&filename);

        fs::write(&path, data)?;
        debug!("[PACKET_LOG] Logged server packet: {} ({} bytes)", filename, data.len());

        Ok(())
    }
}

impl AsRef<PacketLogger> for PacketLogger {
    fn as_ref(&self) -> &PacketLogger {
        self
    }
}

impl Clone for PacketLogger {
    fn clone(&self) -> Self {
        let counter = AtomicUsize::new(self.counter.load(Ordering::SeqCst));
        Self {
            packet_dir: self.packet_dir.clone(),
            counter,
        }
    }
}

impl Default for PacketLogger {
    fn default() -> Self {
        Self::new().expect("Failed to initialize PacketLogger")
    }
}
