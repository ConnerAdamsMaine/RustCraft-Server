# Minecraft Java Edition Protocol — Registry Data

**Registry Data** refers to a set of structured tables (registries) that define gameplay content — such as biomes, dimension types, chat formats, particle definitions, etc. During connection setup, the server sends these registries to the client so that both sides share the same ID mappings and data representations. This enables consistent interpretation of packet fields that reference registry entries by numeric IDs. :contentReference[oaicite:2]{index=2}

---

## 📌 Why Registries Exist

Minecraft uses *registries* to map gameplay objects to IDs:

- Each registry groups objects of a particular type (biomes, chat types, particles, etc.).
- Each entry has:
  - A textual **Identifier** (`minecraft:example`)
  - A numeric runtime ID (the client/server agrees on the ordering).  
- Many packets reference entries by **numeric ID** for efficiency; the registry data makes these mappings deterministic. :contentReference[oaicite:3]{index=3}

---

## 🧠 Protocol Context

Registry Data is exchanged **during the configuration phase** (just after login) via **Registry Data packets** sent from server to client. :contentReference[oaicite:4]{index=4}

The common structure of a Registry Data packet is:

```
Registry Data {
Registry ID (Identifier)
Entries (prefixed array of Identifiers — these define numeric IDs)
Entry Data (prefixed optional NBT per entry — details depend on the registry)
}
```

- **Registry ID** names which registry this packet represents (e.g., `minecraft:worldgen/biome`).  
- **Entries** are the ordered collection; their position → numeric ID.  
- **Entry Data** carries structured data (often NBT) specific to each entry. :contentReference[oaicite:5]{index=5}

> The client must reject references to registry entries that weren’t sent or mapped — this protects protocol consistency. :contentReference[oaicite:6]{index=6}

---

## 🔄 Client/Server Pack Exchange

To avoid sending redundant data, a lightweight handshake occurs:

1. **Server → Client:** Known Packs  
2. **Client → Server:** Known Packs  
3. Server computes **mutually supported datapacks**
4. **Server → Client:** Multiple Registry Data packets  
   - One for each selected registry  
   - Only entries **not shared** through known packs are sent :contentReference[oaicite:7]{index=7}

---

## 📋 Example Registries

Below are commonly sent registries with descriptions of their purpose and primary fields. This is not exhaustive — registries evolve between versions.

### 🏞️ `minecraft:worldgen/biome`

Defines biome characteristics (used in rendering, world gen, and client visuals):

- `has_precipitation` — Byte (rain/snow?).  
- `temperature` / `downfall` — Floats for biome conditions.  
- `effects` — Compound with fog, water, grass/foliage colors, ambient sounds, particles, etc.  
- Client uses these values for environment visuals and sound. :contentReference[oaicite:8]{index=8}

---

### 🗨️ `minecraft:chat_type`

Describes how different chat messages are formatted:

- `chat` — Chat format pattern.  
- `narration` — Narration formatting.  
- Includes translation key and parameter ordering.  
- Used by chat packets to render localized messages. :contentReference[oaicite:9]{index=9}

---

### ⚔️ `minecraft:damage_type`

Used in combat and event packets:

- `message_id` — Death message translation key.  
- `scaling` — How damage scales with difficulty or source.  
- `exhaustion` — Fatigue cost of damage.  
- Optional effects/feedback tags. :contentReference[oaicite:10]{index=10}

---

### 🌍 `minecraft:dimension_type`

Provides dimension parameters used in respawn and world logic:

- `fixed_time` — If present, locks daytime.  
- `has_skylight`, `has_ceiling`, `ultrawarm` — Dimension features.  
- `min_y`, `height`, `logical_height` — World height rules.  
- `effects` — References visual/dimension effects. :contentReference[oaicite:11]{index=11}

---

### 🖼️ `minecraft:painting_variant`

Information for painting assets:

- `asset_id` — Texture location.  
- `width`, `height` — Painting dimensions.  
- `title`, `author` — Localized text for display. :contentReference[oaicite:12]{index=12}

---

### 🧥 Other Registries

Minecraft defines additional registries such as:

- `minecraft:trim_material` — Armor trim visual properties.
- `minecraft:trim_pattern` — Armor trim pattern data.
- (Other registries exist and may be version‑specific.) :contentReference[oaicite:13]{index=13}

---

## 🧩 Entry Data Format

The **structure of `Entry Data`** is registry‑dependent:

- Some are simple primitives (strings, ints).
- Others include **NBT compounds** that describe behaviors or visuals.  
- The server must serialize these correctly so that the client interprets them uniformly. :contentReference[oaicite:14]{index=14}

---

## 🔁 Runtime Implications

- The **order** of registry entries defines their numeric IDs.  
- This **must match** on both client and server.  
- Many packets reference registry entries by numeric IDs (not names).  
- A mismatch → protocol error and disconnect. :contentReference[oaicite:15]{index=15}

---

## 📌 Summary

| Aspect | Description |
|--------|-------------|
| **What** | Protocol‑level tables of game data (biomes, chat types, dimensions, etc.) |
| **When Sent** | Configuration phase after login |
| **Carrier Packet** | `Registry Data` from server to client |
| **Key Concept** | Ordered entries define numeric IDs consistently |
| **Runtime Use** | Other packets use registry IDs for compact references | :contentReference[oaicite:16]{index=16}

---

## 🛠️ Implementation Tips

- Always parse **Registry ID first** to know the schema.  
- Read **entries array first** to build name‑to‑ID maps.  
- Then parse **entry data** if present.  
- Any unknown registry → disconnect is safest. :contentReference[oaicite:17]{index=17}

