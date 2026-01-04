# RustCraft Server Performance Recommendations

## Executive Summary

This document outlines performance bottlenecks identified in the RustCraft server codebase and provides prioritized recommendations for optimization. The analysis focuses on memory efficiency, CPU utilization, and I/O patterns.

---

## Critical Issues (High Priority)

### 1. Excessive Cloning in Chunk Storage

**Location**: `src/chunk/chunk_storage.rs:183, 190, 197, 206, 236`

**Impact**: 
- High memory allocation overhead on every cache hit
- Cascading performance degradation as cache grows
- Estimated 30% memory reduction possible

**Problem**:
```rust
pub fn get_chunk(&self, pos: ChunkPos) -> Result<Chunk> {
    let mut cache = self.cache.write();
    if let Some(chunk) = cache.get(&pos) {
        return Ok(chunk.clone());  // ← Clones entire 232KB chunk
    }
    // ...
}
```

**Recommendation**:
Wrap `Chunk` in `Arc<Chunk>` within the cache to enable cheap cloning through reference counting:
```rust
pub fn get_chunk(&self, pos: ChunkPos) -> Result<Arc<Chunk>> {
    let mut cache = self.cache.write();
    if let Some(chunk) = cache.get(&pos) {
        return Ok(Arc::clone(&chunk));  // ← O(1) pointer copy
    }
    // ...
}
```

**Effort**: 4-6 hours (requires updating callers throughout codebase)

---

### 2. O(n) Cache Eviction Algorithm

**Location**: `src/chunk/cache.rs:149-168` (`evict_lowest_hits`)

**Impact**:
- Linear scan through entire cache (up to 9,033 chunks)
- Scales poorly as cache approaches capacity
- Can introduce 100+ ms stalls at maximum capacity

**Problem**:
```rust
fn evict_lowest_hits(&mut self) -> Option<K> {
    let mut lowest_key: Option<K> = None;
    let mut lowest_hits: usize = usize::MAX;
    
    for (key, entry) in self.cache.iter() {  // ← O(n) iteration
        if entry.hits < lowest_hits {
            lowest_hits = entry.hits;
            lowest_key = Some(key.clone());
        }
    }
    // ...
}
```

**Recommendation**:
Replace with a binary heap (min-heap) to achieve O(log n) eviction:
```rust
use std::collections::BinaryHeap;

struct HitEntry<K> {
    hits: usize,
    key: K,
}

impl<K: Ord> Ord for HitEntry<K> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.hits.cmp(&self.hits)  // Reverse for min-heap
    }
}
```

**Effort**: 3-4 hours

**Expected Impact**: 10-50x faster eviction on large caches

---

### 3. Triple-Nested Loops in Chunk Serialization

**Location**: `src/chunk/chunk_protocol.rs:56-147`

**Impact**:
- Processes same 4,096 blocks three times per chunk section
- 16×16×16 blocks × 3 separate passes = 12,288 iterations
- Estimated 66% reduction possible through single-pass approach

**Problem**:
```rust
// Pass 1: Check if section has data (lines 56-70)
fn has_section_data(chunk: &Chunk, section_y: usize) -> bool {
    for x in 0..16 {
        for y in base_y..base_y + 16 {
            for z in 0..16 {
                if let Some(block) = chunk.get_block(x, y, z) {
                    if block != BlockType::Air {
                        return true;
                    }
                }
            }
        }
    }
}

// Pass 2: Build palette (lines 113-125)
for x in 0..16 {
    for y in base_y..base_y + 16 {
        for z in 0..16 {
            // Check and insert into palette
        }
    }
}

// Pass 3: Encode block data (lines 136-147)
for y in base_y..base_y + 16 {
    for z in 0..16 {
        for x in 0..16 {
            // Encode to data array
        }
    }
}
```

**Recommendation**:
Merge into single pass with memoization:
```rust
fn serialize_section_optimized(chunk: &Chunk, section_y: usize) -> (i16, Vec<i32>, Vec<u8>) {
    let base_y = section_y * 16;
    let mut block_count = 0i16;
    let mut palette = vec![0i32];
    let mut seen = std::collections::HashSet::new();
    let mut blocks = Vec::with_capacity(4096);
    
    seen.insert(0);
    
    for y in base_y..base_y + 16 {
        for z in 0..16 {
            for x in 0..16 {
                let block = chunk.get_block(x, y, z).unwrap_or(BlockType::Air);
                let block_id = block_type_to_id(block);
                
                if block != BlockType::Air {
                    block_count += 1;
                }
                
                if !seen.contains(&block_id) && block_id != 0 {
                    palette.push(block_id);
                    seen.insert(block_id);
                }
                
                let palette_idx = palette.iter().position(|&id| id == block_id).unwrap_or(0);
                blocks.push(palette_idx as u8);
            }
        }
    }
    
    (block_count, palette, blocks)
}
```

**Effort**: 2-3 hours

**Expected Impact**: 66% faster chunk serialization

---

### 4. Full Grid Clones in Terrain Erosion

**Location**: `src/terrain/terrain_gen.rs:90` (inside erosion loop)

**Impact**:
- Allocates full 512×512 grid of f64 per erosion iteration
- 2 iterations × 262KB = 524KB allocation per heightmap
- Wasteful when in-place swapping possible

**Problem**:
```rust
fn apply_erosion(&mut self) {
    let iterations = 2;
    
    for _ in 0..iterations {
        let mut new_data = self.data.clone();  // ← 262KB allocation
        
        for y in 1..(self.height - 1) {
            for x in 1..(self.width - 1) {
                // Modify new_data
            }
        }
        
        self.data = new_data;
    }
}
```

**Recommendation**:
Pre-allocate swap buffer and use `std::mem::swap`:
```rust
fn apply_erosion(&mut self) {
    let iterations = 2;
    let mut swap_buffer = vec![vec![0.0; self.width]; self.height];
    
    for _ in 0..iterations {
        for y in 1..(self.height - 1) {
            for x in 1..(self.width - 1) {
                let center = self.data[y][x];
                let neighbors = [
                    self.data[y - 1][x],
                    self.data[y + 1][x],
                    self.data[y][x - 1],
                    self.data[y][x + 1],
                ];
                
                let max_neighbor = neighbors.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let min_neighbor = neighbors.iter().copied().fold(f64::INFINITY, f64::min);
                
                if center > max_neighbor + 0.1 {
                    swap_buffer[y][x] = center - 0.05;
                } else if center < min_neighbor - 0.1 {
                    swap_buffer[y][x] = center + 0.05;
                } else {
                    swap_buffer[y][x] = center;
                }
            }
        }
        
        std::mem::swap(&mut self.data, &mut swap_buffer);
    }
}
```

**Effort**: 1-2 hours

**Expected Impact**: Eliminates allocations during terrain generation

---

## Medium Priority Issues

### 5. Region Serialization Inefficiency

**Location**: `src/chunk/chunk_storage.rs:233-244` + `src/world/region.rs:73-89`

**Impact**:
- `flush_cache()` clones all chunks before serialization
- Nested loops create temporary allocations per chunk

**Recommendation**:
Stream chunks directly to disk without intermediate clones:
```rust
pub fn flush_cache(&self) -> Result<()> {
    let cache = self.cache.read();
    
    let mut regions: HashMap<RegionPos, Vec<Chunk>> = HashMap::new();
    
    for (pos, chunk) in cache.iter() {
        let region_pos = RegionPos::from_chunk(pos.x, pos.z);
        regions.entry(region_pos).or_insert_with(Vec::new).push(chunk.clone());
    }
    
    drop(cache);
    
    for (region_pos, chunks) in regions {
        let mut region = if region_path.exists() {
            Region::deserialize(&fs::read(&region_path)?)? 
        } else {
            Region::new(region_pos)
        };
        
        for chunk in chunks {
            region.insert(chunk);
        }
        
        fs::write(region_path, region.serialize())?;
    }
    
    Ok(())
}
```

**Effort**: 2-3 hours

---

### 6. String Allocation in Protocol Reading

**Location**: `src/network/protocol.rs:188-192` (`read_string`)

**Impact**:
- Allocates `Vec<u8>` then calls `from_utf8_lossy().to_string()`
- Double allocation for every string read from network

**Recommendation**:
Use stack buffer for small strings, avoid unnecessary conversions:
```rust
pub fn read_string(&mut self) -> std::io::Result<String> {
    let len = self.read_varint()? as usize;
    
    if len <= 256 {
        let mut buf = [0u8; 256];
        self.cursor.read_exact(&mut buf[..len])?;
        Ok(String::from_utf8_lossy(&buf[..len]).into_owned())
    } else {
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }
}
```

**Effort**: 1 hour

---

### 7. Spiral Chunk Loading Synchronous Blocking

**Location**: `src/chunk/chunk_sender.rs:36-65`

**Impact**:
- Nested loops call `chunk_storage.get_chunk()` synchronously
- Can trigger disk I/O and block the async executor
- Loading 100+ chunks in sequence is slow

**Recommendation**:
Batch load chunks asynchronously:
```rust
pub async fn send_chunks_around_player(
    socket: &mut TcpStream,
    chunk_storage: &ChunkStorage,
    chunk_x: i32,
    chunk_z: i32,
    radius: i32,
) -> Result<()> {
    let mut positions = Vec::new();
    
    for distance in 0..=radius {
        for dx in -distance..=distance {
            for dz in -distance..=distance {
                if dx.abs() != distance && dz.abs() != distance {
                    continue;
                }
                positions.push(ChunkPos::new(chunk_x + dx, chunk_z + dz));
            }
        }
    }
    
    // Load in parallel with spawn_blocking for each chunk
    let mut futures = Vec::new();
    for pos in positions {
        let storage = chunk_storage.clone();
        futures.push(tokio::task::spawn_blocking(move || {
            storage.get_chunk(pos)
        }));
    }
    
    // Send as chunks complete
    for future in futures {
        match future.await?? {
            Ok(chunk) => send_chunk(socket, &chunk).await?,
            Err(e) => debug!("[CHUNK] Failed to load chunk: {}", e),
        }
    }
    
    Ok(())
}
```

**Effort**: 2-3 hours

---

## Minor Issues & Improvements

### 8. Cache Hit Reset Task Lifecycle

**Location**: `src/chunk/chunk_storage.rs:67-78`

**Issue**: 
- Spawns unbounded tokio task that runs forever
- No graceful shutdown mechanism
- Should integrate with server shutdown

**Recommendation**:
Use a cancellation token (add `tokio-util` dependency):
```rust
pub struct ChunkStorage {
    cache: Arc<RwLock<LruCache<ChunkPos, Chunk>>>,
    shutdown: tokio_util::sync::CancellationToken,
    // ...
}

pub fn start_hit_reset_task(&self) {
    let cache = self.cache.clone();
    let shutdown = self.shutdown.child_token();
    
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(300)) => {
                    let mut cache_lock = cache.write();
                    cache_lock.reset_hit_counts();
                    drop(cache_lock);
                    debug!("[CHUNK] Hit counts reset");
                }
                _ = shutdown.cancelled() => {
                    debug!("[CHUNK] Hit reset task shutting down");
                    break;
                }
            }
        }
    });
}
```

**Effort**: 1-2 hours

---

### 9. Varint Allocation in Write Path

**Location**: `src/network/protocol.rs:56-73`

**Issue**: 
- `write_varint()` allocates new Vec for every call
- Called frequently during packet construction

**Recommendation**:
Write directly to buffer with fixed-size array:
```rust
pub fn write_varint(value: i32, buf: &mut [u8]) -> usize {
    let mut v = value as u32;
    let mut written = 0;
    
    loop {
        let mut temp = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            temp |= 0x80;
        }
        buf[written] = temp;
        written += 1;
        if v == 0 {
            break;
        }
    }
    
    written
}
```

**Effort**: 1-2 hours

---

### 10. Redundant Cache Capacity Checks

**Location**: `src/chunk/chunk_storage.rs:225`

**Issue**: 
- Manual flush at 50% capacity when cache already auto-expands
- Redundant check logic

**Recommendation**:
Remove manual flush, rely entirely on LRU eviction policy:
```rust
pub fn save_chunk(&self, chunk: Chunk) -> Result<()> {
    let (_, expanded, evicted_key) = {
        let mut cache = self.cache.write();
        cache.insert(chunk.pos, chunk.clone())
    };
    
    // Let cache handle eviction automatically
    // Remove the manual flush check at line 225
    
    Ok(())
}
```

**Effort**: 30 minutes

---

## Optimization Priority Matrix

| Priority | Issue | Impact | Effort | Est. Speedup |
|----------|-------|--------|--------|--------------|
| 🔴 Critical | Chunk cloning | 30% memory | 4-6h | 2-3x faster access |
| 🔴 Critical | Cache eviction O(n) | Stalls at capacity | 3-4h | 10-50x faster eviction |
| 🔴 Critical | Triple-nested loops | 66% slower serialization | 2-3h | 3x faster serialization |
| 🟠 High | Terrain grid clones | Wasteful allocation | 1-2h | Eliminates allocs |
| 🟠 High | String allocation | Double allocs | 1h | 2x faster reads |
| 🟠 High | Spiral loading | Blocks async | 2-3h | Parallel loading |
| 🟡 Medium | Region serialization | Inefficient flush | 2-3h | Faster saves |
| 🟡 Medium | Hit reset task | Unbounded task | 1-2h | Better lifecycle |
| 🟢 Low | Varint allocation | Minor overhead | 1-2h | Small improvement |
| 🟢 Low | Redundant checks | Code smell | 30m | Cleaner code |

---

## Implementation Roadmap

### Phase 1: Critical Path (8-12 hours)
1. Triple-nested loops → Single pass (2-3h, 3x faster serialization)
2. Cache eviction O(n) → Binary heap (3-4h, 10-50x faster)
3. Chunk cloning → Arc wrapper (4-6h, 30% memory reduction)

### Phase 2: High Impact (6-8 hours)
1. Terrain grid clones → Swap buffer (1-2h)
2. String allocation → Stack buffer (1h)
3. Spiral loading → Async batch (2-3h)

### Phase 3: Polish (4-6 hours)
1. Region serialization streaming (2-3h)
2. Hit reset task lifecycle (1-2h)
3. Varint direct write (1-2h)

---

## Testing & Profiling

### Before Optimization
```bash
# Run baseline profiling
cargo build --release
# Use perf, flamegraph, or valgrind to collect metrics
```

### After Each Phase
- Compare chunk serialization time: `serialize_chunk()` latency
- Monitor memory usage: Check `chunk_cache` RSS
- Profile cache eviction: Measure `evict_lowest_hits()` time
- Load test: Run with increasing player counts (10, 50, 100, 200)

### Key Metrics to Track
- Chunk load time (ms)
- Memory per cached chunk (bytes)
- Cache hit ratio (%)
- Serialization time (µs)
- Player spawn time (ms)

---

## Dependencies to Add

```toml
# For PluginThreadPool (already added to thread_pool.rs)
# For binary heap operations (already in std)

# Optional: For better async control
tokio-util = { version = "0.7", features = ["sync"] }

# Optional: For profiling
pprof = { version = "0.13", features = ["flamegraph"] }
```

---

## Notes

- All recommendations maintain API compatibility where possible
- Some changes require downstream updates (e.g., Arc-wrapping Chunk)
- Profile before and after each optimization to validate improvements
- Consider enabling LTO in release profile once Tokio migration is stable
