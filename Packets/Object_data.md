# Minecraft Java Edition Protocol — Object Data

**Object data** is a packet‑level field used in the **Spawn Entity** and related entity creation packets in the Java Edition network protocol. Its meaning is **entity‑type dependent** — the network doesn’t assign a single unified interpretation; instead, the receiver uses the entity type to decode this integer into the appropriate semantics. :contentReference[oaicite:1]{index=1}

---

## 🧠 Protocol Context

In packets such as:

```
Spawn Entity
Spawn Experience Orb
Spawn Global Entity
Spawn Mob
Spawn Painting
```


one of the fields is called **Object Data** (type `Int`). The **entity type** field preceding it tells the client how to interpret this integer. :contentReference[oaicite:2]{index=2}

---

## 🪟 Interpretations by Entity Type

Below are the common interpretations of **Object Data** for several entity types that use it:

### 🎯 Item Frame

| Value | Orientation |
|-------|-------------|
| `0` | Down |
| `1` | Up |
| `2` | North |
| `3` | South |
| `4` | West |
| `5` | East |

The client clamps out‑of‑range values with modulo semantics; negative values are turned positive. Velocity fields are ignored for item frames. :contentReference[oaicite:3]{index=3}

---

### 🖼️ Painting

| Value | Orientation |
|-------|-------------|
| `2` | North |
| `3` | South |
| `4` | West |
| `5` | East |

Values `0` and `1` are technically invalid, but the vanilla client still processes them via clamping. Invalid values can prevent the entity from spawning. Note that velocity is ignored. :contentReference[oaicite:4]{index=4}

---

### 🧱 Falling Block

For falling block entities (e.g., sand, gravel falling in the world), **Object Data** is interpreted as:

- **Block State ID (Int)** — the numeric block state that this falling block represents.

The velocity in the spawn packet is ignored, and physics are implied by the game engine. :contentReference[oaicite:5]{index=5}

---

### 🎣 Fishing Hook

| Field | Meaning |
|-------|---------|
| `Owner` (Int) | The entity ID of the fishing rod’s owner |

The client uses this to link the fishing bobber entity to the owning player. If the referenced entity doesn’t exist locally, the hook may fail to spawn. Velocity is ignored for this entity. :contentReference[oaicite:6]{index=6}

---

### 🏹 Projectile (Generic)

For some projectiles that include this field, **Object Data** is the **Entity ID** of the owner or shooter. The client may use it to determine game logic like pickup or acceleration. :contentReference[oaicite:7]{index=7}

---

### 🧟 Warden

The warden uses **Object Data** as:

- **Pose (Int)** — when set to `1`, it indicates the warden should spawn in the *emerging* pose. Any other value is ignored. :contentReference[oaicite:8]{index=8}

---

## 📌 Notes

- The field exists purely for *entity‑specific supplemental information*; it does not have a universal interpretation across all entity types. :contentReference[oaicite:9]{index=9}  
- The vanilla client often ignores other packet fields such as velocity for these entities, relying instead on server or local logic for movement and animation. :contentReference[oaicite:10]{index=10}  
- Because meanings depend on the entity type, protocol implementations must switch based on that type before decoding this field. :contentReference[oaicite:11]{index=11}

---

## 🧾 Summary (Entity Object Data Interpretations)

| Entity Type       | Meaning of Object Data |
|------------------|------------------------|
| Item Frame       | Orientation enum (0–5) |
| Painting         | Orientation enum (2–5) |
| Falling Block    | Block State ID         |
| Fishing Hook     | Owner Entity ID        |
| Projectile       | Owning Entity ID       |
| Warden           | Spawn Pose flag        |

---

## 📘 Implementation Guidance

When decoding a `Spawn Entity` packet:

1. Read **Entity Type ID**  
2. Read **Object Data (Int)**  
3. Dispatch interpretation based on the entity type:  
   - If item frame/painting → interpret as orientation  
   - If falling block → treat as block state ID  
   - If hook/projectile → associate with owning entity  
   - If warden → use as pose flag  
4. Ignore velocities when the client spec says they aren’t used for that type.

This ensures correct entity initialization according to the protocol’s intended semantics. :contentReference[oaicite:12]{index=12}

