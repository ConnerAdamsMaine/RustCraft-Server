use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};

use anyhow::Result;
use parking_lot::RwLock;
use tracing::{debug, info, trace, warn};

use crate::chunk::cache::LruCache;
use crate::consts::{
    CHUNK_SIZE_BYTES,
    INITIAL_BUFFER_MB,
    INITIAL_CAPACITY,
    MAX_BUFFER_MB,
    MAX_CAPACITY,
    WORLD_PATH,
};
use crate::core::thread_pool::ChunkGenThreadPool;
use crate::terrain::{Chunk, ChunkGenerator, ChunkPos};
use crate::world::{Region, RegionPos};

// Memory budget constants
// const CHUNK_SIZE_BYTES: usize = 232 * 1024; // ~232 KB per chunk
// const INITIAL_BUFFER_MB: usize = 256; // 256 MB initial
// const MAX_BUFFER_MB: usize = 2048; // 2 GB max
//
// const INITIAL_CAPACITY: usize = INITIAL_BUFFER_MB * 1024 * 1024 / CHUNK_SIZE_BYTES; // ~1130 chunks
// const MAX_CAPACITY: usize = MAX_BUFFER_MB * 1024 * 1024 / CHUNK_SIZE_BYTES; // ~9033 chunks

#[allow(dead_code)]
pub struct CacheLenCapacity((usize, usize));

impl From<(usize, usize)> for CacheLenCapacity {
    fn from(value: (usize, usize)) -> Self {
        CacheLenCapacity(value)
    }
}

pub struct ChunkStorage {
    // PERF: @locking : Is there a way to work around the use of a RwLock here?
    cache:           Arc<RwLock<LruCache<ChunkPos, Chunk>>>,
    world_dir:       PathBuf,
    chunk_generator: Arc<ChunkGenerator>,
    // PERF: @atomics : Could we use an atomic counter here instead of RwLock?
    evictions:       Arc<RwLock<usize>>,
    chunk_gen_pool:  Arc<ChunkGenThreadPool>,
}

impl ChunkStorage {
    pub fn new(
        chunk_generator: Arc<ChunkGenerator>,
        chunk_gen_pool: Arc<ChunkGenThreadPool>,
    ) -> Result<Self> {
        // let world_dir = PathBuf::from(WORLD_NAME);
        let world_dir = PathBuf::from(WORLD_PATH);

        // Create world directory if it doesn't exist
        if !world_dir.exists() {
            fs::create_dir_all(&world_dir)?;
        }

        info!(
            "[STARTUP] Initializing chunk cache: {}-{}MB ({}-{} chunks)",
            INITIAL_BUFFER_MB, MAX_BUFFER_MB, INITIAL_CAPACITY, MAX_CAPACITY
        );

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
        debug!("[STARTUP] Starting pregeneration of spawn area...");
        storage.pregenerate_spawn_area()?;

        Ok(storage)
    }

    pub fn start_hit_reset_task(&self) {
        let cache = Arc::clone(&self.cache);
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
        info!("[STARTUP] Pregenerating spawn area (16x16 chunks)...");

        let start = std::time::Instant::now();
        let mut generated = 0;
        let (tx, rx) = mpsc::channel();

        // Generate a 16x16 area centered around origin using thread pool
        for cx in -8..8 {
            for cz in -8..8 {
                let chunk_pos = ChunkPos::new(cx, cz);

                // Check if chunk exists on disk
                let region_pos = RegionPos::from(chunk_pos);

                // if !self.chunk_exists_on_disk(region_pos)? {
                if !self.world_dir.join(region_pos.filename()).exists() {
                    // Clone needed data for thread pool task
                    let generator = Arc::clone(&self.chunk_generator);
                    let tx = tx.clone();

                    // Submit to thread pool
                    self.chunk_gen_pool.execute(move || {
                        let chunk = generator.generate(chunk_pos);
                        let _ = tx.send((chunk_pos, chunk));
                    })?;

                    generated += 1;

                    // Periodically receive and cache generated chunks
                    if generated % 256 == 0 {
                        trace!("[CHUNK] Submitted {} chunks to generation pool", generated);
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
            debug!("[CHUNK] Caching pregenerated chunk at {}", pos);
            let (_, expanded, evicted) = {
                let mut cache = self.cache.write();
                cache.insert(pos, chunk)
            };

            if expanded {
                let cache = self.cache.read();
                debug!("[CHUNK] Cache expanded to {} chunks during pregeneration", cache.current_capacity());
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
                debug!("[CHUNK] Cache expanded to {} chunks during pregeneration", cache.current_capacity());
            }

            if let Some(evicted_pos) = evicted {
                debug!("[CHUNK] Evicted {} during pregeneration", evicted_pos);
            }
        }
        Ok(())
    }

    pub fn get_chunk(&self, chunk_pos: ChunkPos) -> Result<Chunk> {
        // Check cache first
        {
            let mut cache = self.cache.write();
            if let Some(chunk) = cache.get(&chunk_pos) {
                debug!("[CHUNK] Cache hit for {}", chunk_pos);
                return Ok(chunk.clone());
            }
        }

        // Try to load from disk
        if let Ok(chunk) = self.load_chunk_from_disk(chunk_pos) {
            debug!("[CHUNK] Loaded chunk {} from disk", chunk_pos);
            self.cache.write().insert(chunk_pos, chunk.clone());
            return Ok(chunk);
        }

        // Generate new chunk
        debug!("[CHUNK] Generating new chunk at {}", chunk_pos);
        let chunk = self.chunk_generator.generate(chunk_pos);
        self.cache.write().insert(chunk_pos, chunk.clone());

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
            info!("[CHUNK] Cache expanded to {} chunks ({:.1}% usage)", capacity, usage * 100.0);
        }

        if let Some(evicted_pos) = evicted_key {
            let mut evictions = self.evictions.write();
            *evictions += 1;
            warn!("[CHUNK] Evicted low-hit chunk {} (total evictions: {})", evicted_pos, *evictions);
        }

        // If cache is getting full, flush to disk
        let cache = self.cache.read();
        if cache.len() > cache.current_capacity() / 2 {
            warn!("[CHUNK] Cache over 50% full, flushing to disk...");
            drop(cache);
            self.flush_cache()?;
        }

        Ok(())
    }

    pub fn flush_cache(&self) -> Result<()> {
        warn!("[CHUNK] Flushing all cached chunks to disk...");

        let chunks_to_save: Vec<Chunk> = self
            .cache
            .write()
            .iter()
            .map(|(_, chunk)| chunk.clone())
            .collect();

        let region_positions: Vec<RegionPos> = chunks_to_save
            .iter()
            .map(|chunk| RegionPos::from_chunk(chunk.pos.x, chunk.pos.z))
            .collect();

        let start = std::time::Instant::now();

        for chunk in chunks_to_save {
            let region_pos = RegionPos::from_chunk(chunk.pos.x, chunk.pos.z); // refactor - temp 
            self.save_chunk_to_disk_vectored(region_pos, chunk)?;
        }

        let end = std::time::Instant::now();
        let duration = end.duration_since(start);
        info!(
            "[CHUNK] Flushed {} chunks to disk in {:.2}s ({:.00} chunks/sec)",
            region_positions.len(),
            duration.as_secs_f64(),
            region_positions.len() as f64 / duration.as_secs_f64()
        );

        Ok(())
    }

    fn load_chunk_from_disk(&self, chunk_pos: ChunkPos) -> Result<Chunk> {
        let region_pos = RegionPos::from_chunk(chunk_pos.x, chunk_pos.z);
        let region_path = self.world_dir.join(region_pos.filename());

        if !region_path.exists() {
            return Err(anyhow::anyhow!("Region file not found"));
        }

        let data = fs::read(&region_path)?;
        let region = Region::deserialize(&data)?;

        region
            .get(chunk_pos.x, chunk_pos.z)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Chunk not found in region"))
    }

    fn save_chunk_to_disk_vectored(&self, region_pos: RegionPos, chunk: Chunk) -> Result<()> {
        // let region_pos = RegionPos::from_chunk(chunk_pos.x, chunk_pos.z);

        if !region_pos.is_valid() {
            warn!("Chunk ({:?}, {:?}) is outside valid world bounds", region_pos.x, region_pos.z);
            return Ok(());
        }

        let region_path = self.world_dir.join(region_pos.filename());

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

        debug!("Saved chunk ({:?}, {:?}) to {:?}", region_pos.x, region_pos.z, region_path.canonicalize()?);

        Ok(())
    }

    // fn get_region_path(&self, region_pos: RegionPos) -> PathBuf {
    //     self.world_dir.join(region_pos.filename())
    // }

    pub fn cache_stats(&self) -> CacheLenCapacity {
        CacheLenCapacity::from((self.cache.read().len(), self.cache.read().current_capacity()))
    }

    // fn save_chunk_to_disk(&self, chunk_pos: ChunkPos, chunk: Chunk) -> Result<()> {
    //     let region_pos = RegionPos::from_chunk(chunk_pos.x, chunk_pos.z);
    //
    //     if !region_pos.is_valid() {
    //         warn!("Chunk {:?} is outside valid world bounds", chunk_pos);
    //         return Ok(());
    //     }
    //
    //     let region_path = self.get_region_path(chunk_pos);
    //
    //     // Load existing region or create new one
    //     let mut region = if region_path.exists() {
    //         let data = fs::read(&region_path)?;
    //         Region::deserialize(&data)?
    //     } else {
    //         Region::new(region_pos)
    //     };
    //
    //     region.insert(chunk.clone());
    //
    //     let serialized = region.serialize();
    //     fs::write(&region_path, serialized)?;
    //
    //     debug!("Saved chunk {:?} to {}", chunk_pos, region_path.display());
    //
    //     Ok(())
    // }
}

impl Clone for ChunkStorage {
    fn clone(&self) -> Self {
        Self {
            cache:           self.cache.clone(),
            world_dir:       self.world_dir.clone(),
            chunk_generator: self.chunk_generator.clone(),
            evictions:       self.evictions.clone(),
            chunk_gen_pool:  self.chunk_gen_pool.clone(),
        }
    }
}
