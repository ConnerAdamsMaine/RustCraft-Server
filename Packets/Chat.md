# Minecraft Java Edition Protocol — Chat System

This document describes the **chat subsystem** of the Minecraft Java Edition protocol — covering both server‑bound and client‑bound chat packets, how they interact with player settings, the various chat message types, and how they are processed. :contentReference[oaicite:1]{index=1}

---

## 📌 Chat Overview

In Minecraft Java Edition, **chat is more than a single packet** — the protocol includes:

- Client → Server packets for *sending chat text* and *commands*.
- Server → Client packets for *broadcasting messages*, *system feedback*, and *signed chat*.  
- Client settings that influence which messages are shown. :contentReference[oaicite:2]{index=2}

The chat system is tightly coupled with **message signing, chat types, and filtering**. :contentReference[oaicite:3]{index=3}

---

## 🧑‍💻 Client Chat Settings

The client exposes settings that affect what chat it *may send* and *what it will display*:

**Chat Mode Options:**
| Mode | Meaning |
|------|---------|
| **0 — Enabled** | Everything shown |
| **1 — Commands Only** | Only commands sent/displayed |
| **2 — Hidden** | Normal chat hidden | :contentReference[oaicite:4]{index=4}

**Interaction with Packets:**
- When chat is **Hidden** or **Commands Only**, the client is restricted from sending plain chat messages.  
- Commands can still be sent when restricted by mode.  
- If the server rejects a sent chat (e.g., due to mode or validation), the server sends a **System Chat Message** with a *`chat.disabled.options`* translation text. :contentReference[oaicite:5]{index=5}

---

## 🚀 Server‑Bound Chat Packets

These are packets the *client sends* related to chat:

### 1. **Chat Message (`chat`)**
**Direction:** Client → Server  
**State:** Play  
**Fields:**
- `message`: String (up to 256 characters)
- `timestamp`: Long — milliseconds since epoch
- `salt`: Long — random per message
- `signature`: Optional 256‑byte signature
- Last‑seen message metadata (counts, checksums) :contentReference[oaicite:6]{index=6}

**Purpose:**
- Client sends this to transmit a text message to the server.
- Includes cryptographic fields when *signed chat* is enabled. :contentReference[oaicite:7]{index=7}

---

### 2. **Chat Command (`chat_command`)**
**Direction:** Client → Server  
**State:** Play  
**Fields:**
- `command`: String  
- This indicates the player typed a command (starting with `/`). :contentReference[oaicite:8]{index=8}

---

### 3. **Signed Chat Command (`chat_command_signed`)**
**Direction:** Client → Server  
**State:** Play  
**Fields:**
- Full command string
- Timestamp, salt
- Argument signatures
- Other fields to support signature verification :contentReference[oaicite:9]{index=9}

**Purpose:**
- Used when the client supports *signed command chat* (improves reliability for server‑side command logging/verification). :contentReference[oaicite:10]{index=10}

---

### 4. **Acknowledge Message (`chat_ack`)**
**Direction:** Client → Server  
**State:** Play  
**Fields:**
- `Message Count`: VarInt  
**Purpose:**
- Acknowledges receipt of server chat messages (used to manage last‑seen list for signing). :contentReference[oaicite:11]{index=11}

---

## 🗣️ Server‑Bound Chat Protocol Logic

1. The client maintains a **chat session ID** and **per‑message index**.  
2. For signed chat, each message includes cryptographic metadata so the server can verify authenticity and order.  
3. The server uses the acknowledgement from clients (`chat_ack`) to maintain a *last‑seen set* for signatures. :contentReference[oaicite:12]{index=12}

---

## 💬 Client‑Bound Chat Packets

These packets notify clients of various chat events:

### 1. **System Chat Message (`system_chat`)**
**Direction:** Server → Client  
**State:** Play  
**Fields:**
- `content`: Text Component  
- `overlay`: Boolean — indicates action bar (true) vs chat (false) display. :contentReference[oaicite:13]{index=13}

**Purpose:**
- System messages: feedback from server, errors, announcements.  
- Overlay = display above hotbar (e.g., scoreboard notifications). :contentReference[oaicite:14]{index=14}

---

### 2. **Disguised Chat Message**
**Direction:** Server → Client  
**State:** Play  
**Fields:**
- `message`: Text Component  
- `chat_type`: ID or inline Chat Type  
- `sender name`: Text Component  
- `target name`: Optional Text Component :contentReference[oaicite:15]{index=15}

**Purpose:**
- Used when the server wants to send messages with custom sender/target (e.g., `/say`, `/me`).  
- Does *not include signature verification data* (compared to Player Chat Message). :contentReference[oaicite:16]{index=16}

---

### 3. **Player Chat Message**
**Direction:** Server → Client  
**State:** Play  
**Fields (important ones):**
- `header`: Global Index (VarInt)
- `sender UUID`: UUID
- `indexed content`: Structured signature and text components
- `filter type`: Enum (pass through, filtered)  
- `chat formatting`: Chat Type (registry or inline)
- `sender name`: Text Component
- `target name`: Optional Text Component :contentReference[oaicite:17]{index=17}

**Purpose:**
- Broadcast actual player messages signed by the server; includes signatures for client verification and filtering. :contentReference[oaicite:18]{index=18}

---

## 📚 Chat Formatting and Chat Types

Many chat packets carry a **Chat Type** field:

- This is either:
  - A **registry reference** to `minecraft:chat_type`, defined by Registry Data, or  
  - An inline definition of chat formatting parameters. :contentReference[oaicite:19]{index=19}

The **Chat Type** defines:
- Translation keys (message format strings)
- Parameters (sender, target, content)
- Optional style NBT  
so that clients know how to render messages consistently. :contentReference[oaicite:20]{index=20}

---

## 🛡️ Message Blocking & Chat Privacy

Clients may enforce **blocking** based on player blocks or settings:

- When a player is blocked client‑side:
  - Player chat messages may be filtered based on sender UUID.  
  - System messages are blocked based on textual occurrences of the blocked name only if *Hide Matched Names* is enabled.  
  - Disguised chat is *never* blocked. :contentReference[oaicite:21]{index=21}

---

## 📌 Summary

| Direction | Packet | Purpose |
|-----------|--------|---------|
| Client → Server | `chat` | Send chat text |
| Client → Server | `chat_command` | Send a typed command |
| Client → Server | `chat_command_signed` | Signed command for verification |
| Client → Server | `chat_ack` | Acknowledge server chat |
| Server → Client | `system_chat` | System/feedback messages |
| Server → Client | `disguised_chat` | Message from entity/console |
| Server → Client | `player_chat` | Broadcast signed chat | :contentReference[oaicite:22]{index=22}

---

## 🧠 Notes

- Chat packets use **text components**, meaning messages are structured JSON‑like objects, not raw text.  
- Signature data is critical for *secure chat* and must be handled per the protocol when present. :contentReference[oaicite:23]{index=23}

