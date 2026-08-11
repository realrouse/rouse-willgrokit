# Scoped MVP: browser end user + LBRY download superpeer over Iroh

| Field | Value |
|-------|--------|
| **Topic ID** | `scoped-mvp-browser-superpeer` |
| **Date** | 2026-08-11 (addendum 2026-08-12) |
| **Type** | Experiment definition / scoped MVP (P2P only) |
| **Primary document** | [EXPERIMENT-MVP.md](./EXPERIMENT-MVP.md) |
| **Builds on** | [iroh-lbry-video-streaming](../iroh-lbry-video-streaming/), [unification-of-pcfreak30-ideas](../unification-of-pcfreak30-ideas/) |
| **Requested as** | Experiment for rouse / community; pcfreak30 “go” |

## One-line scope

Keep the **LBRY blob system**. Solve **how a browser gets those blobs** from a **superpeer that behaves like a LBRY download peer**, using **Iroh (including e2e relays)** as the web-friendly path. **No new blockchain work** in this experiment. **Browser-to-browser CDN** is explicitly a later step.

## Implementation status

**Executable prototype:** [`experiments/browser-superpeer`](../../experiments/browser-superpeer/) (Rust superpeer + companion web UI + demo fixtures). CLI e2e verified: packed LBRY-shaped blobs round-trip over Iroh with hash match to source.

## Addendum (language boundaries)

liblbry / `@lumeweb/lbry-sdk` can help **blob verify and parse**; Iroh stays the transport. Prefer **not** a hot-path **WASM → JS → WASM** sandwich. See **Section 3.5** in the main doc. This does not change experiment goals—only how you assemble verify vs networking.
