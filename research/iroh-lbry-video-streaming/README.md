# Iroh video streaming × LBRY blob revision × WASM web

| Field | Value |
|-------|--------|
| **Topic ID** | `iroh-lbry-video-streaming` |
| **Context** | Research requested for community discussion (pcfreak30 → rouse) |
| **Date** | 2026-08-11 |
| **Type** | Read-only technical research report |
| **Primary deliverable** | [REPORT.md](./REPORT.md) |
| **Discord blurb** | [DISCORD-REPLY.md](./DISCORD-REPLY.md) |
| **Sources index** | [SOURCES.md](./SOURCES.md) |

## Scope

Research **using Iroh for video streaming** and what a **revision of LBRY’s blob system** could look like if built on:

1. **Iroh** (including **e2e relays**)
2. **Native WASM** in the browser

…versus **WebTorrent-style** “shoehorn classic P2P (UDP/BT) into browser transports (WebRTC).”

## Not in scope

- Implementation, deploys, or product code
- Live network abuse / secret material
- Treating informal ecosystem notes as gospel (primary sources preferred)

## Ecosystem context (public)

Relevant public LBRY / LumeWeb threads this report may inform:

- Browser LBRY clients (e.g. extension + optional local daemon / companion patterns)
- [Urma](https://urma.xyz/) — on-chain pointers to off-chain storage
- [liblbry](https://github.com/LumeWeb/liblbry) — Go LBRY stack
- [@lumeweb/lbry-sdk](https://www.npmjs.com/package/@lumeweb/lbry-sdk) — TypeScript LBRY SDK

## Status

**Complete** — full report in `REPORT.md`.
