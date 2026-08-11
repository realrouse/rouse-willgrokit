# Discord reply (paste for pcfreak30)

Hey — dug into Iroh for video + what an LBRY-blob-shaped redesign could look like vs WebTorrent-style “UDP on the web.” Write-up is ready if you want the long form.

**Short version:** Iroh’s interesting piece for LBRY-like VOD is **`iroh-blobs`** (BLAKE3 content-addressed blobs, **verified streaming**, real **byte-range** fetches over QUIC) — not “livestream MoQ” (that’s `iroh-live`, still early). That range+verify model is much closer to what you want for seekable video than classic whole-blob LBRY exchange.

**e2e relays:** connectivity glue + encrypted fallback. Relay **can’t read** payload; **can** see who talked to whom and how many bytes. Public relays are rate-limited toys for prod video. Dedicated/self-hosted relays for NAT; **separate always-on providers** (disk/S3) replace LBRY reflectors. Don’t confuse relay with storage.

**Browser WASM:** real (docs + demos), but **browser traffic is relay-only** today — no hole-punch from the sandbox. Fine for experiments; for seed/publish/NAT-hard users the **companion/node + local HTTP to the player** path still wins (same spirit as browser LBRY clients with optional local daemon). Transport e2e ≠ LBRY content AES — paid streams still need app-layer keys.

**vs WebTorrent:** WebTorrent was the right hack to put **BitTorrent** in a tab (WebRTC + WS trackers). If we’re redesigning **claims → content identity → transfer**, shoehorning classic BT piece markets into RTCDataChannels is the wrong long-term bet. Better: claims/Urma still point at identity; data plane becomes Iroh tickets + blobs; dual-read old `sd_hash` via rehost/gateway instead of a big-bang rehash.

Curious what you’d pick for content identity on new publishes: **one BLAKE3 of the file**, or **HashSeq of ~2MiB encrypted LBRY-compatible chunks** so existing blob farms can migrate transport without re-encrypting everything?
