use crate::cache::LruCache;
use crate::chunk::{Chunk, ChunkPos};
use crate::chunk_generator::ChunkGenerator;
use crate::region::{Region, RegionPos};
use crate::thread_pool::ChunkGenThreadPool;
use anyhow::Result;
use parking_lot::RwLock;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use tracing::{debug, info, warn};

const WORLD_DIR: &str = "world";

// Memory budget constants
const CHUNK_SIZE_BYTES: usize = 232 * 1024; // ~232 KB per chunk
const INITIAL_BUFFER_MB: usize = 256; // 256 MB initial
const MAX_BUFFER_MB: usize = 2048; // 2 GB max

const INITIAL_CAPACITY: usize = INITIAL_BUFFER_MB * 1024 * 1024 / CHUNK_SIZE_BYTES; // ~1130 chunks
const MAX_CAPACITY: usize = MAX_BUFFER_MB * 1024 * 1024 / CHUNK_SIZE_BYTES; // ~9033 chunks

pub struct ChunkStorage {
    cache: Arc<RwLock<LruCache<ChunkPos, Chunk>>>,
    world_dir: PathBuf,
    chunk_generator: Arc<ChunkGenerator>,
    evictions: Arc<RwLock<usize>>,
    chunk_gen_pool: Arc<ChunkGenThreadPool>,
}

impl ChunkStorage {
    pub fn new(chunk_generator: Arc<ChunkGenerator>) -> Result<Self> {
        let world_dir = PathBuf::from(WORLD_DIR);

        // Create world directory if it doesn't exist
        if !world_dir.exists() {
            fs::create_dir_all(&world_dir)?;
        }

        info!(
            "[STARTUP] Initializing chunk cache: {}-{}MB ({}-{} chunks)",
            INITIAL_BUFFER_MB, MAX_BUFFER_MB, INITIAL_CAPACITY, MAX_CAPACITY
        );

        let chunk_gen_pool = Arc::new(ChunkGenThreadPool::new());

        let storage = Self {
            cache: Arc::new(RwLock::new(LruCache::with_growth(
                INITIAL_CAPACITY,
                MAX_CAPACITY,
                CHUNK_SIZE_BYTES,
            ))),
            world_dir,
            chunk_generator,
            evictions: Arc::new(RwLock::new(0)),
            chunk_gen_pool,
        };

        // Pregenerate 64x64 chunk area on startup
        storage.pregenerate_spawn_area()?;

        Ok(storage)
    }

    pub fn start_hit_reset_task(&self) {
        let cache = self.cache.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await; // 5 minutes
                let mut cache_lock = cache.write();
                cache_lock.reset_hit_counts();
                drop(cache_lock);
                debug!("[CHUNK] Hit counts reset");
            }
        });
    }

    fn pregenerate_spawn_area(&self) -> Result<()> {
        info!("[STARTUP] Pregenerating spawn area (64x64 chunks)...");

        let start = std::time::Instant::now();
        let mut generated = 0;
        let (tx, rx) = mpsc::channel();

        // Generate a 64x64 area centered around origin using thread pool
        for cx in -32..32 {
            for cz in -32..32 {
                let pos = ChunkPos::new(cx, cz);

                // Check if chunk exists on disk
                if !self.chunk_exists_on_disk(pos)? {
                    // Clone needed data for thread pool task
                    let generator = Arc::clone(&self.chunk_generator);
                    let tx = tx.clone();

                    // Submit to thread pool
                    self.chunk_gen_pool.execute(move || {
                        let chunk = generator.generate(pos);
                        let _ = tx.send((pos, chunk));
                    })?;

                    generated += 1;

                    // Periodically receive and cache generated chunks
                    if generated % 256 == 0 {
                        debug!("[CHUNK] Submitted {} chunks to generation pool", generated);
                        self.receive_and_cache_chunks(&rx)?;
                    }
                }
            }
        }

        // Drop the original sender so receiver knows when all tasks are done
        drop(tx);

        // Receive all remaining chunks
        self.receive_and_cache_all_chunks(&rx)?;

        self.flush_cache()?;

        let elapsed = start.elapsed();
        let cache = self.cache.read();
        info!(
            "[STARTUP] Pregeneration complete: {} new chunks in {:.2}s ({:.0} chunks/sec), cache: {}/{}",
            generated,
            elapsed.as_secs_f64(),
            generated as f64 / elapsed.as_secs_f64(),
            cache.len(),
            cache.current_capacity()
        );

        Ok(())
    }

    fn receive_and_cache_chunks(&self, rx: &mpsc::Receiver<(ChunkPos, Chunk)>) -> Result<()> {
        // Receive chunks with a short timeout to avoid blocking
        while let Ok((pos, chunk)) = rx.try_recv() {
            let (_, expanded, evicted) = {
                let mut cache = self.cache.write();
                cache.insert(pos, chunk)
            };

            if expanded {
                let cache = self.cache.read();
                debug!(
                    "[CHUNK] Cache expanded to {} chunks during pregeneration",
                    cache.current_capacity()
                );
            }

            if let Some(evicted_pos) = evicted {
                debug!("[CHUNK] Evicted {} during pregeneration", evicted_pos);
            }
        }
        Ok(())
    }

    fn receive_and_cache_all_chunks(&self, rx: &mpsc::Receiver<(ChunkPos, Chunk)>) -> Result<()> {
        // Receive all remaining chunks from the channel
        while let Ok((pos, chunk)) = rx.recv() {
            let (_, expanded, evicted) = {
                let mut cache = self.cache.write();
                cache.insert(pos, chunk)
            };

            if expanded {
                let cache = self.cache.read();
                debug!(
                    "[CHUNK] Cache expanded to {} chunks during pregeneration",
                    cache.current_capacity()
                );
            }

            if let Some(evicted_pos) = evicted {
                debug!("[CHUNK] Evicted {} during pregeneration", evicted_pos);
            }
        }
        Ok(())
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Result<Chunk> {
        // Check cache first
        {
            let mut cache = self.cache.write();
            if let Some(chunk) = cache.get(&pos) {
                debug!("[CHUNK] Cache hit for {}", pos);
                return Ok(chunk.clone());
            }
        }

        // Try to load from disk
        if let Ok(chunk) = self.load_chunk_from_disk(pos) {
            debug!("[CHUNK] Loaded chunk {} from disk", pos);
            self.cache.write().insert(pos, chunk.clone());
            return Ok(chunk);
        }

        // Generate new chunk
        debug!("[CHUNK] Generating new chunk at {}", pos);
        let chunk = self.chunk_generator.generate(pos);
        self.cache.write().insert(pos, chunk.clone());

        Ok(chunk)
    }

    pub fn save_chunk(&self, chunk: Chunk) -> Result<()> {
        // Update cache
        let (_, expanded, evicted_key) = {
            let mut cache = self.cache.write();
            cache.insert(chunk.pos, chunk.clone())
        };

        if expanded {
            let cache = self.cache.read();
            let usage = cache.usage_ratio();
            let capacity = cache.current_capacity();
            drop(cache);
            info!(
                "[CHUNK] Cache expanded to {} chunks ({:.1}% usage)",
                capacity,
                usage * 100.0
            );
        }

        if let Some(evicted_pos) = evicted_key {
            let mut evictions = self.evictions.write();
            *evictions += 1;
            warn!(
                "[CHUNK] Evicted low-hit chunk {} (total evictions: {})",
                evicted_pos, *evictions
            );
        }

        // If cache is getting full, flush to disk
        let cache = self.cache.read();
        if cache.len() > cache.current_capacity() / 2 {
            drop(cache);
            self.flush_cache()?;
        }

        Ok(())
    }

    pub fn flush_cache(&self) -> Result<()> {
        let cache = self.cache.write();

        let chunks_to_save: Vec<Chunk> = cache
            .iter()
            .map(|(_, chunk)| chunk.clone())
            .collect();

        drop(cache);

        for chunk in chunks_to_save {
            self.save_chunk_to_disk(&chunk)?;
        }

        Ok(())
    }

    fn chunk_exists_on_disk(&self, pos: ChunkPos) -> Result<bool> {
        let region_path = self.get_region_path(pos);
        Ok(region_path.exists())
    }

    fn load_chunk_from_disk(&self, pos: ChunkPos) -> Result<Chunk> {
        let region_path = self.get_region_path(pos);

        if !region_path.exists() {
            return Err(anyhow::anyhow!("Region file not found"));
        }

        let data = fs::read(&region_path)?;
        let region = Region::deserialize(&data)?;

        region
            .get(pos.x, pos.z)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Chunk not found in region"))
    }

    fn save_chunk_to_disk(&self, chunk: &Chunk) -> Result<()> {
        let region_pos = RegionPos::from_chunk(chunk.pos.x, chunk.pos.z);

        if !region_pos.is_valid() {
            warn!("Chunk {:?} is outside valid world bounds", chunk.pos);
            return Ok(());
        }

        let region_path = self.get_region_path(chunk.pos);

        // Load existing region or create new one
        let mut region = if region_path.exists() {
            let data = fs::read(&region_path)?;
            Region::deserialize(&data)?
        } else {
            Region::new(region_pos)
        };

        region.insert(chunk.clone());

        let serialized = region.serialize();
        fs::write(&region_path, serialized)?;

        debug!(
            "Saved chunk {:?} to {}",
            chunk.pos,
            region_path.display()
        );

        Ok(())
    }

    fn get_region_path(&self, chunk_pos: ChunkPos) -> PathBuf {
        let region_pos = RegionPos::from_chunk(chunk_pos.x, chunk_pos.z);
        self.world_dir.join(region_pos.filename())
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read();
        (cache.len(), cache.current_capacity())
    }
}

impl Clone for ChunkStorage {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            world_dir: self.world_dir.clone(),
            chunk_generator: self.chunk_generator.clone(),
            evictions: self.evictions.clone(),
            chunk_gen_pool: self.chunk_gen_pool.clone(),
        }
    }
}
