# Minecraft Java Edition Protocol — Block Actions

The **Block Action** packet is a client‑bound Play‑state packet used to trigger **block‑specific animations and effects** that are not tied to persistent block state changes.  
These actions are usually short‑lived visual or mechanical effects that the client must render or simulate locally. It does **not** directly change block state (that is done via `Block Update` or `Set Section Blocks`). :contentReference[oaicite:1]{index=1}

---

## 📦 Packet: Block Action

**Packet ID:** `block_event` (Play state)  
**Direction:** Server → Client  
**Purpose:** Notify the client of a block animation or action associated with certain block types. :contentReference[oaicite:2]{index=2}

### **Fields**
| Field | Type | Description |
|-------|------|-------------|
| **Location** | `Position` | The position of the block triggering the action |
| **Action ID** | `Unsigned Byte` | Determines the meaning of the parameters based on the block type |
| **Action Parameter** | `Unsigned Byte` | Extra data interpreted per block type |
| **Block Type** | `VarInt` | ID from the `minecraft:block` registry (ignored by vanilla clients) |

> In protocol terms this packet ID sends the block’s position and two small values. The block type is sent but the *vanilla client ignores it* and instead uses the block state at the given position in its world. :contentReference[oaicite:3]{index=3}

---

## 🎮 Block Actions by Block Type

Different blocks interpret the **Action ID** and **Action Parameter** fields in different ways:

### 🪵 **Note Block**
- **Action ID:** Always `0`  
- **Action Parameter:** Ignored (always `0`)  
- Vanilla clients ignore parameters and use block state to decide note/pitch. :contentReference[oaicite:4]{index=4}

---

### 🧱 **Piston / Sticky Piston**
- **Action IDs:**
  - `0` — Extend piston
  - `1` — Retract piston
  - `2` — Cancel ongoing extension  
- **Action Parameter:** Direction the piston is facing (0=down, 1=up, 2=south, 3=west, 4=north, 5=east)  
- The client uses this to simulate a piston animation independent of actual block state, with the client waiting extra ticks before finalizing movement. :contentReference[oaicite:5]{index=5}

---

### 🗄️ **Chest (and Variants)**
- **Action ID:** `1`  
- **Action Parameter:** Number of players currently viewing the chest  
- Used to animate the lid opening and closing based on viewer count. :contentReference[oaicite:6]{index=6}

---

### 🌀 **Mob Spawner**
- **Action ID:** `1`  
- **Action Parameter:** Ignored  
- Triggers a reset of the spawner’s internal spawn delay timer on the client. :contentReference[oaicite:7]{index=7}

---

### 🌀 **End Gateway**
- **Action ID:** `1`  
- **Action Parameter:** Ignored  
- Triggers the purple beam animation when an entity travels through a gateway. :contentReference[oaicite:8]{index=8}

---

### 📦 **Shulker Box**
- **Action IDs:**
  - `0` — Opening or closing animation
  - `1` — Update viewer count  
- **Action Parameter:**
  - `0` — Close animation
  - `1` — Open animation or viewer count  
- Used to animate the shell opening and closing depending on viewer interaction. :contentReference[oaicite:9]{index=9}

---

### 🔔 **Bell**
- **Action ID:** `1`  
- **Action Parameter:** Direction the bell was rung (0=down, 1=up, etc.)  
- Causes the bell ring animation; the sound is handled with a separate Sound Effect packet. :contentReference[oaicite:10]{index=10}

---

### 🏺 **Decorated Pot**
- **Action ID:** `1`  
- **Action Parameter:** Wobble style  
  - `0` — Positive wobble (item inserted)
  - `1` — Negative wobble (interaction failed)  
- Triggers a wobble animation on the pot model. :contentReference[oaicite:11]{index=11}

---

## 🧠 Important Notes

- The **Block Type** field exists for completeness, but vanilla clients ignore it — they always infer block type locally from the world state at the given coordinates. :contentReference[oaicite:12]{index=12}
- Not all blocks have associated actions. If a block type is unsupported by the vanilla client’s action tables, the packet is usually ignored. :contentReference[oaicite:13]{index=13}
- Block actions are typically short, client‑side effects and do not carry persistent world state changes — those are handled by separate block update packets. :contentReference[oaicite:14]{index=14}

---

## 🧾 When Is It Used?

Block Action packets are sent when **something in the world needs to produce an animated response** that is not otherwise covered by a simple block state update. Examples include pistons extending/retracting, chest lids moving, bells ringing, and shulker boxes opening. :contentReference[oaicite:15]{index=15}

---

## 📌 Summary

The Block Action packet allows the server to trigger **context‑sensitive, block‑specific animations/effects** on the client with minimal data.

