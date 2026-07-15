# Map Design Lessons — evolving our worldgen against the "open worlds lost their magic" thesis

> Source: an essay-video on why modern open worlds feel dead (Daggerfall → Morrowind →
> Ubisoft towers → Starfield → BotW/Elden Ring/KCD2). Ideas taken as *provocations to test
> against our architecture*, not law. Companion to [WORLD_DEPTH.md](../WORLD_DEPTH.md),
> [GENERATION.md](GENERATION.md), and the Godot [terrain-diffusion spike](../../deep-world-godot/docs/SPIKE_TERRAIN_DIFFUSION.md).

## 1. The thesis, compressed

The video's whole argument reduces to three rules the great worlds followed by instinct,
and one business trap that made the industry abandon them:

1. **Density beats size.** Small map where every corner earns its place > country-sized
   wallpaper. Daggerfall (procedural, country-sized, 15,251 locations, forgettable) vs
   Morrowind (hand-placed, 0.01% the size, legendary).
2. **Every place needs an *offer*.** A human decides what goes where and *why*. The offer
   is the falling wizard, the dead tax-collector in the swamp, the Easter egg behind the
   unmarked window — a joke/story written in advance for one player to stumble on. "An
   algorithm will never do that… because it can't imagine you standing there."
3. **The world has to trust you.** No breadcrumb trail, no icon wall revealed before
   discovery. Trust is what makes finding the treasure *mean* something.

**The trap:** the Assassin's Creed viewpoint → map-reveal turned "*I wonder what's out
there*" (curiosity) into "*what's left on the list?*" (inventory management). Repeatable,
budgetable, spreadsheet-friendly — so it ate the recipe. Then the engagement-container
business model (time-in-game as revenue) made padding *rational*, and procedural
generation became the final cost-squeeze (Starfield's 1,000 empty planets).

**The counter-proof:** BotW, Elden Ring, KCD2 — ~70M copies in one decade — all won by
returning to density + authorship + trust. The recipe never stopped working; publishers
stopped believing anyone would pay for it.

## 2. Where Deep World already lands on the right side

We are not the target of this critique by default — but only because of specific choices
that are worth naming so we don't regress:

- **We are not an engagement container.** TUI shipped 1.0 as a premium, finite artefact;
  no PRI/retention metric, no store, no "time-savers." The entire economic engine that
  drove the rot is absent. That is a structural asset — protect it.
- **Our procedural gen is lore-conditioned, not noise.** Charts (`GENERATION.md`) bend
  outcomes by people/region/class grounded in the bible. A Sepät in the mountains is
  *likely* a smith — statistical organicness, not uniform mush. This is a real defense
  against "99% soldiers" flatness.
- **Reveal is already Nintendo-correct, not Ubisoft.** `DiscoveryEffect::Reveal`
  (AncientRuin, climbed "for the lie of the land") reveals *terrain*, not icons —
  the exact BotW Sheikah-tower reframe the video praises. We never rain a to-do list.
- **Consequence + memory exist.** NPC memory, collapse outcomes, heirs — the "world
  reacts / remembers" axis the video credits Gothic and RDR2 for.
- **WORLD_DEPTH already adopts "hourly content needs dozens of variants."** The
  exposure-rate rule is our version of "density beats size."

## 3. Where we are still exposed — the real gaps

The video's sharpest blade cuts at **statistical variety ≠ memorable place**. Our worldgen
produces *no two towns with identical stats*, but structurally it may still be wallpaper:
same terrain generator, same encounter tables, same 26 discovery *types* scattered by the
same rules. Daggerfall also had statistical variety. Morrowind had **specific, authored,
one-of-one** places. Three concrete gaps:

### Gap A — no landmark gravity (the "triangle rule")

BotW's world is built of triangles: **big** landmarks (volcanoes/mountains) visible
map-wide that *pull* you; **medium** hills that *hide a question* ("what's behind the
ridge?"); **small** rocks giving ground rhythm. Every triangle does two jobs — gives a goal
*and* hides a surprise. Distances were calibrated against **real Kyoto city blocks**, not
square kilometers.

Our gen is a **homogeneous field** — regions are a grid, terrain is per-region fill, there
is no macro skeleton of gravitational anchors visible across the province. Nothing pulls
the eye or poses a geometric question. This is the single biggest miss.

> **Evolution:** add an explicit **anchor layer** on top of the region field. A small,
> hand-tiered set of landmarks per province (great = province-visible, mid = sector-visible
> "what's over that ridge", minor = local flavor), each placed for *gravity*, each carrying
> an **offer** (below). Terrain gen then flows *around* anchors instead of anchors being
> sprinkled into flat terrain. Calibrate spacing to **on-foot walk-time between offers**
> (our Kyoto-blocks = playtested minutes-per-discovery), not tile counts.

### Gap B — discoveries are generic *types*, not authored *offers*

We have 26 `DiscoveryKind`s. They are excellent connective tissue but they are *categories*
("a standing stone", "a bone circle") sampled statistically — the algorithmic kind the
video says can never write the falling wizard. We have **4.63M words of authored lore** (439
docs, a named-individuals catalogue, nation histories, treaties) and **almost none of it is
pinned to coordinates**. The bible is the largest untapped *offer reservoir* imaginable.

> **Evolution — two-tier generation** (this is Morrowind-over-Daggerfall, automated):
> 1. **Authored anchor pass:** mine the lore for named places, historical events, and named
>    NPCs, and *place the specific ones* — the battlefield where treaty X was signed with
>    the actual named dead; the healer from the named-individuals catalogue actually living
>    in her town with her actual daughter as heir. Deterministic, seed-stable, but *these
>    exact cells are one-of-one*.
> 2. **Statistical fill pass:** current charts fill everything *between* the anchors.
>
> Add an **"offer bank"** distinct from the encounter *tables*: a hand-written set of
> witnessed-by-nobody set-pieces (the falling wizard = a scripted micro-event seeded ~N per
> province, never repeated), separate from the base-rate encounter distribution. Charts give
> the texture; the offer bank gives the *memorable spikes* the video demands.

This is the direct application of **vainola's model to Deep World.** vainola already sidesteps
wallpaper by anchoring spawns to **41k real muinaisjäännös (archaeological) coordinates** —
every anchor is a real, unique, meaning-loaded place, i.e. authorship for free. Deep World's
lore corpus is our muinaisjäännös database; we just have not wired it to the map.

### Gap C — the quest board is a to-do list

WORLD_DEPTH notes the quest board is "always visible" and goes "samey within one life." A
persistent board of listed errands is exactly the "what's left on the list" failure mode,
and `VisitDiscovery` quests risk turning our (currently trust-correct) discoveries into
checklist entries.

> **Evolution:** push toward **diegetic, Morrowind-style direction-giving** — an NPC
> tells you "follow the road west, cross the old bridge" (words, not a marker) — over a
> standing board. Keep quests *pulled* from rumors/NPCs in the world, not *pushed* onto a
> list you open on arrival. We already have a rumor channel; feed leads through it.

## 4. Re-reading the terrain-diffusion spike in this light

The spike evaluates a learned, real-world-fidelity terrain model (trained on ETOPO +
WorldClim). The video's Starfield autopsy is a **direct warning here: realism ≠ meaning.**
More-real terrain that is uniformly real is still wallpaper — Daggerfall/Starfield's exact
mistake in a prettier coat. Adopt diffusion *only* where it serves the three rules:

- **Do NOT** use it to make every square kilometer plausibly realistic and homogeneous.
  That is chasing polygons/planets — the second-order version of the same disease.
- **DO** consider it for **macro legibility** — big, distinct, far-visible landform
  *silhouettes* that give the triangle rule its big triangles — *if* it reads better than a
  hand-drawn coarse map. The spike already recommends "supply our own coarse map (synthetic
  Perlin or hand-drawn / Azgaar)"; the video pushes us to **author that coarse skeleton by
  hand** (landmark gravity is a design act, not a sampling act) and let diffusion/Perlin
  only fill interstitial detail.
- Keep Perlin for point queries (spike's own conclusion). The **anchor layer (Gap A) sits
  above whichever terrain generator wins** — it is the design decision that matters more
  than the noise source.

Net: the spike's *technical* two-track (bake-first, live-later) is fine. But the **priority
ordering changes** — the terrain generator is downstream of the anchor/offer layer, not the
headline feature. Better terrain on a homogeneous field buys nothing the video values.

## 5. Proposed order of work (cheap → expensive, alive-per-effort)

1. **Diegetic leads over the board (Gap C)** — route quests through NPC directions/rumors;
   stop pre-listing. Cheapest, protects the trust we already have.
2. **Offer bank (Gap B, part 2)** — a data-driven bank of ~dozens of one-shot witnessed-by-
   nobody set-pieces, seeded sparsely, never repeated. Pure content + a placement rule.
3. **Anchor layer / triangle rule (Gap A)** — worldgen change: tiered gravitational
   landmarks placed first, terrain flows around them, spacing calibrated to walk-time.
4. **Lore-pinned authored anchors (Gap B, part 1)** — mine the bible → a gazetteer of
   named places/people/events → place the specific ones as one-of-one cells. Biggest payoff,
   biggest effort; depends on #3's anchor slots existing.
5. **Terrain generator choice (spike)** — only after the anchor layer exists, and judged by
   whether it strengthens big-triangle legibility, not by realism.

## 6. The one-line test to keep

For any worldgen change, ask the video's question: does this make the player think
**"I wonder what's out there"** (curiosity) or **"what's left on the list"** (workload)?
Ship the first. And remember the dashboard trap — hours/tiles/counts all go *up* when you
make the world worse; the only thing that drops is the thing with no column: *what it feels
like to be there.*
