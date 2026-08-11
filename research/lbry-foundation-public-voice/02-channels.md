# Channels: Discord as HQ, then the public record

Most of LBRY’s **day-to-day activity and development conversation** already happens on the **LBRY Foundation Discord**. That is a feature of the community — not a bug — but it creates a **visibility gap**: work that only lives in Discord is invisible to search engines, journalists, historians, and “what is happening with LBRY?” queries.

This document proposes a clear **channel hierarchy** so Discord stays the living room while other surfaces keep the public record honest.

## Channel map

```text
                    ┌─────────────────────────┐
                    │     Real-time work      │
                    │  Discord (HQ / shop)    │
                    │  chat · design · debug  │
                    └───────────┬─────────────┘
                                │
                    promote durable outcomes ↓
                                │
          ┌─────────────────────┼─────────────────────┐
          ▼                     ▼                     ▼
   ┌─────────────┐      ┌─────────────┐       ┌─────────────┐
   │   Forum     │      │  lbry.org   │       │   GitHub    │
   │  long-form  │      │  home/docs  │       │  code/tags  │
   │  releases   │      │  downloads  │       │  issues/PRs │
   └──────┬──────┘      └──────┬──────┘       └──────┬──────┘
          │                    │                     │
          └────────────────────┼─────────────────────┘
                               │
                    point / teach / invite ↓
                               │
                    ┌──────────▼──────────┐
                    │   X / Reddit / TG   │
                    │  public square      │
                    │  low volume, high   │
                    │  trust pointers     │
                    └─────────────────────┘
```

## 1. Discord — headquarters (living system)

**Official entry:** https://chat.lbry.org · https://discord.com/invite/lbry  

**What it is good at**

- Real-time development discussion  
- Support and debugging  
- Informal design talk, experiments, “is anyone looking at X?”  
- Onboarding people who already care enough to join a chat  
- Culture, humor, coordination  

**What it is bad at**

- Being citable six months later  
- Being indexed by search / AI as “official status”  
- Reaching people who will never join Discord  
- Speaking for the Foundation to the wider internet  

**Implication for growth**

Discord should remain the **best place to hang out and build**. Public channels should **invite people into Discord** when conversation is the goal — not try to replace Discord with Twitter threads.

**Implication for X**

Regular X posts can and should say, in plain language:

> Most day-to-day development and community discussion happens in the LBRY Foundation Discord: https://chat.lbry.org

That single sentence is high-value, non-hype, and true.

### Suggested Discord → public pipeline

Not everything in Discord should be publicized. Use a simple filter:

| Discord activity | Promote outside Discord? |
|------------------|---------------------------|
| “Working on FOO tonight” | No (unless FOO is public and unfinished talk is labeled carefully) |
| Decision that changes how people use the network | Yes → forum post, then X link |
| Release candidate / tag / binary | Yes → forum **Releases**, then X |
| Useful how-to that got repeated 5 times | Yes → forum **Guides** or docs, then occasional X |
| Support answer unique to one user | No |
| Governance / board-relevant outcome | Yes → site or forum, then X if appropriate |
| Community tool someone shipped | Yes → projects list / forum **Community Projects**, then optional X |

**Rule of thumb:** Discord is the **workshop**. Forum/site/GitHub are the **catalog**. X is the **window sign**.

## 2. Forum — durable discussion & releases

**URL:** https://forum.lbry.org  

The Foundation has already positioned the forum as a place for **announcements and software release notes first**. That is the right design for a nonprofit protocol org.

**Recommended roles**

| Category | Use |
|----------|-----|
| Announcements | Governance, policy, channel policy, major news |
| Releases | Anything shippable (even “source-only alpha”) |
| Developers | Protocol / API / integration discussion that should last |
| Support | Searchable troubleshooting |
| Community Guides | How-tos promoted out of Discord lore |
| Community Projects | Third-party tools and experiments |

**Why this matters for X**

If release notes live on the forum, `@LBRYFoundation` never has to invent substance. It only has to **link**.

## 3. lbry.org — home & stack map

**URL:** https://lbry.org  

Public home for:

- Mission  
- Downloads / component map (Nova, daemon, CLI, hub, blockchain)  
- Board  
- Projects catalog  
- Contact / social links  

**Growth idea:** treat the downloads page as a **product map**, not a marketing page. When people ask “what even is LBRY now?”, the answer should be a calm stack diagram on the site — which X can point at.

## 4. GitHub — proof of engineering life

**URL:** https://github.com/LBRYFoundation  

Repos and READMEs are the strongest “something is happening” signal for technical audiences. X does not need commit spam; it needs occasional pointers when:

- A meaningful tag lands  
- A README becomes the onboarding path  
- A component’s role is clarified (daemon vs hub vs client)

## 5. X / Twitter — public square (low volume, high trust)

**Account:** https://x.com/LBRYFoundation  

**What X is uniquely good at for LBRY**

- Reaching people who remember LBRY/Odysee but never joined Discord  
- Correcting “LBRY is dead” narratives in public  
- Feeding search engines and AI systems with **fresh official text**  
- One-click sharing of forum/docs links into crypto, FOSS, and creator circles  

**What X is bad at**

- Replacing Discord depth  
- Hosting long technical design  
- Surviving on pure vibes without eroding trust  

### Recommended posture

| Dimension | Recommendation |
|-----------|-----------------|
| Volume | Low: **≥1 post / month**, plus every real release/announcement |
| Tone | Librarian / steward |
| Primary job | **Point** and **teach** |
| Secondary job | Invite to Discord / forum when discussion is needed |
| Not a job | Daily engagement farming |

Pin one evergreen post (see [03-x-content-ideas.md](./03-x-content-ideas.md)).

## 6. Other surfaces (brief)

| Channel | Role |
|---------|------|
| **Telegram** | Lightweight chat mirror / invite; don’t make it source of truth |
| **Reddit** | Q&A and myth-busting; link forum/docs; avoid drama spirals as official voice |
| **Odysee / LBRY apps** | Content distribution layer — not the same as Foundation comms |
| **Personal board accounts** | Technical color commentary; link back to official artifacts |

## The visibility gap (problem statement)

```text
Without a pipeline:

  Discord: ████████████████ busy
  Forum:   ██ occasional
  Site:    ██ static-ish
  X:       ░ dead air

Public perception: “nothing is happening.”
```

```text
With a thin pipeline:

  Discord: ████████████████ still HQ
  Forum:   ████ durable notes
  Site:    ████ clear map
  X:       ██ honest pointers + education

Public perception: “quiet but alive; here’s where to go.”
```

## Discord-specific growth tactics (non-spammy)

1. **Welcome path** — one pinned Discord message: site, forum, downloads, “how to ask for help.”  
2. **Weekly optional “forum export”** — a volunteer (or officer) moves 1 repeated answer into a forum guide.  
3. **Release ritual** — no public “it’s out” on X until forum post exists (even a short one).  
4. **Public office hours** (optional) — rare, announced on forum + X, held in Discord voice/text.  
5. **Contributor spotlight (opt-in)** — forum post about a merged PR or community tool; X links the forum, not a hype thread.  
6. **Clear channel list legend** — so newcomers know where dev vs support vs random lives.

None of these require hype. All of them turn Discord energy into **shared memory**.

## Division of labor (suggested)

| Job | Primary surface |
|-----|-----------------|
| Design debate | Discord → summary on forum if it matters |
| Support | Discord + forum |
| Release notes | Forum → X pointer |
| Protocol education | Site/docs → X series |
| “Where do I start?” | Site + Discord invite |
| Governance transparency | Site/forum → rare X |
| Code | GitHub → forum/X when tagged |

## Takeaway

**Do not try to make X the center of LBRY.**  
**Do make X an honest window into the center** — which is Discord for living work, and forum/site/GitHub for durable truth.
