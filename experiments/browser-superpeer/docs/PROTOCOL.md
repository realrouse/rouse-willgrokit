# Protocol notes: LBRY blobs over Iroh (`lbry-blob-iroh/1`)

## Goals

- Keep **LBRY blob identity** (SHA-384 of each blob; stream descriptor JSON; AES-256-CBC content blobs).
- Carry those bytes over **Iroh** between a **download superpeer** and a client (CLI or localhost companion).
- Do **not** implement reflector upload, blockchain resolve, or browser-to-browser CDN in this MVP.

## ALPN

```
lbry-blob-iroh/1
```

## Ticket

Superpeer prints a **ticket**: URL-safe base64 (no padding) of a JSON `iroh::EndpointAddr`.

Clients decode the ticket and `Endpoint::connect(addr, ALPN)`.

## Framing (one request per bidirectional stream)

### Client → superpeer

1. `u8` command  
   - `1` = Have  
   - `2` = GetBlob  
2. `u8` length of hash hex string (ASCII, lowercase preferred)  
3. `length` bytes of hex characters (SHA-384 hex is 96 chars)

Then finish the send side.

### Superpeer → client

**Have**

- `u8` `1` if present, `0` if missing

**GetBlob**

- `u32` big-endian status: `0` = OK, `1` = not found, `2` = bad request  
- if OK: `u64` big-endian length, then `length` raw blob bytes (the exact LBRY blob file)

## Client verification

1. Fetch blob for `sd_hash`, verify `SHA-384(bytes) == sd_hash`.  
2. Parse JSON stream descriptor (`version`, `blobs[]`, `key`, `filename`).  
3. For each content entry: fetch, verify hash, AES-256-CBC decrypt with stream `key` and entry `iv` (PKCS7).  
4. Concatenate plaintext in order.

Transport e2e encryption is **Iroh/QUIC**. Content encryption is **LBRY stream key**. They are different layers.

## What this is not

- Not a DHT.  
- Not an upload reflector.  
- Not claim/wallet protocol.  
- Not pure browser WASM Iroh (companion uses native Iroh; page is ordinary HTML/JS).
