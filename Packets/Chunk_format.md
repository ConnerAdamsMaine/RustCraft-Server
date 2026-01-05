# Minecraft Java Edition Protocol — Chunk Format

This document summarizes the **chunk format** used by the Minecraft Java Edition protocol when sending world data in a `Chunk Data` packet. A “chunk” here means all the block/biome/light data for a 16×16 slice of the world, broken vertically into 16×16×16 *chunk sections*. :contentReference[oaicite:1]{index=1}

> The chunk format described here applies to modern Java Edition releases (1.18+ data model); older implementations differ significantly. :contentReference[oaicite:2]{index=2}

---

## 📌 High‑Level Concepts

### Chunk Column vs Chunk Section

- A **chunk column** is a vertical stack of *chunk sections*, each 16×16×16 blocks tall, covering the full world height. :contentReference[oaicite:3]{index=3}  
- Chunk sections are the **unit of data** sent per section (some may be omitted if empty). :contentReference[oaicite:4]{index=4}

---

## 🧱 Packet Structure (Chunk Data Packet)

When a server sends world terrain, the `Chunk Data` packet includes these core fields:

| Field | Type | Meaning |
|-------|------|---------|
| **Chunk X** | Int | Chunk column X coordinate |
| **Chunk Z** | Int | Chunk column Z coordinate |
| **Heightmaps** | Prefixed array | Serialized heightmap values |
| **Size** | VarInt | Byte length of next data blob |
| **Data** | Byte Array | Packed chunk section data |
| **Additional Data** | Various | Block entities, etc. | :contentReference[oaicite:5]{index=5}

> This table represents the *on‑the‑wire* structure used for client terrain updates. :contentReference[oaicite:6]{index=6}

---

## 🗺️ Heightmaps

Heightmaps encode the highest non‑air block in each XZ column of the chunk. They are sent so the client can optimize rendering and behavior (e.g., rain occlusion, pathfinding). :contentReference[oaicite:7]{index=7}

### Heightmap Format

For each heightmap:

| Field | Type | Description |
|-------|------|-------------|
| **Type** | VarInt Enum | Heightmap variant (e.g., `WORLD_SURFACE`) |
| **Data** | Prefixed array of Longs | Packed values (one per XZ cell) | :contentReference[oaicite:8]{index=8}

Height values are stored in a compact bit‑packed form based on world height. :contentReference[oaicite:9]{index=9}

---

## 🧱 Chunk Section Layout

A chunk column’s *Data* blob consists of one element per chunk section (0…n), *bottom‑to‑top*:

Each chunk section has:

| Field | Type | Meaning |
|------|------|---------|
| **Block count** | Short | Number of non‑air blocks in the section |
| **Block states** | Paletted Container | 4096 blocks with palettes |
| **Biomes** | Paletted Container | 4×4×4 regions (64 entries) | :contentReference[oaicite:10]{index=10}

Empty sections (all air/cave air/void air) can be omitted entirely. :contentReference[oaicite:11]{index=11}

---

## 🔢 Paletted Containers

Both block states and biomes use **paletted containers** — compact bit arrays with optional palettes to reduce payload size. :contentReference[oaicite:12]{index=12}

### Structure

| Field | Type | Meaning |
|------|------|---------|
| **Bits Per Entry** | Unsigned Byte | Bits used per entry |
| **Palette** | Varies | Maps index → registry ID |
| **Data Array** | Array of Long | Block/biome values packed | :contentReference[oaicite:13]{index=13}

---

## 🧠 Palette Formats

How entries are stored depends on `Bits Per Entry` (BPE):

| BPE | Format | Meaning |
|-----|--------|---------|
| **0** | Single‑valued | One single ID applies to all positions |
| **4–8** (blocks) / **1–3** (biomes) | Indirect | Indexed via palette array |
| **≥ direct threshold** | Direct | Encodes registry IDs directly | :contentReference[oaicite:14]{index=14}

Direct encoding typically occurs when there are too many unique entries to palette compactly. :contentReference[oaicite:15]{index=15}

---

## 📦 Data Array Packing

- Entries are packed into 64‑bit longs, tightly packed.  
- Entries are ordered with **x fastest, then z, then y** (inner → outer).  
- Padding may be inserted to align entries within longs. :contentReference[oaicite:16]{index=16}

---

## 📘 Biomes and Block States

### Block States

- A chunk section has 4096 blocks → one entry per block.  
- Registry IDs reference the server’s **block state registry** (sent via Registry Data). :contentReference[oaicite:17]{index=17}

### Biomes

- Each 16×16×16 section includes 64 biome entries (4×4×4 sampling grid).  
- Biome IDs reference the **biome registry** (also sent via Registry Data). :contentReference[oaicite:18]{index=18}

---

## 🛠 Implementer Notes

- **Empty sections** are omitted — you only send sections with any non‑air block count > 0. :contentReference[oaicite:19]{index=19}  
- **Palettes** save bandwidth by sending only the distinct values in a section. :contentReference[oaicite:20]{index=20}  
- **Heightmaps** are optional — if missing, the client initializes defaults and uses local terrain updates. :contentReference[oaicite:21]{index=21}

---

## 📌 Example Principles

- A fully homogeneous chunk section (all stone) would use **Bits Per Entry = 0** and encode a single stone block state ID. :contentReference[oaicite:22]{index=22}  
- A “mixed” section (many different blocks) may use an **Indirect palette** for efficiency. :contentReference[oaicite:23]{index=23}  
- Direct mode is used when many unique blocks/biomes fill a section. :contentReference[oaicite:24]{index=24}

---

## 🧱 Summary

| Component | Purpose |
|-----------|---------|
| **Heightmaps** | Optimized surface height data |
| **Chunk Sections** | 16×16×16 block + biome + lighting data |
| **Paletted Containers** | Efficient encoding of repeated values |
| **Packed Data Arrays** | Bit‑packed entries for blocks/biomes | :contentReference[oaicite:25]{index=25}

This structure allows the server to transmit complete chunk terrain efficiently and consistently across Minecraft clients.

