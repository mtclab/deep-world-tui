# Building & Settlement: the player's hand on the world

Design for the construction/settlement arc. Baseline as ever: everything has
a purpose, the world is living, decisions have consequences — and now the
land remembers what you raised on it.

## 1. What it takes to build (requirements layer)

Today: materials only, and `start_build` silently picks the first affordable
kind. Wrong on three counts: no player intent, no land, no labor.

**Materials** (exists, keep): Tarp=1 Cloth … Home=280 Wood+80 Nails+70
Stone+20 Thatch+3 Glass.

**Choice**: `build <kind>` — the player says what they're raising.

**Land** (new): each kind has terrain it can stand on. A tarp goes anywhere
you can lie down; a kota wants open ground; a cabin wants real land.

| Kind | Stands on | Notes |
|---|---|---|
| Tarp, LeanTo | any passable, not Settlement/Road | ephemeral camp |
| TarpTent, Laavu | Forest, Grass, Tundra, Swamp, Coast | camp country |
| Kota | Grass, Tundra, Farmland | open ground, hearth-pit |
| Cabin, Longhouse, Home | Grass, Farmland, Forest, Coast | homestead land; also **in a settlement** (see §5) |

**Tools & labor** (new): Cabin+ requires a Tool (and wears it). Big builds
advance only while the player **works the site** (a `work` action: 8h of
labor per visit) — no more walking away from a self-assembling Home. Tents
and laavu stay quick-pitch. Frost doubles groundwork hours for Cabin+.

## 2. What they do to the world (effects layer)

Today: rest tier + comfort bonus + maintain. Each kind gains a worldly role:

- **Tarp/LeanTo** — shelter, nothing more. The road forgets them (no decay
  tracking, as now).
- **TarpTent/Laavu** — a *camp*: −encounter risk resting there (exists via
  tier), and a named waypoint in the journal ("my camp under the ridge").
- **Kota** — a *hearth*: cooking. Meal-type crafts at your own kota yield +1
  (a real fire beats a traveler's pot). Trap lines run from it (trap yields
  +1 radius effectively).
- **Cabin** — a *homestead seed* (§4) and a **stash**: structures Cabin+
  hold a per-structure inventory. Stashes persist through death — the heir
  inherits the house *and what's in it* (legacy deepens).
- **Longhouse** — a *waystation*: NPC travelers shelter there. Each season
  it stands near a road: small reputation trickle with the nearest
  settlement + occasional rumor ("the long hall at X took in the storm-bound").
- **Home** — full residency anchor (§5) and the strongest stash/comfort.
- **Shrine (new player-buildable)** — a god-site: pick the god at raising;
  resting beside it gives that god's affinity a slow pull; pilgrim
  encounters spawn near it. Stone 6 + Cloth 3, like the NPC kind.

## 3. Player farming (prerequisite for the homestead arc)

The Farm/CropType machinery runs for NPC settlements only; the player can't
plant a seed. Add: `plant` on Farmland/Grass within 2 tiles of your Cabin+
(a farmstead farms *its* land, not the wilderness), crop chosen by terrain
suitability; farms tick with the existing growth/weather code; `harvest`
when ready → Food to inventory/stash. Frost kills standing crops (as for
NPCs). 1–3 plots by structure tier (Cabin 1, Longhouse 2, Home 3).

## 4. A village grows around the farmstead (the flagship)

If you feed a place, people come. Checked each season-turn:

**Conditions**: player has Cabin+ with a working farm; stash+farm food
surplus above a threshold; region fit (game_richness > 0.5, no existing
settlement within ~6 tiles, passable open land adjacent); player's regional
standing ≥ baseline.

**Then settlers arrive in waves** — rumor first ("families on the road,
asking after the homestead at X"), then +2–4 people camped by the farm
(journal). At ~12 souls the world recognizes it: a **new Settlement** spawns
at the site — hamlet, population seeded from the arrivals, the player's
structures absorbed as its first buildings, named from the region's naming
tradition (or carrying the player's name in the founding record). It then
lives by every existing settlement system: farms, stores, construction,
festivals, growth — it can become a village, a town; it can starve too.

**Founder status**: permanent high standing there, founder's-rest (own bed
forever), the founding noted in the chronicle-style journal, and the heir
keeps it all.

## 5. Building in an existing village (residency)

A Cabin/Home raised on a settlement tile (allowed when local bias isn't
hostile and standing ≥ baseline — the village must *let* you build):

- **Own bed**: Inn-tier rest, free, forever (exists mechanically via
  structure tiers — gate it to owner).
- **Stash** at home (§2).
- **Resident standing**: counts as the friend-discount tier; festival
  invitations are yours by right.
- The house counts toward the settlement's buildings; the village remembers
  who built it (founder-style journal line).
- Refusal is real: hostile peoples turn your timber away at the gate.

## 6. Re-population & land-taking (the world builds back)

The Fall's long tail runs in reverse when conditions allow — and the game
already makes ghost towns; now it can fill them:

- **Resettlement**: each season, a ghost town (pop 0) with a prosperous
  neighbor (stores/head high, pop above threshold, same region or adjacent)
  draws a resettlement party: rumor ("X sends families to reopen Y"), then
  the settlement re-seeds (pop 10–20, people sampled from the sponsor,
  services rebuilt from none). The sponsor's polity gets the credit in the
  telling.
- **Land-taking**: a region with no settlement, high richness, and a
  prosperous neighbor occasionally receives a founding party the same way —
  new hamlets born without the player's hand, because the world is alive
  with or without you.
- Both feed the rumor mill and the markets (new mouths, new stores).

## 7. The wider field (future tier, scoped small on purpose)

- **Roads**: player-laid road tiles (Stone+labor) cut travel hours on a
  route you actually use; NPCs prefer them; caravan chance rises along them.
- **Bridges**: cross a Water tile (heavy cost) — the canon resonance is
  obvious (Karsath held; Velkarmoss opened the Fall).
- **Wells**: water source on dry tiles (auto-drink range in desert/steppe).
- **Waymarkers**: cheap cairns that extend reveal radius along a path.
- **Palisade**: settlement-adjacent safety boost (Safety need + encounter
  table softening).
- **Signal fires**: one-shot world-visible event (rescue/rumor hook).

## Build order (each its own PR, gated as always)

1. **Foundations**: `build <kind>` choice, terrain fitness, tool gate,
   work-the-site labor for Cabin+ (kills ghost construction).
2. **Player farming**: plant/tend/harvest at the homestead.
3. **Stash + residency**: per-structure inventory; build-in-village with
   consent gates; own-bed.
4. **Homestead → hamlet**: settler waves, settlement spawn, founder status.
5. **Re-population + land-taking**: ghost towns refill, new NPC hamlets.
6. **Structure world-effects**: kota cooking, longhouse waystation, shrine.
7. **Future tier**: roads/bridges/wells/waymarkers as appetite allows.

## 8. Canon-logic audit (is each piece true to the world?)

Every mechanic above tested against the world's own physics — the
Conservation Principle (nothing from nothing), the Fall's slow recovery,
withdrawn-but-present gods (proximity never confirmed), god-peoples as
diaspora minorities, and the novels' tone (no chosen ones; competence and
maintenance, not miracles).

**Farming (#343)** — agriculture is the canon base economy (grain farmers
are the most common profession; the Harkomi plain feeds the continent), so
the mechanic is native. Two corrections it must carry: **seed costs** —
planting consumes a measure of Food (nothing from nothing; this also goes
for NPC farms, which currently plant from air); and the **forest-edge
tension** — clearing wooded land for fields is the Ahjorath sin in
miniature (the Kaelva Burning's whole substance). Farming adjacent to
Forest tiles should cost Keuru affinity and Metsik-family bias; swidden is
deeply Finno-Ugric and deeply contested. The crop list should eventually
align with canon's crop appendix (geography/crop_species...).

**Stash & residency (#344)** — mundane storage and a council's consent:
both native. The consent gate (bias + standing) IS the inter-people world
working as written. Inheritance of house and stash matches the
family-standing canon. Future-true addition: polities tax; a resident
might owe a yearly coin (skip for now, note for the polity layer).

**Homestead → hamlet (#345)** — this is the era's signature story (the
Basin Leagues formed exactly this way, ~50 AF). Constraints to honor:
settlers must come FROM somewhere — sampled from neighboring settlements
and SUBTRACTED from them (people are conserved); the hamlet inherits their
peoples mix, so god-peoples stay minorities even in the player's town; the
name comes from the region's naming tradition, never the player's (the
founding is remembered in the record instead — fame without naming rights,
very Archive). Founder status is local standing, not cosmic election.

**Re-population & land-taking (#346)** — directly canonical (the chronicle
is full of reopenings), with two disciplines: the sponsor PAYS (population
and stores actually move — conservation again), and the pace stays slow
and rare; the Fall's tail is long, and a world that bounces back in a
season betrays the whole setting. Some ghost towns should simply stay
empty for economic reasons — not every wound closes.

**Structure effects (#347)** — kota cooking and longhouse waystations are
mundane and waystation-canon (the dying waystation network is a core
motif; the player quietly restarting one node of it is the recovery era in
a single mechanic). The player SHRINE needs the careful frame: gods are
withdrawn; a new shrine is the player's devotional practice, not a summons
— its affinity pull is slow and small, and the pilgrims it draws are
ordinary people on ordinary roads. No miracle sites.

**Infrastructure (#348)** — the most canon-resonant tier of all, with two
corrections: scale honesty (one person lays a TRAIL and a FOOTBRIDGE over
a stream — Karsath was god-era engineering; the names must stay humble)
and, non-negotiable: **player infrastructure decays and demands
maintenance from day one**. A footbridge left untended becomes a private
Velkarmoss. That is the Fall's lesson made playable, and it is the reason
this tier exists.

**Fixes applied to shipped content during this audit:** the GodCampsite
collapse text showed all Five Gods in person, speaking — flatly against
the proximity-event canon (never confirmed in 11,000 years; the reveal
belongs to Book 3). Rewritten deniable: a kept fire, wrapped food, no
tracks, and a survivor who isn't sure they believe themselves. The rescue
screen's "X watches over you" likewise softened to a felt thing. Noted
for later passes: temple penance might ask a donation (restitution is more
Masa than absolution); theft-witness odds could scale with settlement size
(more eyes in a city); CANON_RUMORS are timeless ambient truths, which
suits an era that changes slowly.
