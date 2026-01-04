use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use anyhow::Result;
use tracing::debug;

pub struct PacketLogger {
    packet_dir: PathBuf,
    counter: Arc<AtomicUsize>,
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
            for entry in fs::read_dir(&packet_dir)? {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "bin") {
                        let _ = fs::remove_file(path);
                    }
                }
            }
        }
        
        tracing::info!("[PACKET_LOG] Logger initialized, packets directory ready");
        
        Ok(Self {
            packet_dir,
            counter: Arc::new(AtomicUsize::new(0)),
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

impl Clone for PacketLogger {
    fn clone(&self) -> Self {
        Self {
            packet_dir: self.packet_dir.clone(),
            counter: Arc::clone(&self.counter),
        }
    }
}

impl Default for PacketLogger {
    fn default() -> Self {
        Self::new().expect("Failed to initialize PacketLogger")
    }
}
