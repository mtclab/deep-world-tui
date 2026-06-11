# Scale: the world the map is a slice of

Binding design for population, settlement size, and map scale. Source of
truth: deep-world-history `population_scale_and_settlement_hierarchy.md`
(Rennik of the Arkit Archive, 155 AF) and `Geography.md`.

## 1. The canon facts the game must respect

- Sorethel carries **12–15 million souls** in the present age (recovering
  from a 18–22M imperial peak through a ~50% Fall die-off).
- Seven settlement tiers: Major City 50–100k+ (8–12 of them), City 15–50k
  (40–60), Town 3–15k (300–500), Village 500–3,000 (2,000+), Hamlet 50–500,
  Steading 5–50, Holy Place (seasonal).
- **Demography is hydraulic** (canon principles, verbatim spirit):
  - *river-corridor*: people settle where water runs; off-river densities
    are one-fifth to one-tenth of the riverine standard.
  - *low-point / altitude*: major cities at sea level on floodplains and
    harbors; villages climb the slopes; holy places sit highest.
  - *grain hinterland*: a settlement above ~3,000 cannot feed itself from
    walking-distance fields — it requires surplus moved by river, sea, or
    road. No hinterland, no town. Cut the road and the town starves.
  - *rain-shadow*: the dryness gradient is the population gradient.

## 2. The map is a slice, not the continent

One tile is roughly a house-plot. A region sector is therefore a *local*
landscape — a stretch of valley, forest, or coast — and the playable map
(a handful of sectors) is **one province** of Sorethel. The millions, the
Sampa Crossing of 80,000, the Basin Leagues — these exist in the same world
and reach the player as rumor, trade goods, caravans, chronicle entries,
and canon place-names. The player province plausibly holds: many steadings
and hamlets, villages along its waters, a town or few where rivers meet
roads, and — on rich water with a real hinterland — at most one city.

## 3. No authored sizes: carrying capacity

A settlement's ceiling is computed from its land, never written by hand:

    capacity ≈ base(terrain)
             × water_factor   (river/coast tiles within reach)
             × arable_factor  (farmland/grass hinterland within radius)
             × trade_factor   (roads + harbor: imported grain capacity)

Population grows toward capacity through the existing settlement-life sim
(harvests, stores, famine, migration) and *tier follows population*:
hamlet 50–500, village 500–3k, town 3k–15k, city 15k+. Steppe wells stay
hamlets forever; a delta crossroads can carry thousands; the gradient is
the canon gradient because it is computed from the same physics.

## 4. Roofs follow households

district roofs ≈ population / 7 (one roof = one household). Footprints are
sized from roofs, not tiers: a 200-soul hamlet is a handful of roofs; a
5,000-soul town is ~700 roofs and rightly dominates its valley; a Tier-II
city spans more than one sector. Sectors grow (and movement gains
sub-hour steps: several tiles per hour on open land) so walking across a
real town takes in-game hours — as it should.

## 5. Staging (versioned growth)

1. Canon population tiers + carrying-capacity model + grain-hinterland
   dependency for 3k+ settlements (starvation when the hinterland or its
   road fails).
2. Sectors and movement rescale; districts sized from roofs up through
   towns (~15k); the playable province gets at most one city candidate.
3. Tier-II city as multi-sector flagship; off-map canon cities wired into
   trade/rumor flows by name.
