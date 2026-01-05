# Minecraft Java Edition Protocol — Inventory

This document describes the *Inventory* system in the Minecraft Java Edition network protocol: how slot indexes are defined across different container windows, the layout of player inventory and other GUI types, and how these relate to the packets that manipulate inventory state. :contentReference[oaicite:1]{index=1}

---

## 📌 Overview

In the Minecraft protocol, inventory state is referenced using **slot indices**.  
- Inventory slots are numbered sequentially starting at **0**.  
- A player’s inventory and any open container (chest, furnace, etc.) form a single contiguous index space per window.  
- Minecraft also uses special indices (e.g., **−1**) for interactions outside a window (such as dropping items).  
- The server and client must agree on these indices for packets like *Set Container Slot*, *Click Container*, and *Set Container Content*. :contentReference[oaicite:2]{index=2}

Slot indices vary by **container type** (player inventory, chest, horse, etc.). :contentReference[oaicite:3]{index=3}

---

## 🧠 Slot Index Basics

- The base index **0** always refers to the start of the container’s *unique slots* (its own GUI slots).  
- After counting all unique slots, slot indices continue into the player inventory portion.
- If window id `−1` is used, it refers to the **cursor item** — the stack being dragged by the mouse. :contentReference[oaicite:4]{index=4}

Example: In the player inventory itself, index **0** is the crafting output slot. :contentReference[oaicite:5]{index=5}

---

## 🎮 Player Inventory

The default player inventory (opened with `E`) has this layout:

| Slot Index | Meaning |
|------------|---------|
| `0` | Crafting output |
| `1–4` | 2×2 crafting input (`1 + x + 2*y`) |
| `5–8` | Armor slots (head, chest, legs, feet) |
| `9–35` | Main inventory |
| `36–44` | Hotbar |
| `45` | Offhand slot | :contentReference[oaicite:6]{index=6}

This layout *always* exists even if no explicit container window (like chest) is open. :contentReference[oaicite:7]{index=7}

---

## 🐴 Vehicle & Mob Containers

Depending on the rideable entity, the window layout changes:

### Horse

| Slot Index | Description |
|------------|-------------|
| `0` | Saddle |
| `1` | Armor |
| `2–28` | Horse inventory |
| `29–37` | Hotbar | :contentReference[oaicite:8]{index=8}

### Donkey (or Mule)

| Slot Index | Description |
|------------|-------------|
| `0` | Saddle |
| `1` | Armor |
| `2–16` | Donkey inventory |
| `17–43` | Player main inventory |
| `44–52` | Hotbar | :contentReference[oaicite:9]{index=9}

### Llama

Llamas have varying inventory size based on *strength*:

| Slot Index | Description |
|------------|-------------|
| `0` | Saddle |
| `1` | Carpet |
| `2–(2 + 3×strength)` | Llama inventory |
| `… subsequent slots` | Player inventory + hotbar | :contentReference[oaicite:10]{index=10}

---

## 📦 Other Container Types

### Chest (`generic_9x3`)

| Slot Index | Description |
|------------|-------------|
| `0–26` | Chest inventory |
| `27–53` | Main inventory |
| `54–62` | Hotbar | :contentReference[oaicite:11]{index=11}

### Dispenser (`generic_3x3`)

| Slot Index | Description |
|------------|-------------|
| `0–8` | Dispenser contents |
| `9–35` | Main inventory |
| `36–44` | Hotbar | :contentReference[oaicite:12]{index=12}

### Furnace

| Slot Index | Description |
|------------|-------------|
| `0` | Ingredient |
| `1` | Fuel |
| `2` | Output |
| `3–29` | Main inventory |
| `30–38` | Hotbar | :contentReference[oaicite:13]{index=13}

### Brewing Stand

| Slot Index | Description |
|------------|-------------|
| `0–2` | Bottles / potions |
| `3` | Ingredient |
| `4` | Blaze powder fuel |
| `5–31` | Main inventory |
| `32–40` | Hotbar | :contentReference[oaicite:14]{index=14}

### Crafting Table

| Slot Index | Description |
|------------|-------------|
| `0` | Crafting output |
| `1–9` | 3×3 crafting input |
| `10–36` | Main inventory |
| `37–45` | Hotbar | :contentReference[oaicite:15]{index=15}

### Other Containers

Other GUI types like **anvil**, **beacon**, **grindstone**, **hopper**, **loom**, **merchant**, **smithing**, **stonecutter**, and **cartography table** each have defined slot ranges within the inventory index space. For example:

- **Anvil**: slots 0–2 (input/output) followed by player inventory. :contentReference[oaicite:16]{index=16}  
- **Hopper**: slots 0–4 for hopper contents followed by player back inventory and hotbar. :contentReference[oaicite:17]{index=17}

The exact slot ranges are documented on the interview list for each container type. :contentReference[oaicite:18]{index=18}

---

## 🧪 Protocol Relevance

Slot indices are used in the inventory‑related packets such as:

### Container/Inventory Packets

- **Set Container Content** — server → client  
- **Set Container Slot** — server → client  
- **Click Container** — client → server  
- **Close Container** — client → server (indicates a window has been closed)  
- **Open Screen** — server → client (opens a window with an ID and type)

Slots are referenced within these packets using the index conventions above. :contentReference[oaicite:19]{index=19}

---

## 📦 Slot Data Structure

Each slot in inventory packets uses the **Slot data** structure (defined elsewhere) that encodes:

- Item count (VarInt)
- Optional item ID (VarInt)
- Optional component changes (used for structured item data)  
This replaces older raw NBT representations. :contentReference[oaicite:20]{index=20}

---

## 🟢 Special Indices

- **−1** — Represents the *cursor item* (the item being dragged). :contentReference[oaicite:21]{index=21}
- **Windows** — Each open inventory has a **window ID**; slot indices are relative to their window. :contentReference[oaicite:22]{index=22}

---

## 📌 Summary

| Context | What It Means |
|---------|----------------|
| Slot indexing | Sequential index representing inventory and UI slots |
| Multiple inventories | Each container type has its own slot layout |
| Protocol use | Inventory packets use these indices to reference slots |
| Special cases | Cursor (−1), hotbar/façade rules | :contentReference[oaicite:23]{index=23}

---

## 🛠 Implementation Tips

- Abstract slot indexing by **container type** in your implementation (e.g., map slots to logical roles).  
- In inventory manipulation logic, check for **out‑of‑bounds slot indices** according to window type.  
- For **Click Container** packets, the server validates slot indices against the current window’s layout. :contentReference[oaicite:24]{index=24}

---

*End of Inventory Protocol Reference*
::contentReference[oaicite:25]{index=25}
