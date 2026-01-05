# Minecraft Java Edition Protocol — Command Data

**Command Data** defines the **full structure of commands as a directed graph** that the client receives from the server in the Play state. This enables clients to parse, validate, and provide features like syntax highlighting and tab completion for commands. :contentReference[oaicite:1]{index=1}

---

## 📌 Overview

Since **Minecraft 1.13**, command syntax is not represented as simple text alone — it is encoded as a **graph of nodes (root, literal, argument)** that describes every valid command and its structure.  
The client builds this graph and uses it to interpret, restrict, and suggest command completions. :contentReference[oaicite:2]{index=2}

---

## 🧠 Graph Structure

The command graph is a **directed graph** consisting of interconnected nodes:

- **Root** — unnamed origin for all commands.
- **Literal** — represents specific fixed keywords (e.g., `give`, `tp`, `say`).
- **Argument** — variable placeholder (e.g., player name, number).  
Nodes may **redirect** to other nodes or have multiple children. :contentReference[oaicite:3]{index=3}

---

## 📦 Node Format

Each command graph node in the packet has these fields:

| Field | Type | Description |
|-------|------|-------------|
| **Flags** | `Byte` | Bitfield indicating node type and behaviors (*see below*) |
| **Children count** | `VarInt` | Number of child node indices |
| **Children** | Array of `VarInt` | Indices of child nodes |
| **Redirect node** | Optional `VarInt` | Present if `flags & 0x08` (redirect) |
| **Name** | Optional `String` | Present for literal and argument nodes |
| **Parser ID** | Optional `VarInt` | Only for argument nodes |
| **Properties** | Varies | Parser‑specific extra data |
| **Suggestions type** | Optional `Identifier` | Only if `flags & 0x10` | :contentReference[oaicite:4]{index=4}

The values above refer to the graph **as sent over the network** — the index values connect nodes into a navigable structure. :contentReference[oaicite:5]{index=5}

---

## 🧬 Flags Bit Mask

Each node’s **Flags** byte has bits with these meanings:

| Mask | Name | Meaning |
|------|------|---------|
| `0x03` | **Node type** | 0: root, 1: literal, 2: argument |
| `0x04` | **Executable** | If set, this node represents a complete valid command |
| `0x08` | **Has redirect** | Followed by `Redirect node` |
| `0x10` | **Has suggestions type** | Followed by `Suggestions type` |
| `0x20` | **Restricted** | Requires elevated permission level to use | :contentReference[oaicite:6]{index=6}

---

## 📌 Parsers

Argument nodes specify a **parser ID** to interpret typed arguments. Clients must implement all parsers present in the graph; if the client encounters an unknown parser, it **cannot continue parsing commands beyond that point**. :contentReference[oaicite:7]{index=7}

Below are some of the most common parsers and their meanings:

### 📍 Basic Types

| ID | Identifier | Description |
|----|------------|-------------|
| `0` | `brigadier:bool` | Boolean (`true` or `false`) |
| `1` | `brigadier:float` | Floating‑point number with optional min/max |
| `2` | `brigadier:double` | Double precision floating‑point |
| `3` | `brigadier:integer` | Integer |
| `4` | `brigadier:long` | Long integer |
| `5` | `brigadier:string` | String with various behaviors | :contentReference[oaicite:8]{index=8}

---

## 📍 Minecraft‑Specific Parsers

Minecraft extends brigadier with many argument types:

| ID | Identifier | Meaning |
|----|------------|---------|
| `6` | `minecraft:entity` | Selector, player name, or UUID |
| `7` | `minecraft:game_profile` | Player profile |
| `8` | `minecraft:block_pos` | Block position (x, y, z) |
| `10` | `minecraft:vec3` | 3D vector position |
| `12` | `minecraft:block_state` | Block state |
| `14` | `minecraft:item_stack` | Item with data |
| `16` | `minecraft:color` | Chat color |
| `18` | `minecraft:component` | JSON text component |
| `20` | `minecraft:message` | In‑game chat message |  
(…and many more, including resource and tag selectors) :contentReference[oaicite:9]{index=9}

Each parser may optionally include **properties** such as min/max values for numeric arguments or registry references for resources. :contentReference[oaicite:10]{index=10}

---

## 📌 Parser Properties

Some parsers have extra structured data:

### Numeric Parsers (Example: `brigadier:double`)

| Field | Description |
|-------|-------------|
| **Flags** | Presence of min and/or max values |
| **Min** | Optional Double |
| **Max** | Optional Double | :contentReference[oaicite:11]{index=11}

Flags determine whether the optional min/max fields are present. Similar structures exist for float/int/long types. :contentReference[oaicite:12]{index=12}

---

## 🚀 Suggestions Types

When a node has a **suggestions type** (`flags & 0x10`), an Identifier follows indicating how the client should suggest completions. Common suggestion types include:

| Identifier | Suggestion Meaning |
|------------|-------------------|
| `minecraft:ask_server` | Client should request tab completions from the server |
| `minecraft:all_recipes` | Suggest all known recipe names |
| `minecraft:available_sounds` | Suggest all loaded sound identifiers |
| `minecraft:summonable_entities` | Suggest all entities that can be summoned | :contentReference[oaicite:13]{index=13}

Unknown suggestion types default to `minecraft:ask_server` behavior. :contentReference[oaicite:14]{index=14}

---

## 🧾 Putting It All Together

When combined:

1. **Server sends the Commands packet** containing:
   - A **prefixed array of `Node` objects**
   - A **root index** pointing to the root node. :contentReference[oaicite:15]{index=15}

2. The client builds an **in‑memory graph** linking nodes via indices.

3. The client uses this graph to:
   - Validate user command strings
   - Provide **tab completion suggestions**
   - Enforce restrictions (e.g., permission levels)

4. If the client encounters an **unknown parser ID**, it must not assume anything about what follows. :contentReference[oaicite:16]{index=16}

---

## 🗺️ Example (Abstract)

