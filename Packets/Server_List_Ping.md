# Minecraft Java Edition — Server List Ping Protocol

This document describes the **Server List Ping (SLP)** interface of the Minecraft Java Edition protocol — the mechanism used by clients and server listings to query server status (MOTD, version, player counts, favicon, and latency). :contentReference[oaicite:1]{index=1}

---

## Overview

- SLP runs over a **TCP connection** to the server’s standard port (default `25565`). :contentReference[oaicite:2]{index=2}
- It uses the **standard Minecraft packet framing** (length-prefixed with VarInt) and the **Status** protocol state. :contentReference[oaicite:3]{index=3}
- It can be used standalone or as part of the handshake before a client actually joins. :contentReference[oaicite:4]{index=4}
- Modern Minecraft (1.7+ Java Edition) uses this protocol; legacy formats exist for older versions but are outside the scope here. :contentReference[oaicite:5]{index=5}

---

## Connection Flow

The high-level sequence for a modern status ping (protocol 1.7 and newer) is:

1. **TCP Connect**
2. **Handshake (Next State: Status)**
3. **Status Request**
4. **Status Response (JSON)**
5. **Ping Request (optional)**
6. **Pong Response**
7. **Connection Close** :contentReference[oaicite:6]{index=6}

---

## 1. Handshake

**Direction:** Client → Server  
**Purpose:** Switches the protocol state to *Status*.  
**Packet:**  
