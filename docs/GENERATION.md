# Generation — the possibility-chart engine

> The heart of the game: how the organic world + people + player are sampled.
> Deterministic from a seed. Data-driven (charts live in `data/`, generated/tuned
> from the lore bible). This doc defines the model the Rust code implements.

## 1. Determinism

- One **u64 seed** → the entire world. From it, derive sub-seeds per subsystem and
  per entity (e.g. `splitmix64(seed ^ hash("region:3"))`) so generating a region,
  a town, or an NPC is **independent + reproducible** and order-insensitive.
- RNG: `rand` + `rand_chacha::ChaCha8Rng` (seedable, portable, stable across
  platforms). **Never** use thread RNG or system entropy in generation.
- Rule: *same seed + same data + same code → byte-identical world.* A test must
  assert this (see `ARCHITECTURE.md` test strategy).

## 2. The chart model

A **Chart** is a weighted distribution over outcomes, optionally **conditioned** on
already-sampled dimensions. Sampling a person = sampling a sequence of charts in
dependency order, each conditioned on the prior picks.

```
Person sampling order (each conditioned on the ones before):
  people  →  region/settlement  →  social_class  →  profession
          →  craft_affinity  →  personality/values  →  age/sex
          →  household (spouse?/children?/dependents)  →  name
```

### Weighted table
```ron
// outcome -> relative weight (need not sum to 1)
WeightedTable(
  entries: { "farmer": 100, "labourer": 70, "fisher": 25, "smith": 8,
             "trader": 6, "soldier": 5, "scribe": 3, "priest": 2 },
)
```

### Conditional modifiers (the "not 99% soldiers" mechanism)
Base rates make most people farmers/labourers. **Modifiers** (multipliers) shift
the odds by the already-sampled dimensions — they bend, they don't replace:
```ron
ConditionalTable(
  base: "profession_base",                 // the WeightedTable above
  modifiers: [
    // when people == sepat, multiply these outcome weights:
    (when: People("sepat"),  mult: { "smith": 6.0, "miner": 4.0, "farmer": 0.4 }),
    (when: People("ahjo"),   mult: { "soldier": 3.0, "smith": 3.0, "miner": 3.0 }),
    (when: People("vayla"),  mult: { "trader": 8.0, "sailor": 5.0 }),
    (when: Region("coast"),  mult: { "fisher": 4.0, "sailor": 3.0, "farmer": 0.6 }),
    (when: Region("forest"), mult: { "forester": 4.0, "farmer": 0.7 }),
    (when: Class("low"),     mult: { "scribe": 0.1, "priest": 0.2 }),
  ],
)
```
Sampling: start from `base` weights, multiply by every matching modifier, then
weighted-pick. Result: a Sepät in the mountains is *likely* a smith/miner but can
still be a farmer; soldiers stay rare overall but cluster in Ahjo/frontier. Tune
the multipliers against the lore (see §5).

## 3. Dimensions (v1)

| Dimension | Source / notes |
|---|---|
| **people** | Arkit, Metsik, Väylä, Laakso, Sepät, Ahjo (+ non-humans later). Weights ≈ lore demographics. SAST names. |
| **region / settlement** | river-corridor density principle (lore): coast/river-valley dense, forest/steppe/upland sparse; settlement size tiers (hamlet→town→city) by hierarchy doc. |
| **social_class** | from `social_class_mobility_and_stratification.md`; gates some professions; per-people class shapes. |
| **profession** | `professions_and_trades_catalogue.md` — broad base (farmer/labourer/herder/fisher dominate) + specialists; conditioned on people+region+class. |
| **craft_affinity** | a god-craft lean (Word/Current/Still/Forge/Root) — rare; conditioned on people (Sepät/Ahjo→Forge, Metsik→Root, Väylä→Current, Arkit→Word, Laakso→Still). Most people have *none*. |
| **personality / values** | a small trait model (e.g. 4–6 axes or a tag set) + a values bias (per-people leanings); seeds dialogue + reactions. |
| **age / sex** | demographic pyramid (lots of young, fewer old). |
| **household** | spouse? children? dependents? debts? — these are the hooks the consequence system pulls on. |
| **name** | per-people **name grammar** (the lore Naming Atlas/Guide). Names MUST fit the sampled people. |

## 4. The player is a sampled person

Character creation = sample a person from the same charts, then let the player
**reroll** and/or **point-buy** adjustments (swap profession, allocate perk points,
choose/cut household ties). Starting perks are themselves drawn from a weighted
perk chart (conditioned on people/profession), so even "min-maxers" get an organic
character. The player begins embedded in a life (household, trade, town) — the
thing they can later keep, change, or abandon.

## 5. Data + lore grounding

Charts live in `data/` as RON (or JSON), loaded at runtime with serde, so they're
tunable without recompiling. **Generate/tune weights from the lore**, citing the
file. Key sources (in `deep-world-history/src/docs/`):
- `nations/population_scale_and_settlement_hierarchy.md` → people + region weights, density.
- `culture/professions_and_trades_catalogue.md`, `crafts_guilds_and_working_life.md`,
  `daily_life_by_nation.md` → profession base rates + per-people/region modifiers.
- `culture/social_class_mobility_and_stratification.md` → class.
- `peoples/` naming docs + `wiki/Naming-Atlas.md` → name grammars per people.

A starter chart ships in `data/charts.ron` (see issue #2) — **starter weights,
flagged for tuning against the lore**, not final.

## 6. Validation (tests)

- **Determinism:** same seed → identical generated world (assert deep equality).
- **Distribution sanity:** generate N=10k people; assert no single profession > X%
  (e.g. soldiers < 10%, farmers/labourers are the plurality), per-people modifiers
  visibly shift outcomes, names match people. Catch "99% soldiers" automatically.
- **Referential integrity:** every chart outcome/condition references a defined id.
