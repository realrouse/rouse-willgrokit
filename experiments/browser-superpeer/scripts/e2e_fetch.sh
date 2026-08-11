#!/usr/bin/env bash
# One-shot CLI e2e: start superpeer, fetch, compare to source, stop.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
BIN="${BIN:-./target/release/browser_superpeer}"
if [[ ! -x "$BIN" ]]; then
  cargo build --release
fi
"$BIN" superpeer --blobs fixtures/demo > /tmp/bsp-sp.log 2>&1 &
echo $! > /tmp/bsp-sp.pid
cleanup() { kill "$(cat /tmp/bsp-sp.pid)" 2>/dev/null || true; }
trap cleanup EXIT
for _ in $(seq 1 60); do
  grep -q 'ticket  =' /tmp/bsp-sp.log && break
  sleep 0.25
done
TICKET=$(sed -n 's/^  ticket  = //p' /tmp/bsp-sp.log)
SD=$(python3 -c "import json;print(json.load(open('fixtures/demo/DEMO.json'))['sd_hash'])")
"$BIN" fetch --ticket "$TICKET" --sd-hash "$SD" --out /tmp/bsp-assembled.wav
cmp -s /tmp/bsp-assembled.wav fixtures/source_demo.wav
echo "e2e OK: assembled matches source ($(wc -c < fixtures/source_demo.wav) bytes)"
