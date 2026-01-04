# Minecraft 1.21.7 Core Packet Structures with Responses

This document extends the previous packet spec with **server and client responses** for each packet.

---

## 1) Handshake Packet

**State:** Handshaking (client → server)
**Packet ID:** `0x00` (VarInt)

**Request Structure:**

```
Handshake {
    Protocol Version       VarInt
    Server Address         String
    Server Port            Unsigned Short
    Next State             VarInt // 1 = Status, 2 = Login
}
```

**Server Responses:**

* **Status Request Packet (0x00 in Status state)**: Client queries server info (MOTD, players, version)
* **Login Start Packet (0x00 in Login state)**: If Next State = 2, server expects Login Start

---

## 2) Login Start Packet

**State:** Login (client → server)
**Packet ID:** `0x00` (VarInt)

**Request Structure:**

```
Login Start {
    Username              String
}
```

**Server Responses:**

* **Login Success (0x02)**: Confirms login, provides UUID and username
* **Disconnect (0x00)**: If login fails (e.g., invalid username, banned, or auth failed)
* **Encryption Request (0x01)**: Optional if server requires online mode authentication

---

## 3) Login Success Packet

**State:** Login → Play (server → client)
**Packet ID:** `0x02` (VarInt)

**Response Structure:**

```
Login Success {
    UUID               UUID
    Username           String
}
```

**Client Action:**

* Switch to **Play state** immediately
* Await **Join Game** packet and world data

---

## 4) Join Game Packet

**State:** Play (server → client)
**Packet ID:** `0x26` / `0x28` (VarInt)

**Response Structure:**

```
Join Game {
    Entity ID              Int
    Hardcore Flag          Boolean
    Gamemode               Unsigned Byte
    Previous Gamemode      Unsigned Byte
    World Count            VarInt
    World Names            []IdentifierString
    Dimension Codec        NBT
    Dimension              NBT
    World Name             IdentifierString
    Hashed Seed            Long
    Max Players            VarInt
    View Distance          VarInt
    Reduced Debug Info     Boolean
    Enable Respawn Screen  Boolean
    Is Debug               Boolean
    Is Flat                Boolean
}
```

**Client Responses:**

* **Confirm Teleport / Position** packets as needed
* Begin **Chunk Data** requests and rendering
* Send **Player Position / Look** packets if needed

---

## 5) Chunk Data Packet

**State:** Play (server → client)
**Packet ID:** `0x20` / `0x27` (VarInt)

**Response Structure:**

```
Chunk Data {
    Chunk X              Int
    Chunk Z              Int
    Heightmaps           NBT
    Biomes (optional)    VarInt Array
    Data Size            VarInt
    Data                 Byte[]
    Block Entities       VarInt
    Block Entity NBT     NBT[]
}
```

**Client Responses / Actions:**

* Acknowledge chunk if needed (some mods/plugins expect chunk loading ACKs)
* Update local block states / lighting
* Send **Player Position** and **Entity Interactions** if player moves into chunk

---

## 6) Player Info Update Packet

**State:** Play (server → client)
**Packet ID:** `0x2D` / `0x53` (VarInt)

**Response Structure:**

```
Player Info {
    Action                VarInt
    Number of Entries     VarInt
    Entries[] {
        UUID              UUID
        // Fields vary by Action:
        // 0 = Add Player: Name, Properties[], Gamemode, Ping, Display Name
        // 1 = Update Gamemode
        // 2 = Update Ping
        // 3 = Update Display Name
    }
}
```

**Client Responses:**

* Update the **tab list UI** accordingly
* Track player entities for spawning or despawning
* Adjust local gamemode / display name if affected by packet

---

**Notes:**

* All **responses are determined by client expectation and server state**. Vanilla protocol rarely expects explicit ACK packets, but mods/plugins may add them.
* **Compression** must be applied if Set Compression was sent before these packets.
* **VarInt framing** remains critical for decoding all packets.

---

**References:**

* [Minecraft Protocol Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets)
* [Chunk Format Documentation](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format)
* [Player Info Packet](https://protocol.griefergames.dev/version/1.12)
