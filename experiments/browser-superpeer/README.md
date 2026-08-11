# Experiment: browser end user + LBRY download superpeer (Iroh)

Executable MVP for the scoped experiment defined in  
[`research/scoped-mvp-browser-superpeer`](../../research/scoped-mvp-browser-superpeer/).

**What it proves**

1. LBRY-shaped blobs (SHA-384 + AES-256-CBC + stream descriptor) are preserved.  
2. A native **superpeer** serves them as a **download peer** (not an upload reflector).  
3. Traffic uses **Iroh** (direct and/or e2e relayed).  
4. A **browser** can play media via a small **localhost companion** (no classical `lbrynet`, no WASM→JS→WASM).  
5. No blockchain work is required for the demo.

## Requirements

- Rust 1.91+ (see `iroh` crate)  
- Network access to public Iroh relays (default N0 preset) for first connect

## Quick demo

```bash
cd experiments/browser-superpeer

# Already packed demo pack under fixtures/demo (35s tone WAV, 2 content blobs).
# Re-pack from source if you want:
#   cargo run --release -- pack --input fixtures/source_demo.wav --out fixtures/demo

# Terminal 1 — superpeer
cargo run --release -- superpeer --blobs fixtures/demo
# copy the printed "ticket" line

# Terminal 2 — companion (browser bridge)
cargo run --release -- companion
# open http://127.0.0.1:8787

# Or CLI assemble (Slice B) without the browser:
cargo run --release -- fetch \
  --ticket 'PASTE_TICKET' \
  --sd-hash "$(python3 -c "import json;print(json.load(open('fixtures/demo/DEMO.json'))['sd_hash'])")" \
  --out /tmp/out.wav
```

`fixtures/demo/DEMO.json` holds `sd_hash` and `stream_key` for the demo pack.

## Commands

| Command | Role |
|---------|------|
| `pack` | Encrypt/split a file into LBRY-shaped blobs + sd |
| `superpeer` | Serve a blob directory over Iroh |
| `fetch` | CLI download + verify + decrypt + write file |
| `companion` | HTTP API + static web UI on `127.0.0.1:8787` |

## Layout

```
src/           Rust binary (Iroh + LBRY blob crypto + axum companion)
web/           Browser UI (calls companion only)
fixtures/      Demo pack (redistributable synthetic tone)
docs/PROTOCOL.md
```

## Deliberately skipped (phase two+)

- Browser-to-browser re-share CDN  
- Pure WASM Iroh in the tab  
- Claim / wallet / chain resolve  
- Upload reflector ingestion  
- Tor/Nym privacy transports  
- Multi-superpeer discovery beyond pasteable tickets  

## License

MIT OR Apache-2.0 (same as typical Rust dual license; experiment code).
