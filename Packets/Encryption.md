# Minecraft Java Edition Protocol — Encryption Overview

This document explains the **encryption layer** used by Minecraft’s Java Edition protocol during the login handshake. Encryption is only applied when connecting to servers in **online mode** (and optionally in newer offline-mode configurations). :contentReference[oaicite:1]{index=1}

---

## 📌 Purpose

Encryption in Minecraft Java Edition serves two goals:

1. **Authenticate the player with Mojang/Microsoft session servers** (online mode).  
2. **Secure the rest of the login process and gameplay traffic** using a symmetric cipher.  

Everything beyond the encryption handshake (including packet lengths and IDs) is encrypted once enabled. :contentReference[oaicite:2]{index=2}

---

## 🔑 High-Level Encryption Flow

The encryption handshake occurs after the initial login handshake:

C → S: Handshake (State = Login)

C → S: Login Start

S → C: Encryption Request

(Optional) Client authenticates with Mojang

C → S: Encryption Response

(Optional) Server authenticates with Mojang

Both sides enable symmetric encryption

S → C: Login Success
⚠️ If the server is in **offline mode without encryption**, steps 3–7 may be skipped entirely. :contentReference[oaicite:3]{index=3}

---

## 🧾 Encryption Request (Server → Client)

**Packet:** Encryption Request  
Sent when the server wants to initiate secure login.

**Contains:**
- **Server ID** — empty string in modern Java Edition.  
- **Public Key** — server’s RSA public key (ASN.1 DER-encoded).  
- **Verify Token** — random bytes for client verification.  

The server’s RSA keypair (usually 1024 bits) is generated on startup. The public key is sent so the client can encrypt sensitive information. :contentReference[oaicite:4]{index=4}

---

## 🔐 Key Exchange (Client → Server)

**Packet:** Encryption Response

**Process:**
1. Client generates a 16-byte random **shared secret**.
2. Client encrypts:
   - The **shared secret**
   - The **verify token**
   with the server’s RSA public key (PKCS#1 v1.5 padded).
3. Both encrypted values are sent to the server.

The server uses its private key to decrypt these and validate the token. This verifies the client’s identity and establishes a shared secret key for symmetric encryption. :contentReference[oaicite:5]{index=5}

---

## 🔁 Enabling Symmetric Encryption

Once the server decrypts and validates the client’s response:

- Both sides use the shared secret as the **AES/CFB8 key and IV**.
- Two AES/CFB8 ciphers are created:
  - One for encrypting outgoing data
  - One for decrypting incoming data

From this point onward, **all traffic — including packet lengths and IDs — is encrypted** over the TCP connection. :contentReference[oaicite:6]{index=6}

> **Note:** AES in CFB8 mode means encryption/decryption operates one byte at a time with an 8-bit feedback segment. Many crypto libraries require explicit configuration for CFB8. :contentReference[oaicite:7]{index=7}

---

## 🧠 Authentication With Mojang

### 📌 Client Authentication
Before sending the Encryption Response:

1. Client computes a SHA-1 hash over:
   - Server ID string  
   - Shared secret  
   - Server public key bytes  
2. The hash is converted to Minecraft’s non-standard hex format.
3. The client sends this hash to Mojang’s session server via HTTP POST:
```
POST https://sessionserver.mojang.com/session/minecraft/join

Content-Type: application/json
{
"accessToken": "<accessToken>",
"selectedProfile": "<player UUID>",
"serverId": "<computed hash>"
}
```
A `204 No Content` response means success. :contentReference[oaicite:8]{index=8}

### 🧾 Server Authentication
After decrypting the shared secret, the server computes the same hash and queries:
```
GET https://sessionserver.mojang.com/session/minecraft/hasJoined?username=
<username>&serverId=<hash>&ip=<client ip_optional>
```
If the server receives a valid response, it extracts the player’s UUID and profile properties to send in the **Login Success** packet. :contentReference[oaicite:9]{index=9}

---

## 🧪 Offline / Optional Encryption Modes

- **Offline Mode (no encryption):** The server never sends an Encryption Request; Login Success follows Login Start directly. :contentReference[oaicite:10]{index=10}
- **Offline Mode with encryption (modern):** Protocol supports encryption even without online authentication, but vanilla servers don’t typically use it. :contentReference[oaicite:11]{index=11}

---

## 🧾 Practical Notes

- Because everything after encryption is encrypted (including packet framing), you **must enable decryption before processing further packets**. Otherwise packets will appear as random bytes. :contentReference[oaicite:12]{index=12}
- The RSA key size and ASN.1 DER key format are critical when importing the public key into crypto libraries. :contentReference[oaicite:13]{index=13}

---

## 🗂 Summary

Encryption setup in the Minecraft Java Edition protocol:

| Stage | Description |
|-------|-------------|
| Client generates shared secret | Used for AES |
| Server sends public key + verify token | Client uses this to encrypt shared secret |
| Client sends Encryption Response | Encrypted shared secret + token |
| Symmetric encryption enabled | All further packets are encrypted |
| Client & server authenticate with Mojang | Ensures identity in online mode |

This encrypted handshake ensures both the **security** and **authentication integrity** of a Java Edition session. :contentReference[oaicite:14]{index=14}

