# Minecraft Java Edition Protocol — Data Types

This document defines the standard data types used on the Minecraft Java Edition network protocol.

> All multi-byte primitives (except VarInt/VarLong) are **big-endian** (most significant byte first). :contentReference[oaicite:1]{index=1}

---

## 1. Primitive Types

| Type            | Wire Size    | Meaning / Encoding |
|-----------------|--------------|--------------------|
| **Boolean**     | 1 byte       | `0x00` = false, `0x01` = true. :contentReference[oaicite:2]{index=2} |
| **Byte**        | 1 byte       | Signed 8-bit integer (−128 to 127). :contentReference[oaicite:3]{index=3} |
| **Unsigned Byte** | 1 byte     | Unsigned 0 to 255. :contentReference[oaicite:4]{index=4} |
| **Short**       | 2 bytes      | Signed 16-bit integer. :contentReference[oaicite:5]{index=5} |
| **Unsigned Short** | 2 bytes   | Unsigned 0 to 65535. :contentReference[oaicite:6]{index=6} |
| **Int**         | 4 bytes      | 32-bit signed integer. :contentReference[oaicite:7]{index=7} |
| **Long**        | 8 bytes      | 64-bit signed integer. :contentReference[oaicite:8]{index=8} |
| **Float**       | 4 bytes      | IEEE 754 32-bit floating point. :contentReference[oaicite:9]{index=9} |
| **Double**      | 8 bytes      | IEEE 754 64-bit floating point. :contentReference[oaicite:10]{index=10} |

---

## 2. Variable-Length Integers

### **VarInt**
- **1–5 bytes**
- Encodes a 32-bit signed integer in base-128 with continuation bits.
- More compact for small magnitudes.
- Used for packet IDs, array lengths, registry IDs, etc. :contentReference[oaicite:11]{index=11}

### **VarLong**
- **1–10 bytes**
- Same format as VarInt but for 64-bit signed integers. :contentReference[oaicite:12]{index=12}

---

## 3. Strings and Identifiers

### **String**
- UTF-8 encoded.
- Prefixed with a **VarInt** length (in bytes) on the wire.
- Maximum length (in UTF-16 code units) often enforced by context (e.g., 32767). :contentReference[oaicite:13]{index=13}

### **Identifier**
- A string limited to certain formats (namespaces, paths).
- Encoded as a normal Minecraft string. :contentReference[oaicite:14]{index=14}

---

## 4. Compound & Structured Types

### **Position**
- 8 bytes (packed).
- Contains three signed coordinates:
  - X: 26 bits
  - Z: 26 bits
  - Y: 12 bits  
- Decoded with bit masking and arithmetic shifts. :contentReference[oaicite:15]{index=15}

### **UUID**
- 16 bytes.
- Two 64-bit unsigned parts: high and low. :contentReference[oaicite:16]{index=16}

---

## 5. Optional & Array Types

### **Optional X**
- Either absent or contains one `X`.
- Presence often determined by context. :contentReference[oaicite:17]{index=17}

### **Prefixed Optional X**
- Boolean precedes the value.
- If `true`, one `X` follows. :contentReference[oaicite:18]{index=18}

### **Array of X**
- Sequence of `X` values, length defined by context. :contentReference[oaicite:19]{index=19}

### **Prefixed Array of X**
- Begins with a **VarInt** length.
- Then that many `X` elements. :contentReference[oaicite:20]{index=20}

---

## 6. Bit Sets

### **BitSet**
- Prefixed by a **VarInt** number of `long`s.
- Each bit indicates a flag. :contentReference[oaicite:21]{index=21}

### **Fixed BitSet (n)**
- Exactly `ceil(n / 8)` bytes.
- Pads unused bits to zero. :contentReference[oaicite:22]{index=22}

---

## 7. Registry & Inline Types

### **ID or X**
- VarInt `id`.
- If `id == 0`: follows with a full value `X`.
- Else: `id − 1` represents a registry reference. :contentReference[oaicite:23]{index=23}

### **ID Set**
- First a type VarInt.
- If `0`: follows an **Identifier** tag for a registry tag.
- Else: array of registry IDs (`type − 1` elements). :contentReference[oaicite:24]{index=24}

---

## 8. Miscellaneous Protocol Structures

| Type | Encoded As | Notes |
|------|------------|-------|
| **NBT** | Dependent | Standard Named Binary Tag format. :contentReference[oaicite:25]{index=25} |
| **Slot** | Structured | Encodes item stack data. :contentReference[oaicite:26]{index=26} |
| **Chunk / Light Data** | Composite | Prefixed arrays + masks. :contentReference[oaicite:27]{index=27} |
| **Teleport Flags** | 4 bytes | Int bitmask for positional update flags. :contentReference[oaicite:28]{index=28} |

---

## 9. Endianness Rules

- **VarInt/VarLong:** variable-length, no fixed endianness.
- All other primitive numbers (Short, Int, Long, Float, Double): **big-endian** on the wire. :contentReference[oaicite:29]{index=29}

---

## 10. Binary Packing Notes (Rust/Python/TS)

- Use **big-endian primitives** for fixed sizes.
- Implement VarInt/VarLong with continuation bit logic.
- Respect maximum lengths on strings and identifiers.
- For structured types (Position, BitSets), decode/encode with bit operations.

---

*End of Data Types Reference*
