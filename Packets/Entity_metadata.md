# Minecraft Java Edition Protocol — Entity Metadata

**Entity Metadata** is a flexible, dynamic set of data fields sent from the server to the client to describe various entity properties such as animations, health, effects, custom names, and other stateful attributes.  
These fields are used during entity spawn and update packets to synchronize entity state on the client.:contentReference[oaicite:1]{index=1}

---

## 📦 Metadata Field Format

Entity metadata is encoded as a **sequence of entries**, each with:

| Field | Type | Description |
|-------|------|-------------|
| **Index** | `Unsigned Byte` | Field identifier. `0xFF` marks the end of the metadata list. |
| **Type** | `VarInt Enum` | Type of the metadata value (present only if Index isn’t `0xFF`). |
| **Value** | *Varies by `Type`* | The metadata value interpreted according to the type. |

> The list ends when `Index = 0xFF` is read. Values beyond that aren’t decoded.:contentReference[oaicite:2]{index=2}

---

## 🧠 Metadata Types

The **Type** enum determines how the subsequent value is read. Common types include:

| Type ID | Meaning | Encoded As |
|---------|---------|------------|
| `Byte` | Small integer / bit mask | 1 byte |
| `VarInt` | Variable‑length integer | VarInt |
| `Float` | Floating‑point number | 4‑byte big‑endian |
| `String` | UTF‑8 text | Prefixed String |
| `Text Component` | JSON structured text | Chat component encoding |
| `Optional Text` | Boolean + text | Optional structure |
| `Position` | Block position | 8 bytes packed |
| `Slot` | Inventory slot | Slot format |
| `Block State` | Block state ID | VarInt |
| `Entity Reference` | Entity ID | VarInt |

> See packet docs for a complete mapping of type IDs to formats; types determine how the `Value` is decoded.:contentReference[oaicite:3]{index=3}

---

## 📌 How It Appears on the Wire

The sequence looks like:

