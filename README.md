# rouse-willgrokit

Public research write-ups produced with Grok for LBRY-related questions and community discussion.

**Repo:** https://github.com/realrouse/rouse-willgrokit

## Contents

| Topic | Date | Summary |
|-------|------|---------|
| [Iroh × LBRY blobs × WASM web](./research/iroh-lbry-video-streaming/) | 2026-08-11 | Using Iroh for video streaming and what a revision of LBRY’s blob system could look like with e2e relays and browser WASM, vs WebTorrent-style BT-over-WebRTC |
| [Unification of pcfreak30’s ideas → MVP](./research/unification-of-pcfreak30-ideas/) | 2026-08-11 | Digest of pcfreak30’s Iroh/web chat notes, unified with the prior report, assessment, and an MVP-shaped gap-fill in complete sentences |
| [Scoped MVP: browser + download superpeer](./research/scoped-mvp-browser-superpeer/) | 2026-08-11 | Experiment definition: keep LBRY blobs; browser leecher + Iroh superpeer as download peer (not reflector); no chain work; browser CDN is phase two |
| [**Executable experiment**](./experiments/browser-superpeer/) | 2026-08-12 | Working Rust superpeer + companion web UI + demo fixtures (LBRY-shaped blobs over Iroh) |
| [LBRY Foundation public voice & growth](./research/lbry-foundation-public-voice/) | 2026-08-11 | Discord as HQ, X as public square; nonprofit content doctrine, sample posts, and legibility-first growth ideas for `@LBRYFoundation` |

## Publishing rules

This repository is **public**. Only material that is:

1. **LBRY / LumeWeb / related open protocol ecosystem** content, and  
2. **safe for public sharing** (no secrets, no private infrastructure, no unrelated personal projects)

…belongs here.

**Never commit:** API tokens, private keys, wallet seeds, `.env` files with credentials, internal hostnames/paths, or non-LBRY personal project material.

## License

Unless noted otherwise in a topic folder, text is shared for discussion under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Cite sources; check upstream licenses for third-party material.
