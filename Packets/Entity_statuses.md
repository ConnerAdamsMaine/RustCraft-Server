# Minecraft Java Edition Protocol — Entity Statuses

Entity statuses are small numeric values sent in the **Entity Event** packet to trigger **animations**, **sounds**, **particles**, and other **entity effects** on the client side.  
The meaning of a status code depends on the entity type receiving it. :contentReference[oaicite:2]{index=2}

---

## 🧠 Packet Context

**Packet:** `Entity Event`  
**State:** Play  
**Direction:** Server → Client

```
entity_event {
entity_id: VarInt
entity_status: Byte
}
```

- `entity_id` identifies the entity the status applies to.  
- `entity_status` is a small enum whose meaning varies by entity.  
- Clients handle these codes to display animations or trigger effects. :contentReference[oaicite:3]{index=3}

---

## 📋 Common Status Codes

Below are examples of entity status codes and their effects for typical classes of entities. Codes not listed for a specific class simply do nothing by default.

---

### 🧍 Entity (Base)

| Status | Description |
|--------|-------------|
| **53**  | Spawn honey block slide particles at feet |

> Base entity statuses apply to all entities unless overridden. :contentReference[oaicite:4]{index=4}

---

## 🪐 Projectile Entities

### **Snowball**

| Status | Effect |
|--------|--------|
| **3** | Spawns 8 `snowballpoof` particles |

> Affects snowball entities only. :contentReference[oaicite:5]{index=5}

### **Egg**

| Status | Effect |
|--------|--------|
| **3** | Spawns 8 `iconcrack` particles with egg as parameter |

> Similar to snowball but with egg visuals. :contentReference[oaicite:6]{index=6}

---

### 🎣 Fishing Hook

| Status | Effect |
|--------|--------|
| **31** | If hooked entity is the local player, pulls them toward the rod’s caster |

> Applies motion on the client. :contentReference[oaicite:7]{index=7}

---

## 🐾 Living & Mob Entities

### **Living Entity (general)**

| Status | Effect |
|--------|--------|
| **3** | Play death animation and sound |
| **29** | Shield block sound |
| **30** | Shield break sound |
| **35** | Totem of Undying animation & sound |
| **46** | Chorus teleport portal particles |
| **47–52** | Play equipment break sounds/particles (main hand, off hand, head, chest, legs, feet) |
| **54** | Spawn honey fall particles |
| **55** | Swap hand items |
| **60** | Spawn death smoke particles |

> These apply to all living entities (mobs, players) unless overridden by more specific types. :contentReference[oaicite:8]{index=8}

---

### **Player‑specific**

| Status | Effect |
|--------|--------|
| **9** | Mark item use as finished (e.g., eating, drinking) |
| **22** | Enable reduced debug info |
| **23** | Disable reduced debug info |
| **24–28** | Set operator permission level (0–4) |
| **43** | Spawn cloud particles related to Bad Omen effect |

> These codes control UI state and player effects. :contentReference[oaicite:9]{index=9}

---

## 🐄 Other Entity Types

### **Armor Stand**

| Status | Effect |
|--------|--------|
| **32** | Play hit sound, reset hit cooldown |

> Only applies to armor stand entities. :contentReference[oaicite:10]{index=10}

---

### **Mob (Generic)**

| Status | Effect |
|--------|--------|
| **20** | Spawn explosion particle (silverfish, spawner effects) |

> Many mobs use this for visual cues. :contentReference[oaicite:11]{index=11}

---

## 🐬 Water Animals

### **Squid**

| Status | Effect |
|--------|--------|
| **19** | Reset squid rotation to 0 radians |

> Helps maintain correct orientation. :contentReference[oaicite:12]{index=12}

### **Dolphin**

| Status | Effect |
|--------|--------|
| **38** | Spawn “happy villager” particles when fed |

> Client effect only. :contentReference[oaicite:13]{index=13}

---

## 🐷 Additional Animals

### **Animal (Generic)**

| Status | Effect |
|--------|--------|
| **18** | Spawn “love mode” heart particles |

> Applies to ageable animals. :contentReference[oaicite:14]{index=14}

### **Abstract Horse**

| Status | Effect |
|--------|--------|
| **6** | Spawn smoke on failed taming |
| **7** | Spawn hearts on successful taming |

> Similar flags apply to tameable animals. :contentReference[oaicite:15]{index=15}

---

## 🧠 Monster & Unique Mobs

### **Zombie Villager**

| Status | Effect |
|--------|--------|
| **16** | Play zombie villager cure sound |

> For conversion feedback. :contentReference[oaicite:16]{index=16}

### **Guardian**

| Status | Effect |
|--------|--------|
| **21** | Play guardian attack sound |

> Applies to guardians and elder guardians. :contentReference[oaicite:17]{index=17}

---

## 🛞 Minecarts

### **Minecart TNT**

| Status | Effect |
|--------|--------|
| **10** | Ignite TNT (no sound) |

> Visual ignition signal. :contentReference[oaicite:18]{index=18}

### **Minecart Spawner**

| Status | Effect |
|--------|--------|
| **1** | Reset spawner delay to default minimum |

> Client resets internal timer. :contentReference[oaicite:19]{index=19}

---

## 🐻 Unique Newer Entities

### **Warden**

| Status | Effect |
|--------|--------|
| **4** | Stop roar, play attack animation |
| **61** | Tendril shaking animation |
| **62** | Sonic boom attack animation (visual only) |

> Complex animation codes. :contentReference[oaicite:20]{index=20}

### **Sniffer**

| Status | Effect |
|--------|--------|
| **63** | Play digging sound if target and in proper state |

> Applies digging behavior. :contentReference[oaicite:21]{index=21}

---

## 📌 Notes

- **Entity statuses are discrete, lightweight triggers** for effects that are otherwise not reflected in persistent entity state. :contentReference[oaicite:22]{index=22}  
- Many statuses correspond to **particles, sounds, or simple animations** rather than gameplay logic. :contentReference[oaicite:23]{index=23}  
- If an entity receives an unrecognized status, vanilla clients typically **ignore** it. :contentReference[oaicite:24]{index=24}

