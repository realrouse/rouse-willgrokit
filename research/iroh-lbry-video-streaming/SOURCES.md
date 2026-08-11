# Sources (accessed / consulted 2026-08-11)

## Iroh (primary)

| Source | URL | Notes |
|--------|-----|--------|
| What is iroh? | https://docs.iroh.computer/what-is-iroh | Architecture: Endpoint, Router, protocols, relays |
| Docs index (llms.txt) | https://docs.iroh.computer/llms.txt | Full doc map |
| Blobs protocol | https://docs.iroh.computer/protocols/blobs | BLAKE3, verified streaming, range requests, collections |
| Streaming protocols | https://docs.iroh.computer/protocols/streaming | iroh-live, callme, iroh-roq |
| Relays | https://docs.iroh.computer/concepts/relays | Roles, public vs dedicated, auth |
| Security & privacy | https://docs.iroh.computer/concepts/security-privacy | E2E encryption, what relays can see |
| WASM / browsers | https://docs.iroh.computer/languages/wasm-browser | Relay-only browser path, limitations |
| Iroh & the Web (blog, 2024-07-01) | https://www.iroh.computer/blog/iroh-and-the-web | Roadmap phases: WS → WASM → full stack → beyond WS |
| Roadmap | https://www.iroh.computer/roadmap | Past: iroh-blobs compiles to WASM; 1.0 work |
| iroh-blobs crates.io / docs.rs | https://docs.rs/iroh-blobs/latest/iroh_blobs/ | v0.103.0 (2026-06-15); BAO verified streams; ranges; **not yet “production quality” note on latest** |
| iroh-blobs repo | https://github.com/n0-computer/iroh-blobs | Examples, transfer |
| iroh-live | https://github.com/n0-computer/iroh-live | MoQ live A/V; early tech preview; browser WebTransport relay |
| browser-echo / browser-chat examples | https://github.com/n0-computer/iroh-examples | Live demos exist |
| iroh-blobs WASM tracking | https://github.com/n0-computer/iroh-blobs/issues/90 | Browser build tracking (opened 2025-05) |

## LBRY (primary)

| Source | URL | Notes |
|--------|-----|--------|
| LBRY Protocol Spec | https://spec.lbry.com/ | Blobs ≤2MiB, AES-256-CBC, SHA-384, manifest/stream hash, DHT, blob exchange, reflectors |
| Content downloading (lbry.tech) | https://lbry.tech/resources/download-overview/ | sd_hash → DHT → sd blob → content blobs |

## WebTorrent (comparison)

| Source | URL | Notes |
|--------|-----|--------|
| Mozilla Hacks: WebTorrent | https://hacks.mozilla.org/2018/08/dweb-building-a-resilient-web-with-webtorrent/ | BT over WebRTC; tracker protocol changes; wire protocol same after connect |
| webtorrent.io | https://webtorrent.io/ | Project home; browser BitTorrent via WebRTC |

## Public ecosystem context (secondary; not gospel)

| Source | URL | Role |
|--------|-----|------|
| Urma | https://urma.xyz/ | On-chain pointers to off-chain storage |
| liblbry | https://github.com/LumeWeb/liblbry | Go LBRY protocol library |
| @lumeweb/lbry-sdk | https://www.npmjs.com/package/@lumeweb/lbry-sdk | TypeScript LBRY SDK for web/JS |
| LBRY reflector (prism) | https://github.com/lbryio/reflector.go | Historical reflector / hosting stack |

## Uncertainty flags

- Exact production maturity of **iroh-blobs in browser WASM** for large media: docs say core `iroh` + gossip work; blobs WASM listed on roadmap as done for compile, but full browser media pipeline still needs a **spike**.
- iroh-live is explicitly **early tech preview** (auth gaps, platform gaps).
- Latest docs.rs `iroh-blobs` notes **not yet production quality** for the newest major line (use older pin for production if needed — verify current pin before shipping).
