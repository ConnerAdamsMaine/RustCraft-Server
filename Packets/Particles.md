# Minecraft Java Edition Protocol — Particles

Particles are **client‑side visual effects** spawned by the server using the **Level Particles** packet (`level_particles`). Each particle type has an associated ID and may require additional data. Clients render particles based on these values. :contentReference[oaicite:1]{index=1}

---

## 🧠 Particle Packet Context

**Packet:** `level_particles`  
**State:** Play  
**Direction:** Server → Client

```
level_particles {
long_distance: Boolean
always_visible: Boolean
x: Double
y: Double
z: Double
offset_x: Float
offset_y: Float
offset_z: Float
max_speed: Float
count: Int
particle_id: VarInt
particle_data: Varies by particle type
}
```

- **long_distance** increases visibility range.  
- **always_visible** forces rendering regardless of view distance.  
- **coordinates** are where the effect originates.  
- **offsets** with random Gaussian distribution apply spread.  
- **max_speed** affects motion.  
- **count** is how many particles to emit.  
- **particle_id** references a particle type.  
- **particle_data** carries extra, type‑specific fields. :contentReference[oaicite:2]{index=2}

---

## 🧾 Particle Types & Extra Data

The table below shows each particle type’s **name, numeric ID, and data fields**.

| Name | ID | Extra Fields | Description |
|------|----|--------------|-------------|
| `minecraft:angry_villager` | 0 | none | Angry villager effect |
| `minecraft:block` | 1 | `block_state: VarInt` | Block particle showing a specific block state |
| `minecraft:block_marker` | 2 | `block_state: VarInt` | Small marker particle tied to a block |
| `minecraft:bubble` | 3 | none | Underwater bubble |
| `minecraft:cloud` | 4 | none | Cloud puff |
| `minecraft:crit` | 5 | none | Critical hit effect |
| `minecraft:damage_indicator` | 6 | none | Damage pop effect |
| `minecraft:dragon_breath` | 7 | none | Dragon breath cloud |
| `minecraft:dripping_lava` | 8 | none | Lava drip |
| `minecraft:falling_lava` | 9 | none | Falling lava droplet |
| `minecraft:landing_lava` | 10 | none | Lava landing splash |
| `minecraft:dripping_water` | 11 | none | Water drip |
| `minecraft:falling_water` | 12 | none | Falling water |
| `minecraft:dust` | 13 | `color: Int, scale: Float` | Colored dust; color (0xRRGGBB) and size |
| `minecraft:dust_color_transition` | 14 | `from_color: Int, to_color: Int, scale: Float` | Dust transitioning between two colors |
| `minecraft:entity_effect` | 20 | `color: Int` | Effect particle tinted with ARGB color |
| `minecraft:falling_dust` | 28 | `block_state: VarInt` | Dust from falling block |
| `minecraft:item` | 46 | `item: Slot` | Item‑based particle |
| `minecraft:vibration` | 47 | Complex: depends on **source type** | Vibration particle with source and travel ticks |
| `minecraft:trail` | 48 | `target_x/y/z: Double, Color: Int, duration: VarInt` | Trail to a point with color and lifetime |
| `minecraft:sculk_charge` | 37 | `roll: Float` | Particle with rotation |
| `minecraft:shrieking` | 102 | `delay: VarInt` | Particle shown after delay |
| `minecraft:dust_pillar` | 108 | `block_state: VarInt` | Dust pillar based on block state |
| `minecraft:block_crumble` | 112 | `block_state: VarInt` | Block crumble effect |
| *(others with no fields)* | … | none | Many other vanilla particles have no extra fields and rely only on basic parameters. |

> All particles not listed explicitly here have **no extra fields** and carry only the default packet data. :contentReference[oaicite:3]{index=3}

---

## 🧠 Special Particle Notes

### `minecraft:vibration` (ID 47)

This particle has **type‑specific fields** depending on a built‑in registry for *position source type*:

- **Source Type 0 (`minecraft:block`)**:
  - `position: Position` — block origin.
- **Source Type 1 (`minecraft:entity`)**:
  - `entity_id: VarInt` — entity origin.
  - `entity_eye_height: Float` — height above entity base.

Then:
- `ticks: VarInt` — how many ticks until arrival. :contentReference[oaicite:4]{index=4}

---

## 🧠 Color Fields

Whenever a particle carries a **color Int**, it encodes:
`0xRRGGBB`
- R: red
- G: green
- B: blue  
Alpha bits are *ignored* for most particles. :contentReference[oaicite:5]{index=5}

---

## 🧠 Rendering Behavior

- Some particles are **directionless** and appear around the origin.  
- Others (like **trail**) use destination coordinates for vector effects.  
- **Count** and **offsets** determine randomness and spread.  
- **Long distance** increases visibility range. :contentReference[oaicite:6]{index=6}

---

## 📌 Particle Usage

Particles are spawned by:

- **Game events** (explosions, potions) using world/level event packets.  
- **Commands** and custom server logic sending `level_particles`.  
- **Block and entity effects** (like slime bounce, dust).  
- Texture atlases define visuals; server only sends IDs and data — client renders from the particle atlas. :contentReference[oaicite:7]{index=7}

---

## 🧩 Example

To spawn **colored dust** at position (10, 64, −20) with red color and scale 1.0:

```
level_particles {
long_distance: false,
always_visible: true,
x: 10, y: 64, z: -20,
offset_x/y/z: 0, 0, 0,
max_speed: 0,
count: 1,
particle_id: 13,
color: 0xFF0000,
scale: 1.0
}
```

This creates a single stationary red particle. :contentReference[oaicite:8]{index=8}

---

## 🛠 Implementation Tips

- Read **particle_id** first to decide how to parse extra fields.  
- Always apply **offsets and count** even for particles with no extra fields.  
- Apply the **long_distance** and **always_visible** flags to determine client rendering logic. :contentReference[oaicite:9]{index=9}

---

*End of Particles Protocol Artifact*
::contentReference[oaicite:10]{index=10}
