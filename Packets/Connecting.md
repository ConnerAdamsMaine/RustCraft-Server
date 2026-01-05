# Minecraft Java Edition Protocol – Connection Sequences

This document describes **exact packet-level connection flows** for Minecraft Java Edition based on the official protocol FAQ. It explains *what is sent*, *by whom*, *in what state*, and *why it exists*, covering both **login** and **status ping** flows.

This is written from an implementer’s perspective (proxy, custom client, server, or protocol library).

---

## Protocol States Overview

Minecraft connections are **stateful**. Every packet is interpreted according to the current protocol state.

| State         | Purpose                                           |
| ------------- | ------------------------------------------------- |
| Handshaking   | Selects the next protocol state (Login or Status) |
| Status        | Server list ping (MOTD, players, latency)         |
| Login         | Authentication, encryption, identity              |
| Configuration | Client/server capability exchange before Play     |
| Play          | Normal gameplay                                   |

State transitions are **explicit** and primarily driven by the **Handshake** packet and server responses.

---

## TCP Connection Establishment

Before any Minecraft packets are exchanged:

1. Client opens a **TCP connection** to the server (host:port).
2. No encryption, compression, or framing beyond TCP exists yet.

All subsequent packets are length-prefixed and VarInt-framed per protocol rules.

---

## Normal Login Sequence (Joining a Server)

This sequence is used when a player actually joins a server.

### 1. Handshake (Select Login State)

**Direction:** Client → Server
**State:** Handshaking

**Packet:** Handshake

**Fields:**

* Protocol Version (VarInt)
* Server Address (String)
* Server Port (Unsigned Short)
* Next State (VarInt = `2` for Login)

**Purpose:**

* Tells the server which protocol version the client expects
* Explicitly switches the connection into the **Login** state

⚠️ No other packet is valid before this.

---

### 2. Login Start

**Direction:** Client → Server
**State:** Login

**Packet:** Login Start

**Fields:**

* Username (String)
* Optional UUID (usually omitted by vanilla clients)

**Purpose:**

* Declares the player identity attempting to join
* Triggers authentication logic on the server

At this point, the server decides whether encryption is required.

---

### 3. Encryption Request (Online Mode Only)

**Direction:** Server → Client
**State:** Login

**Packet:** Encryption Request

**Fields:**

* Server ID (String, often empty)
* Server Public Key (Byte Array)
* Verify Token (Byte Array)

**Purpose:**

* Initiates authentication and secure key exchange
* Required for online-mode servers

If the server is **offline-mode**, this step is skipped entirely.

---

### 4. Encryption Response (Online Mode Only)

**Direction:** Client → Server
**State:** Login

**Packet:** Encryption Response

**Fields:**

* Shared Secret (encrypted with server public key)
* Verify Token (encrypted)

**Purpose:**

* Proves the client can decrypt server data
* Establishes the shared AES session key

Immediately after this packet:

* Both sides **enable encryption**
* Server contacts Mojang session servers to verify identity

---

### 5. Login Success

**Direction:** Server → Client
**State:** Login

**Packet:** Login Success

**Fields:**

* UUID (128-bit)
* Username (String)
* Player Properties (textures, signatures, etc.)

**Purpose:**

* Confirms authentication is complete
* Confirms player identity

The connection is now authenticated and (if applicable) encrypted.

---

### 6. Login Acknowledged

**Direction:** Client → Server
**State:** Login

**Packet:** Login Acknowledged

**Fields:** None

**Purpose:**

* Confirms receipt of Login Success
* Signals readiness to transition states

After this packet, the server transitions the connection into **Configuration** state.

---

## Configuration State (Pre-Play Setup)

This state exists to exchange client capabilities and metadata before gameplay begins.

### 7. Client Information

**Direction:** Client → Server
**State:** Configuration

**Packet:** Client Information

**Fields include:**

* Locale
* View distance
* Chat settings
* Main hand
* Text filtering preferences

**Purpose:**

* Informs server of client-side preferences

---

### 8. Plugin Messages (Optional)

**Direction:** Client → Server
**State:** Configuration

**Packet:** Plugin Message (e.g. `minecraft:brand`)

**Purpose:**

* Identifies client brand/mod loader
* Used by servers for compatibility or analytics

---

### 9. Finish Configuration

**Direction:** Server → Client
**State:** Configuration

**Packet:** Finish Configuration

**Purpose:**

* Signals that configuration is complete
* Immediately transitions connection to **Play** state

Gameplay packets begin after this point.

---

## Status Ping Sequence (Server List Ping)

This sequence is used for querying server status (MOTD, players, latency).

No authentication, encryption, or configuration occurs.

---

### 1. Handshake (Select Status State)

**Direction:** Client → Server
**State:** Handshaking

**Packet:** Handshake

**Fields:**

* Protocol Version
* Server Address
* Server Port
* Next State (VarInt = `1` for Status)

**Purpose:**

* Switches the connection into **Status** state

---

### 2. Status Request

**Direction:** Client → Server
**State:** Status

**Packet:** Status Request

**Fields:** None

**Purpose:**

* Requests server status information

---

### 3. Status Response

**Direction:** Server → Client
**State:** Status

**Packet:** Status Response

**Fields:**

* JSON string containing:

  * Server version
  * Player counts
  * MOTD
  * Optional server icon

**Purpose:**

* Provides data shown in the multiplayer server list

---

### 4. Ping Request

**Direction:** Client → Server
**State:** Status

**Packet:** Ping Request

**Fields:**

* Timestamp (Long)

**Purpose:**

* Measures round-trip latency

---

### 5. Pong Response

**Direction:** Server → Client
**State:** Status

**Packet:** Pong Response

**Fields:**

* Same timestamp sent by client

**Purpose:**

* Allows client to calculate ping

The server usually closes the connection afterward.

---

## Summary Diagrams

### Login Flow

```
Client → Server: Handshake (Login)
Client → Server: Login Start
Server → Client: Encryption Request (optional)
Client → Server: Encryption Response (optional)
Server → Client: Login Success
Client → Server: Login Acknowledged
[Configuration packets]
Server → Client: Finish Configuration
→ Play State
```

### Status Flow

```
Client → Server: Handshake (Status)
Client → Server: Status Request
Server → Client: Status Response
Client → Server: Ping Request
Server → Client: Pong Response
[Connection closes]
```

---

## Implementation Notes

* Handshake **always** precedes state-specific packets
* Offline-mode servers skip encryption entirely
* Compression may be enabled during Login or Configuration
* Packet IDs are **state-dependent**
* Sending packets from the wrong state results in disconnect

This sequence is stable across modern Java versions, with minor additions in Configuration and Play.
