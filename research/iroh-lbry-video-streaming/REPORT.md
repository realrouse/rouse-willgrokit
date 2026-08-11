# Iroh for video streaming & an Iroh-shaped LBRY blob revision

**Research report for rouse / pcfreak30 discussion**  
**Date:** 2026-08-11  
**Mode:** Read-only analysis (no product code, no deploys)  
**Source preference:** Official Iroh docs + repos; LBRY protocol spec; WebTorrent primary write-ups. Public LBRY/LumeWeb ecosystem context (Urma, browser clients, liblbry) used only as secondary orientation.

---

## 1. Executive summary

- **Iroh is not “BitTorrent with a new coat of paint.”** It is a modular dial-by-public-key networking stack (QUIC + TLS 1.3) with composable protocols (blobs, gossip, docs). Connectivity prefers direct peer paths and falls back to **relays that cannot read content** (end-to-end encryption).
- **For large files / VOD, the relevant piece is `iroh-blobs`:** content-addressed by **BLAKE3 root hash**, with **verified streaming** and **range requests** so you can fetch byte ranges and check integrity as data arrives — this is the right primitive for seekable video *if* you put a media container + player logic on top.
- **Iroh does not ship a first-class “Netflix ABR player.”** Live A/V is a separate, early stack (`iroh-live` / Media over QUIC). Adaptive bitrate and polished VOD seeking are **application-layer** work on top of ranges + multiple encodings, not magic properties of the transport.
- **LBRY today:** ~2 MiB **encrypted** content blobs (AES-256-CBC), SHA-384 IDs, **sd/manifest blob** listing order + IVs + key material, on-chain claim → `streamHash`/`sd_hash`, discovery historically via **DHT + peers/reflectors**.
- **An Iroh-oriented revision** keeps **claims / identity / payment** LBRY-native (or Urma-like location claims) and moves **discovery + transfer** to Iroh endpoints + `iroh-blobs` (or HashSeq/collections). Encryption for paywalled content stays an **app-layer** concern (do not confuse transport e2e with LBRY content encryption).
- **Browser path is real but constrained:** official docs say **browser WASM works**, but **all browser traffic goes through relays** (no raw UDP hole-punch from the sandbox). Full `iroh-blobs` in pure web still needs a maturity spike; companion/native node remains the high-performance path.
- **WebTorrent shoehorns BitTorrent into WebRTC** so browsers can join *some* swarms. That was brilliant for “BT on the web,” but long-term it keeps you married to BT piece semantics, tracker/signaling quirks, and a dual world (web peers vs classic peers).
- **When the shoehorn is the wrong long-term bet:** you want dial-by-key, verified range streaming, modern multipath/NAT, and a stack designed for *relay-as-fallback* rather than *reinventing UDP piece exchange over RTCDataChannel*.
- **Browser LBRY clients:** pure-web playback can be “resolve claim → fetch via Iroh relay path or HTTP gateway”; full privacy/seed/publish stays **companion optional**; **wallet stays separate from blobs**.
- **Risks to name upfront:** unfinished browser media story, **relay economics/abuse**, encryption parity with LBRY blobs, spam/poisoning of providers, legal/ops of running relays (not content stores — still hot).
- **For pcfreak (one technical line):** Prototype **VOD = BLAKE3 blob (or HashSeq of encrypted chunks) + Bao-verified range GETs over Iroh QUIC**, with **claim metadata pointing at root hash + provider ticket**, dual-read with legacy `sd_hash` via a re-encode/gateway — *not* “port LBRY DHT into WebRTC.”

---

## 2. Iroh video streaming — current state

### 2.1 What Iroh is (accurate as of docs consulted 2026-08-11)

From [What is iroh?](https://docs.iroh.computer/what-is-iroh):

| Layer | Job |
|-------|-----|
| Transport | UDP by default; swappable (Tor, etc.) |
| QUIC + TLS 1.3 | E2E encryption, auth, stream mux |
| **Endpoint** | Identity (`EndpointId` = public key), address lookup, NAT, relay fallback |
| **Router** | Dispatch inbound connections by **ALPN** to protocol handlers |
| **Protocols** | App logic: `iroh-blobs`, `iroh-gossip`, `iroh-docs`, custom |

**Promises of the stack:** reach a peer by key; get the best path available (direct preferred; relay fallback).

### 2.2 Large sequential media: `iroh-blobs`

Official blobs docs ([protocols/blobs](https://docs.iroh.computer/protocols/blobs)) and [docs.rs/iroh-blobs](https://docs.rs/iroh-blobs/latest/iroh_blobs/) (v0.103.0, June 2026):

| Capability | Status / detail |
|------------|-----------------|
| **Content addressing** | 32-byte **BLAKE3 root hash** of opaque blob bytes |
| **Verified streaming** | BLAKE3 tree / BAO-style outboard; integrity checked **while** streaming (not only at end) |
| **Range / seek** | **Range requests**: fetch a verifiable contiguous byte subsequence by streaming only needed tree portions |
| **Concurrency** | Multiple QUIC streams/connections; **Downloader** can pull from multiple providers |
| **Resumability** | Designed for resume (outboard + partial store); practical for interrupted downloads |
| **Collections / HashSeq** | Ordered sequences of hashes (concatenated 32-byte links); metadata blob by convention first |
| **Sizes** | Documented as scaling “kilobytes to terabytes” in ecosystem positioning |
| **Chunk size** | BLAKE3 leaves default **1 KiB** (~6% outboard overhead); tunable **without** changing root hash |
| **Production note** | Latest docs.rs line states it is **not yet considered production quality** for that version line — pin/verify before production |

**What this means for video:**

- **Realistic:** progressive download of an MP4/WebM; **seek** by mapping player byte offsets → blob range requests; integrity of each range; multi-provider fan-in for popular content; ticket-based “fetch this hash from this endpoint.”
- **Hand-wavy if claimed as “done by Iroh”:** full **HLS/DASH ABR**, keyframe-aligned seeks, CDN-class cold-start latency, automatic re-encoding ladders. Those are **media pipeline** concerns. Iroh gives you a verified byte pipe + addressing.

### 2.3 First-class / community patterns for video (not just file download)

| Pattern | What it is | Maturity |
|---------|------------|----------|
| **VOD over blobs** | Store media file (or chunked HashSeq) as blob(s); player does HTTP-like range semantics over blob API | Protocol-ready; **app must glue** MSE/`<video>` |
| **`iroh-live` + MoQ** | Real-time A/V over iroh; Media over QUIC; independent streams for audio vs video renditions; multi-preset / ABR-ish in media layer | **Early tech preview** (auth incomplete, Windows gaps, A/V sync basic) — [iroh-live](https://github.com/n0-computer/iroh-live) |
| **Browser watch path (live)** | Optional **iroh-live-relay** bridges to browsers via **WebTransport** | Demo-level; relay has **no auth yet** (per upstream README) |
| **`callme` / `iroh-roq`** | RTP-over-iroh style realtime audio | Demo / specialized |
| **Streaming docs page** | Points at live + callme — not a VOD product | https://docs.iroh.computer/protocols/streaming |

**Bottom line for LBRY-like catalogs (mostly VOD, not livestreams):**  
`iroh-blobs` + player glue is the correct substrate. `iroh-live` is interesting for **live** or real-time, not a drop-in for “watch this 2 h claim.”

### 2.4 Latency, seeking, adaptive bitrate — realistic bounds

| Concern | Realistic expectation |
|---------|----------------------|
| **Time-to-first-frame** | Dominated by: address lookup → connect (relay or direct) → first verified range covering init segment + first GOPs. Direct path: can be good. **Browser (relay-only):** extra RTT/hop and relay rate limits. Cold start is not “magic CDN edge” unless you put a warm cache provider near the user. |
| **Seek** | Good if container is **fragmented / has index** (fMP4, WebM cues) *and* you map times → byte ranges → blob ranges. Bad if you only have progressive MP4 with moov-at-end and no index. **Iroh range verification helps integrity, not container layout.** |
| **ABR** | Store multiple encodings as separate blobs (or HashSeq of renditions) + a small manifest blob; client switches. **You build this.** Live path has multi-preset hooks; VOD ABR is not a single Iroh API. |
| **Mobile / battery** | Native QUIC is fine; browser relay-only + WASM crypto has CPU/battery cost; needs measurement. |
| **Unknown / needs spike** | End-to-end VOD FPS and seek latency numbers for browser WASM + public relays; concurrent multi-source scheduling policies for video; production pin of iroh-blobs version. |

---

## 3. LBRY blobs today → Iroh-oriented revision

### 3.1 LBRY blob / sdblob responsibilities (as-is)

From [LBRY Protocol Spec — Data](https://spec.lbry.com/) and download overview:

```
Publish path:
  file → chunk (~2MiB-1 plaintext max) → AES-256-CBC + PKCS7 → content blob
       → SHA-384(blob) = blob_hash
  ordered list of {blob_hash, iv, length} + stream key + filename
       → canonical JSON = sd / manifest blob
       → SHA-384(manifest) = stream hash (sd_hash)
  claim on-chain: metadata + streamHash / source = sd_hash

Download path:
  claim → sd_hash
  DHT find peers for sd_hash → fetch sd blob → parse
  for each content blob_hash: DHT / peers / reflector → download → verify hash
  decrypt with stream key + per-blob IV → reassemble file
```

| Concern | LBRY mechanism |
|---------|----------------|
| **Hashing / integrity** | SHA-384 of each encrypted blob; stream identified by hash of manifest |
| **Encryption** | Per-stream AES-256 key; per-blob IV in manifest; optional key withheld until payment |
| **Ordering** | Ordered `blobs[]` in manifest |
| **Discovery** | Kademlia-like DHT (announce/find peers for hash) |
| **Transfer** | Blob exchange protocol (RPC); optional **reflectors** rehost + charge |
| **Identity in marketplace** | On-chain **claims** (name, metadata, fee, channel signature) |

**Important production reality (ecosystem, not pure protocol):** bulk availability has often depended on **datacenter reflectors / object storage**, not a dense seeder mesh. Any redesign should not assume “P2P alone will seed the long tail.”

### 3.2 Architecture sketch: Iroh-shaped revision

Keep the **marketplace/index** LBRY-shaped; replace the **data plane** with Iroh.

```
┌─────────────────────────────────────────────────────────────────┐
│  LBRY-NATIVE (stays)                                            │
│  • Claims / channels / fees / LBC payment proofs                │
│  • Human URLs, claimtrie resolution                             │
│  • Optional: Urma-like location claims (u-{source_claim_id})    │
│  • Content encryption keys for paid streams (app-layer)         │
└────────────────────────────┬────────────────────────────────────┘
                             │  points at content identity
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│  CONTENT IDENTITY (revised)                                     │
│  Option A (preferred for new content):                          │
│    BLAKE3 root of plaintext OR of encrypted envelope            │
│  Option B (compatibility):                                      │
│    HashSeq / collection of ~2MiB encrypted LBRY-style chunks    │
│    (preserve old blob hashes inside a new outer root)           │
│  Option C (dual):                                               │
│    claim carries both sd_hash (legacy) + iroh_hash (new)        │
└────────────────────────────┬────────────────────────────────────┘
                             │  BlobTicket / provider set / gossip
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│  IROH-NATIVE (new data plane)                                   │
│  • EndpointId dial-by-key                                       │
│  • iroh-blobs: range GET + verified streaming                   │
│  • Address lookup: DNS/Pkarr and/or Mainline DHT                │
│  • Relays: NAT help + e2e-encrypted fallback                    │
│  • Optional providers: community nodes, paid “reflectors”       │
│    reimplemented as always-online Iroh providers + disk/S3      │
└─────────────────────────────────────────────────────────────────┘
```

### 3.3 What stays LBRY-native vs becomes Iroh-native

| Responsibility | Stay LBRY / app | Move to Iroh |
|----------------|-----------------|--------------|
| Names, claims, stakes, channel identity | ✅ | |
| Fee address / proof of payment | ✅ | |
| AES content encryption for paid unlock | ✅ (app crypto; keys not in open sd blob if paid) | |
| Media container choice (mp4/webm, ABR ladder) | ✅ | |
| Content-addressed storage unit | (legacy SHA-384 blobs) | **BLAKE3 blobs / HashSeq** |
| Peer connectivity, NAT, multipath | | **Endpoint + relays** |
| Range/seek verified transfer | (whole-blob only historically) | **iroh-blobs ranges** |
| Provider discovery | DHT for blob hashes | **Tickets + address lookup + optional gossip of providers** |
| Reflector role | Upload RPC + S3 | **Always-on Iroh provider** (still may use S3 under the hood) |

### 3.4 Encryption: do not collapse two layers

| Layer | Purpose |
|-------|---------|
| **Iroh transport e2e** | Relay/path cannot read bytes in transit. Protects against on-path observers. |
| **LBRY content encryption** | Hosts can store ciphertext; key released after payment / policy. Protects against **storage hosts** and casual leeching. |

An Iroh redesign **must still support content encryption** if parity with paid LBRY streams matters. Transport e2e alone does **not** replace sdblob keys.

### 3.5 Migration path for existing `sd_hash` / content

Pragmatic phases:

1. **Gateway dual-serve (no claim migration)**  
   Offline job: for selected claims, download legacy blobs → reassemble (or re-encrypt under same key) → import into `iroh-blobs` → publish `BlobTicket` / root hash in **side index** (Urma claim, channel description JSON, companion API). Clients that understand Iroh use new path; others keep reflector/DHT.

2. **Claim value extension (soft fork of clients, not chain consensus)**  
   Metadata grows optional fields, e.g. `sourceType: iroh_blake3` + hash, or dual sources. Old clients ignore; new clients prefer Iroh.

3. **HashSeq of legacy encrypted blobs**  
   Keep SHA-384 list order; wrap as Iroh HashSeq/collection so **byte layout of ciphertext blobs stays compatible** while transfer becomes Iroh. Good if you already have blob farms and want transport upgrade without re-encrypt.

4. **Do not require re-hash of entire library before UX works**  
   Long-tail content can stay legacy until rehosted. Measure health first (catalog tools).

**Hard truth:** migration cost is **rehosting and operational**, not “change a hash algorithm in a PR.” Iroh does not invent free durability.

---

## 4. Relays (e2e) — model, trust, ops

### 4.1 How Iroh relays work

From [Relays](https://docs.iroh.computer/concepts/relays) + [What is iroh?](https://docs.iroh.computer/what-is-iroh):

1. **NAT traversal assist:** endpoints learn public addresses; exchange candidate info via home relay; attempt **hole punch** (QUIC NAT traversal extension).
2. **Encrypted traffic fallback:** if direct fails, bytes flow **through** the relay.
3. **Home relay:** endpoint probes configured relays (QAD), picks closest by latency, keeps a secure session (docs: WebSocket-class path to relay on the public internet / port 443 friendly patterns).
4. **Stateless facilitators:** relays do **not** store app data; scale by adding instances; failover by clients reconnecting.

Docs claim ~**9/10** network conditions allow a **direct** connection once punched; when it works and paths are stable, it keeps working.

### 4.2 What “e2e” means here

From [Security & privacy](https://docs.iroh.computer/concepts/security-privacy):

| Property | Reality |
|----------|---------|
| **Encryption endpoints** | Device ↔ device (QUIC/TLS). Relay **cannot decrypt** payload. |
| **Relay blindness** | Blind to file names, message contents, blob payloads. |
| **What relay still sees** | Endpoint identities on that relay, **which pairs** talk, **when**, **how much** volume it carried. IP metadata of clients connected to it. |
| **After go-direct** | Traffic leaves the relay; relay cannot reliably know total duration/bytes of the direct session. |
| **Public relays** | Free, shared, **rate-limited**, no SLA; n0 monitors abuse; not recommended for sensitive production. |
| **Dedicated relays** | Project-scoped; auth via short-lived tokens from API keys; isolation; optional SLA via Iroh Services; self-host open-source `iroh-relay`. |

**“e2e relays” ≠ Tor.** Single-hop path privacy is limited; IP privacy to the peer is **not** preserved on direct paths. Tor transport is an optional separate path both sides must support.

### 4.3 Abuse, cost, ops

| Issue | Notes |
|-------|--------|
| **Cost** | Bandwidth: video through relay is **expensive for the operator**. Design so **providers and desktop seeders go direct**; browsers may be relay-heavy. |
| **Rate limits** | Public relays will throttle; bulk VOD on public relays is a **non-starter** for production. |
| **Abuse** | Anyone can use public relays; operators block IPs. Dedicated + auth reduces freeloading. |
| **Legal/ops** | Relay is a **bit pipe**, not a content host — still attract attention if used to move infringing high-volume media. Policy, ToS, and jurisdiction still matter. |
| **vs LBRY reflectors** | Reflectors **store and serve content** (hot data plane). Iroh relays **should not store your videos**. Rehost role becomes **Iroh provider + disk/S3**, not the relay. |

### 4.4 Implications: browser / home NAT vs datacenter reflectors

| Client type | Connectivity pattern |
|-------------|----------------------|
| **Home NAT desktop/node** | Best case: punch to providers / other peers; relay only for intro + hard NATs. Ideal seed. |
| **Browser WASM** | Official limitation: **all connections via relay** today (no browser UDP punch). E2E crypto still holds; performance/cost depend on relay placement and limits. Future: WebRTC / WebTransport cert-hash may improve. |
| **Datacenter “reflector” equivalent** | Run **provider** with public EndpointId, fat disk/S3, optional auth for upload; use **dedicated relays** only as connectivity glue — do not confuse the two roles. |

---

## 5. WASM / browser path vs companion / node path

### 5.1 Can Iroh run in browser WASM today?

From [WebAssembly and Browsers](https://docs.iroh.computer/languages/wasm-browser) and [Iroh & the Web](https://www.iroh.computer/blog/iroh-and-the-web):

| Item | Status |
|------|--------|
| **Core `iroh` → wasm32 + wasm-bindgen** | **Yes** — documented; disable default features (`metrics` etc.) |
| **Examples** | browser-echo, browser-chat (gossip) live demos |
| **Direct P2P from browser** | **No** (documented): sandbox forbids raw UDP; **all paths via relay** |
| **iroh-gossip in browser** | Supported from gossip **0.33+** |
| **iroh-blobs in browser** | Roadmap lists **“iroh-blobs compiles to WASM”** among past work; tracking issue existed for full browser support — **treat large-media blobs as spike-required**, not “production VOD in pure WASM” |
| **npm package** | No official full WASM npm of iroh; recommend **app-specific rust wrapper** + wasm-bindgen |
| **Node/Deno** | Prefer **NAPI FFI** native iroh (full hole-punch), not WASM |
| **Threading** | WASM worker patterns needed for UI responsiveness; exact multi-thread story is app-dependent (SharedArrayBuffer COOP/COEP). **Needs spike.** |
| **Transports in browser** | Relay over WebSocket-class browser APIs today; future interest in WebRTC / WebTransport with constraints |

### 5.2 Pros / cons

| | **Native WASM in page** | **Companion / local node + HTTP bridge** (browser LBRY pattern) |
|--|-------------------------|------------------------------------------------------------|
| **Install** | Zero (or extension-only) | Installer / Tauri companion / lbrynet |
| **Connectivity** | Relay-only; no hole-punch | Full Iroh: direct + relay |
| **Seed back to network** | Weak / expensive (relay egress) | Strong |
| **CPU / battery** | WASM crypto + decode | Native codecs, better IO |
| **Wallet separation** | Easy (wallet module independent) | Easy (wallet vs blob modules stay separate) |
| **Publish** | Hard without provider upload API | Natural (local daemon) |
| **Trust** | User trusts extension + your relay choice | User trusts local binary (signing critical) |
| **Complexity** | WASM build, size, feature flags | Distribution, updates, origin lockdown |
| **Fit for browser LBRY clients** | Aspirational pure-web tier | **Matches common two-tier architecture** |

**Recommendation:** treat WASM as **progressive enhancement** for pure-web playback experiments; treat **companion/node** as the path for seed, publish, and reliable seek under NAT — same two-tier philosophy many browser LBRY clients use (public APIs vs local daemon).

---

## 6. Comparison table: Iroh-native web vs WebTorrent-style shoehorn

| Dimension | **Iroh-native web path** | **WebTorrent-style (BT + WebRTC)** |
|-----------|--------------------------|-------------------------------------|
| **Core idea** | Modern dial-by-key QUIC stack; protocols composed on Endpoint | Classic BitTorrent wire protocol; **replace TCP/uTP with WebRTC data channels** |
| **Browser transport** | Relay (WS-class) always; hope for WebRTC/WebTransport later | WebRTC P2P when possible; still needs **signaling** |
| **Discovery** | Address lookup (DNS/Pkarr, optional DHT); tickets; gossip | Magnet/infohash + **WS trackers** (and hybrid trackers) |
| **Integrity** | BLAKE3 verified streaming + ranges | BT piece hashes (SHA-1 historically / v2 Merkle in modern BT) |
| **Seek / streaming** | First-class **byte ranges** on blob tree | Piece prioritization + sequential download tricks; works for video but piece-oriented |
| **NAT** | Designed around punch + **e2e relay fallback** | ICE/STUN/TURN (often TURN cost if direct fails) |
| **Server costs** | Relays (connectivity) + optional providers (bytes). Public relays rate-limited. | Trackers (cheap) + TURN (expensive) + web seeds/HTTP fallbacks |
| **Corporate firewalls** | Port 443-friendly relay paths help | WebRTC often works; enterprise can still break ICE; TURN needed |
| **Mobile** | Native good; browser relay-limited | WebRTC mature for realtime; large VOD still heavy |
| **Interop with classic swarms** | N/A (different network) | Partial: web peers only talk to **WebRTC-capable** clients unless hybrid |
| **Content marketplace / claims** | Orthogonal — **bring your own** (LBRY/Urma) | Orthogonal — magnets have no on-chain naming |
| **Long-term bet** | Invest in **one** connectivity stack + verified content protocol | Forever adapt **1980s–2000s BT assumptions** to each new browser constraint |
| **When wrong long-term** | If you only need casual one-off file sharing and already live in BT ecosystem | If you need dial-by-key, range-verified multi-provider VOD, clean relay economics, and web+native **same protocol** without dual peer classes |

**When “shoehorn classic P2P into browser transports” is the wrong long-term bet**

- You are building a **new** content identity + marketplace (LBRY claims / Urma), not trying to leech existing public torrent swarms.
- You care about **verified range reads** of multi-GB media with modern hashing (BLAKE3), not piece markets.
- You want **one** stack for desktop, mobile, and (relayed) browser — not “BT for native, WebTorrent dialect for web.”
- You expect **hard NATs and mobile carriers** where a first-class **encrypted relay fallback** is part of the design, not an afterthought TURN bill.
- You want providers to be **content-addressed endpoints**, not only members of a swarm for one infohash.

WebTorrent remains the right answer if the goal is **“play this existing torrent in a tab.”** That is not the LBRY redesign problem.

---

## 7. Recommended direction

### 7.1 Prototype first (concrete, small)

1. **Native spike (1–2 weeks of engineering focus)**  
   - Import a sample video into `iroh-blobs` (FsStore).  
   - Second machine / container: range-fetch middle of file; measure seek-to-play with a minimal fMP4 or full MP4 + index.  
   - Document RTT direct vs forced relay.

2. **Claim pointer demo**  
   - Hardcode or publish a **side claim / JSON** mapping `claim_id → blake3 + provider EndpointId/ticket`.  
   - No chain consensus change required.

3. **Encryption parity demo**  
   - Encrypt media under AES (or keep LBRY chunk encryption); store **ciphertext** as blob; prove key gate separate from Iroh transport keys.

4. **Browser LBRY client integration sketch (design only)**  
   - Player path: resolve claim → if `iroh` source present → companion/node fetches → local HTTP range to `<video>`.  
   - Pure web: optional WASM or HTTPS gateway provider (honest about centralization).

5. **Cost model note**  
   - Estimate relay egress $ for browser-only audience vs provider+direct desktop seeders.

### 7.2 What to avoid

| Avoid | Why |
|-------|-----|
| **“Port LBRY DHT + blob exchange into WebRTC”** | Recreates WebTorrent’s dual-world complexity without BT’s swarm benefits |
| **Assuming public n0 relays will host your video CDN** | Rate limits, shared abuse domain, no SLA, wrong economic model |
| **Collapsing content encryption into transport e2e** | Breaks paid content / untrusted hosts model |
| **Betting production UX on iroh-live** | Live stack is early; LBRY catalog is mostly VOD |
| **Big-bang rehash of entire library** | Operational suicide; dual-read + triage instead |
| **Pure WASM as the only path** | Relay-only + maturity gaps; keep companion tier |

### 7.3 Suggested target architecture (north star)

```
[Browser extension UI]
   │ resolve claim (SPV/API)
   │
   ├─(optional) WASM iroh: relayed get of init+GOP ranges ──► [dedicated project relays]
   │
   └─(preferred) Companion / node: full iroh-blobs provider+client
            │
            ├─ direct QUIC to seeders / community providers
            └─ S3-backed always-on provider(s)  ("reflector 2.0")
```

Wallet, claim publish, and blob transport remain **separate modules** (aligns with liblbry / lbry-sdk / browser-client layering).

---

## 8. Open questions for pcfreak30 / community

1. **Content identity:** Prefer **single BLAKE3 of whole file**, **HashSeq of ~2 MiB encrypted chunks** (LBRY-compatible), or dual claims (`sd_hash` + `iroh_hash`)?
2. **Paid streams:** Keep AES key in encrypted channel (payment unlock) while hosts only see ciphertext — confirm product requirement for web clients.
3. **Provider economics:** Who pays for always-on Iroh providers vs pure altruistic seed? Any tie-in to LBC / data markets?
4. **Urma multi-backend:** Should Iroh be **one Location backend** among Sia/etc., or the **primary transfer fabric** with Sia as cold store behind providers?
5. **Browser ambition level:** Relay-only WASM playback acceptable for v1 pure-web, or is companion mandatory for video?
6. **iroh-blobs version pin:** Latest line says not production quality — which version do we standardize on for experiments?
7. **Relay ops:** Self-host dedicated relays for community clients / LumeWeb, or Iroh Services, or both?
8. **Legal posture of providers vs relays:** Community-run video providers reintroduce the classic hosting liability surface; is the project aiming for pure client mesh or “many small reflectors”?
9. **Interop with Odysee/reflector.go world:** Dual-read only, or active rehost pipeline from existing reflector dumps?
10. **Spam/poisoning:** How to prevent junk EndpointIds advertising false possession of hashes (need inventory proofs / trust / staking)?

---

## 9. Optional short Discord reply

See [DISCORD-REPLY.md](./DISCORD-REPLY.md) for a paste-ready half-page note for pcfreak30.

---

## Appendix A — Quick glossary

| Term | Meaning |
|------|---------|
| **sd_hash / stream hash** | LBRY SHA-384 of the stream descriptor (manifest) blob |
| **BlobTicket** | Iroh shareable “this hash + where to dial” string |
| **Outboard / BAO** | Side metadata for BLAKE3 verified streaming without rewriting the blob |
| **HashSeq** | Blob that is a sequence of 32-byte hashes |
| **Reflector (LBRY)** | Host that accepts uploads and rehosts blobs |
| **Relay (Iroh)** | Blind encrypted path + NAT assist; not content storage |

## Appendix B — Source snapshot note

Primary Iroh pages fetched 2026-08-11 from docs.iroh.computer and github.com/n0-computer. LBRY from spec.lbry.com. Details and version numbers will drift — re-check before implementation. Full link list: [SOURCES.md](./SOURCES.md).
