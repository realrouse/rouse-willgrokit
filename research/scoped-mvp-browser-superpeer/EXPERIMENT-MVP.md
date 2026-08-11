# Experiment MVP: browser end user and LBRY download superpeer over Iroh

**Date:** 2026-08-11  
**Status:** Scoped experiment definition (not an implementation)  
**Audience:** pcfreak30, rouse, anyone who wants a buildable prototype boundary  
**Style:** Complete sentences throughout.

This document answers a concrete ask: define an experiment where Grok (or a human implementer) can prototype the **P2P path only**, with a **browser end user** on one side and a **superpeer** on the other. The superpeer is modeled as a **LBRY download peer** (it serves content blobs you already have or can obtain), **not** as a classical **upload reflector** that accepts arbitrary publisher uploads into a hosting business.

It also corrects an earlier over-read: **this experiment does not advocate deleting or replacing the LBRY blob system.** The blob model (stream descriptors, content blobs, hashes, encryption where present) stays. What changes is **how those bytes move to a browser**, using a stack that already treats **relayed web paths** and **direct native paths** as first-class, instead of re-implementing 2010s UDP peer assumptions inside the browser sandbox.

---

## 1. Intent, in pcfreak30’s terms (paraphrased carefully)

The shared view is roughly this. Almost every peer-to-peer system people still like eventually runs into the same **browser versus UDP** problem. Designers then invent dual stacks, weird listeners, or “web peers” that only talk to other web peers. Iroh’s approach is more useful as a starting point: **connectivity is a product**, with **end-to-end encrypted relays** when direct paths fail, and native nodes that can still go direct when they can.

Nobody is pretending relays remove the need for **superpeers**. The thought process is to make **ordinary end users** able to participate without a heavy desktop install, while still having **well-connected nodes** that hold and serve blob data. If a resilient “cockroach” style node operated on this fabric, viewers could rely on **service workers and ordinary web clients**, and attempting to erase connectivity by attacking a relay would look more like attacking a **public utility** (Tor- or IRC-shaped infrastructure) than like taking down a single media host. That is a political and operational intuition, not a legal guarantee; the experiment should not claim DMCA immunity. It should still design roles so that **relays forward encrypted traffic** and **content custody sits on peers that chose to serve blobs**.

Monetization remains an open problem and is **out of scope** for this experiment. The point of the experiment is technical proof that the **browser path plus download superpeer** works with real LBRY blob data.

**Secondary step (explicitly not MVP):** a **P2P CDN** where browser leechers re-share the same video files to other browsers. That is desirable later. It is not required to call the first experiment a success.

**Blockchain (explicitly not MVP):** the experiment **does not invent claim resolution, wallet flows, or chain connections**. You may hard-code a claim id, paste an `sd_hash`, or drop a local stream descriptor file. Solving “find metadata on chain” is a different project.

---

## 2. What is in scope and what is not

### 2.1 In scope

1. **Preserve the LBRY blob system.** Stream descriptors (sd blobs), content blob hashes, ordering, and decryption rules remain the content model. Do not redesign encryption or invent a new hash identity for the experiment unless a tiny adapter is unavoidable for transport framing.

2. **Browser end user.** A person opens a web page or extension surface, points it at content identity (`sd_hash` or equivalent), and receives enough verified blobs to play or save media **without** running classical `lbrynet` on that machine.

3. **Superpeer as LBRY download peer.** A long-running native process (Rust preferred for Iroh; Go is acceptable if bindings are easier for the implementer) that:
   - already has, or can load, a set of LBRY blobs;
   - answers requests for those blobs over Iroh;
   - behaves like a peer you would have asked “do you have this blob hash?” in the old world;
   - does **not** need to implement reflector-style “upload your whole publish pipeline to me” for the experiment.

4. **Iroh connectivity.** Dial by endpoint identity, use relays for browser and hard NAT cases, prefer direct paths between native nodes when available. Relays must be treated as **e2e encrypted bit pipes**, not as content hosts.

5. **Verification.** The browser (or a thin WASM/helper path) checks that received blob bytes match the expected LBRY blob hashes before decrypt or play.

6. **Minimal provider advertisement.** Enough for the browser to know **which superpeer to dial** for this experiment (pasteable ticket, config file, or one-line endpoint id). Full decentralized discovery can wait.

### 2.2 Out of scope (for this experiment)

1. **New blockchain protocol work**, claimtrie changes, payment channels, or mandatory SPV wallet integration.  
2. **Upload reflector product** (accepting publisher uploads, pricing, storage markets). A superpeer may *internally* obtain blobs however it likes (disk cache, old client, manual copy); that is operator plumbing, not the experiment API.  
3. **Browser-to-browser re-sharing CDN** as a success requirement.  
4. **Tor / Nym** privacy modes as default path (optional later).  
5. **Global catalog search**, recommendation feeds, or Odysee replacement UX.  
6. **Monetization**, token incentives, or business model.  
7. **Deleting or replacing** sdhashes / the blob format as a goal.  
8. **Production hardening**, multi-region ops, or legal policy finalization (note risks; do not block the prototype on them).

---

## 3. Roles and trust boundaries

### 3.1 Browser end user

The browser client is a **leecher** in BitTorrent vocabulary: it wants blobs, verifies them, decrypts when keys are available, and plays media. In the MVP it is **not required** to re-export those blobs to other browsers.

Responsibilities:

- Accept an **`sd_hash`** (or a pre-fetched sd blob) and a **superpeer ticket / endpoint**.  
- Request the stream descriptor if needed, then request content blobs in playback order (or a simple sequential strategy).  
- Verify hashes.  
- Decrypt with the stream key if the demo content is encrypted and the key is provided in config for the experiment.  
- Present media in a normal `<video>` element or download a file.

Implementation options, in preferred order for learning:

1. **Native Iroh in a local companion** that the page talks to over localhost HTTP, while the UI stays in the browser. This is often the fastest way to prove the *protocol* if pure WASM Iroh is still painful for large media.  
2. **Browser WASM Iroh** talking through relays to the superpeer, if the stack is ready enough for the implementer.  

Either option still counts as a **browser end user** experience if the human never installs classical LBRY desktop. A companion binary is allowed as scaffolding for the experiment, as long as it is small and purpose-built for Iroh blob fetch, not a full historical daemon.

### 3.2 Superpeer (download peer, not upload reflector)

The superpeer is a **native Iroh endpoint** that serves LBRY blob bytes it possesses.

It is like a helpful peer in a swarm, not like a publish gateway:

| Superpeer (this MVP) | Classical LBRY reflector (not this MVP) |
|----------------------|----------------------------------------|
| Answers “give me blob H if you have it” | Accepts uploads of new publishes at scale |
| Inventory is what it already stores | Ingestion pipeline + object storage business |
| Can be run by a hobbyist with a disk full of blobs | Operator product with uptime and abuse desk |
| Success = correct bytes to leechers | Success = publishers can dump content forever |

How the superpeer **obtained** the blobs is intentionally flexible for the experiment: copy from an existing cache, fetch once with any existing tool offline, or ship a demo pack of blobs next to the binary. The public story is only: **it serves LBRY blobs over Iroh to downloaders.**

### 3.3 Relay

An Iroh relay (public for toy tests, dedicated for a serious demo) helps the browser and superpeer find a path. It should **not** store LBRY blobs and **cannot** read payload content if e2e encryption is working as designed. Operators still see metadata such as who connected and how much volume crossed the relay. The experiment should use **rate limits and abuse basics** even in demo mode so a shared relay is not wrecked by accident.

### 3.4 Optional later: browser leecher as mini-peer

After the MVP works, a second experiment can allow a browser that already holds verified blobs to **offer them** to other browsers (true web mesh / P2P CDN behavior). That step needs careful product and legal thinking, plus likely service worker storage quotas. It is **phase two**, not phase one.

---

## 4. Content model (keep the blob system)

The experiment speaks fluent LBRY data:

1. **Stream descriptor (sd blob)** identified by **`sd_hash`**.  
2. Descriptor lists **content blob hashes**, order, lengths, and per-blob IVs when encryption is used.  
3. **Content blobs** are opaque ciphertext or plaintext chunks as in the existing model.  
4. Client **verifies** each blob against its hash before use.  
5. Client **decrypts** with the stream key when required for playback.

Iroh is the **carrier**. A practical framing for the prototype:

- Map each LBRY `blob_hash` to a **retrievable object** on the superpeer (for example, store the raw blob file keyed by hex hash).  
- Use Iroh connections (and, if helpful, `iroh-blobs` or a tiny custom ALPN) to request “bytes for hash H.”  
- If using `iroh-blobs` natively, you may wrap each LBRY blob as an opaque blob whose **BLAKE3** is *not* the LBRY hash; in that case the protocol must still carry or bind the **LBRY hash** so verification against the sd blob remains correct. The simplest MVP approach is often: **custom request/response for “LBRY blob hash H”** over an Iroh QUIC stream, without forcing BLAKE3 to replace SHA-384 identities.

The important design rule: **LBRY content identity stays LBRY content identity.** Iroh does not have to become the new global naming scheme inside this experiment.

---

## 5. Wire story for the happy path

### 5.1 Human demo script

1. Operator starts **superpeer** with a demo blob pack for one known `sd_hash`.  
2. Superpeer prints an **Iroh ticket** or endpoint address.  
3. User opens the **web UI**, pastes ticket + `sd_hash` (and stream key if needed for the demo pack).  
4. UI fetches sd blob, verifies it, then fetches content blobs in order (or parallel with a small concurrency limit).  
5. UI decrypts and plays.  
6. Optionally, the same UI works a second time after a refresh, still without classical LBRY installed.

### 5.2 Logical messages (implementer-facing)

These names are suggestions, not a frozen standard:

- `Have(hash) -> bool` — optional inventory check.  
- `GetBlob(hash) -> bytes | error` — main path.  
- Optional `GetBlobRange(hash, start, end)` — only if you need seek before full download; nice to have, not required for first play of a short demo file.

All of the above run **inside an Iroh connection** between browser path and superpeer, with relays as needed.

### 5.3 Failure modes the MVP must handle gracefully

- Superpeer does not have a hash: clear error, not a hang.  
- Hash mismatch: discard bytes, show verification failure.  
- Relay timeout: retry or surface “network path failed.”  
- Missing stream key for encrypted demo content: explain that decryption key was not provided (do not pretend transport e2e replaces content keys).

---

## 6. Success criteria (yes/no checklist)

The experiment succeeds if all of the following are true in a recorded run:

1. **Blob system preserved.** The demo content is real LBRY-shaped blobs (sd + content blobs with correct hashes), not a one-off MP4 over a random socket with no LBRY identity.  
2. **No classical LBRY desktop required on the viewer.** The human uses a browser (and at most a small Iroh companion if that was the chosen scaffolding).  
3. **Superpeer is a download peer.** It serves blobs it has; the demo does not depend on implementing reflector upload APIs.  
4. **Path is Iroh-based.** Traffic between client stack and superpeer uses Iroh connectivity (direct and/or e2e relayed).  
5. **Verification happens.** Tampering with bytes on the superpeer disk causes the client to reject the blob.  
6. **Playback or full file assembly works** for at least one stream of meaningful size (minutes of video, not only a tiny text blob).  
7. **No blockchain feature was required** to complete the demo (hard-coded `sd_hash` / local descriptor is allowed).  
8. **Browser-to-browser sharing was not required** for success.

---

## 7. Suggested build slices (so a prototype can actually start)

### Slice A — Superpeer skeleton

- Load a directory of blob files named by hash.  
- Run an Iroh endpoint.  
- Implement `GetBlob`.  
- Print a ticket.  
- Integration test: second native process fetches and verifies one blob.

### Slice B — Stream assembly without UI

- Given `sd_hash`, fetch sd blob, parse JSON, fetch all content blobs, verify, decrypt if needed, write output file.  
- Prove the blob system is intact end to end over Iroh.

### Slice C — Browser end user

- Minimal page: inputs for ticket, `sd_hash`, optional key.  
- Either WASM Iroh or companion HTTP bridge.  
- Progress UI and `<video>` or download link.  
- This is the slice that makes the experiment legible to non-protocol people.

### Slice D — Notes for others

- One markdown page: how to run superpeer, how to pack demo blobs, how to run the page, what was deliberately skipped (CDN phase, chain, reflector uploads).

Stop after Slice C+D unless energy remains. Do not start browser mesh CDN until A–D are boringly reliable.

---

## 8. Phase two (named so it does not creep into phase one)

**Browser leecher re-share / P2P CDN.** After a browser has verified blobs, it may advertise them to other browsers through the same Iroh fabric (likely still relay-mediated). Goals would include reduced superpeer bandwidth and more “torrent-like” cooperation among end users. Risks include storage quotas, tab lifetime, relay cost, and legal framing of who is distributing what. Treat this as a **separate experiment document** when someone is ready.

**Privacy transports.** Tor or Nym modes for users who want stronger network-level privacy. Both ends must support the transport. Not required for the first public demo.

**Chain glue.** Resolve `lbry://` to `sd_hash` via existing APIs or light clients. Valuable product work; orthogonal to proving the P2P pipe.

**Many superpeers.** Provider lists, simple health checks, and fallback if one peer lacks a blob. Natural extension once one superpeer works.

---

## 9. Risks to state out loud (without expanding scope)

**Relays are not magic legal shields.** Calling a relay a public utility is a useful design metaphor for resilience and role separation. It is not advice that operators are free of law or policy. Content-hosting risk still attaches primarily to whoever stores and serves clear or encrypted blobs at rest, and to whoever publishes them.

**Superpeer centralization risk.** If only one operator ever runs a superpeer, the experiment still proves the pipe, but it does not yet prove a healthy mesh. That is acceptable for an MVP.

**Relay cost.** Browser traffic that never goes direct will cost someone bandwidth. Dedicated demo relays or strict rate limits keep the experiment from depending on shared public infrastructure as a free CDN.

**WASM maturity.** If pure browser Iroh cannot yet move large media comfortably, the companion bridge is a valid experimental scaffold, not a moral failure.

---

## 10. Relationship to earlier write-ups

The [Iroh × LBRY research report](../iroh-lbry-video-streaming/REPORT.md) established that Iroh is a reasonable data-plane candidate and that WebTorrent-style shoehorning is the wrong long-term definition of success for this problem.

The [unification document](../unification-of-pcfreak30-ideas/SYNTHESIS-AND-MVP.md) merged that research with pcfreak30’s broader strategic notes (appchain focus, web-first usage, sunk cost on UDP-era designs). Some wording there leaned toward “deprecate the historical P2P stack” as a north star. **This experiment document narrows and corrects the implementation stance:**

- **Do not delete the blob system.**  
- **Do** build a **browser-friendly path** that carries those blobs.  
- **Do** use a **download superpeer**, not a reflector product, for the first prototype.  
- **Do not** require blockchain invention or browser CDN in the first prototype.

If the two docs appear to disagree, **this scoped experiment wins for what to build next.** The strategic essay remains useful for long-term product philosophy.

---

## 11. One-paragraph summary for Discord

We should run a tight experiment that keeps LBRY’s blob and sdhash model exactly as content identity, and only replaces the painful part: getting those blobs into a browser without classical desktop LBRY. A native superpeer acts as a normal download peer that already has blobs and serves them over Iroh, including through end-to-end encrypted relays when the browser cannot punch UDP. The browser (or a tiny companion that still feels like a web workflow) verifies hashes and plays media. No new chain protocol, no upload reflector product, and no requirement that browsers re-seed each other yet. If that demo works, we have proven the secondary web-friendly P2P path; browser-to-browser CDN and privacy transports can be later experiments.

---

## 12. Suggested repository layout for a future prototype (informational only)

This research repo does not have to contain the code. If someone starts an implementation elsewhere, a clean layout would be:

```text
browser-superpeer-experiment/
  docs/PROTOCOL.md
  superpeer/          # native Iroh endpoint, blob directory
  web/                # page or extension UI
  companion/          # optional localhost bridge
  fixtures/           # demo sd_hash + blobs (redistributable only)
  scripts/demo.sh
```

No code is required to accept this experiment definition as complete.
