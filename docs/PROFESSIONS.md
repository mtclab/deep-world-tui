# Professions & population — how many, and can every NPC be unique?

Design note. Two questions: (1) can all NPCs be unique, and (2) how many
professions does a *living, organic, realistic* (fantasy) region actually need?

`Person.profession` is a free-form `String` — there is no enum to extend, so
adding trades is data, not a code change. The sim already uses ~33 of them.

---

## 1. Can every NPC be unique? Yes — within three honest limits.

An NPC is unique along four axes; each is achievable:

- **Identity** (already true): every materialised `Person` has an id, a name from
  the people's name-banks, traits, relationships, a lineage. They are individuals,
  not clones.
- **Appearance**: with the modular compositor (see `deep-world-godot/docs/SPRITES.md`)
  the part-space is already in the **hundreds of thousands** of human combinations
  (and grows with every part added — add a *face* layer of eyes/nose/mouth/marks
  and it is millions). Derive the look **bijectively from a unique key** (person id
  / name-hash decoded as mixed-radix part indices) and distinct NPCs get distinct
  sprites *by construction*, up to the part-space. Size the part-space above the
  province population and "all visually unique" is guaranteed, not hoped-for.
- **Name**: name-banks must out-size the population, or compose names
  (given + patronymic/lineage + by-name) — the Khör already recite lineages; given
  × patronymic × epithet is effectively unbounded.
- **Simulation depth**: this is the real limit — and the **entity-first epic**
  (deep-world-godot#50) settled it. The earlier plan here was *aggregate* level-of-
  detail: `Settlement { people: Vec<Person>, population: u32 }`, a materialised
  sample of full individuals plus a bulk count, with the background "promoted into
  a unique individual the moment you interact." **That is no longer the model.**
  Every inhabitant is now a **real, persistent `Person` from worldgen on**;
  `population` is a *derived cache* of `people.len()`, never a stand-in for people.
  What scales is **temporal** level-of-detail, not existence: the player's region
  ticks live each hour, distant regions advance the same real agents on a coarse
  daily batch (two-rate LOD). The province (8.5k–121k souls) is materialised and
  ticked within budget after the slice-0/1 perf work and the O(n) fixes. NPCs
  pursue needs as agents (the hunger ladder: eat → buy → work → steal → banditry),
  and emergence — trade, famine, desperation, banditry — falls out of individuals
  under scarcity, not aggregate passes. **There is no LLM in the game**: dialogue
  and flavour are deterministic voice templates (`src/voice/`) only.

**Verdict:** yes — unique in identity, name, and appearance, and now *real and
needs-driven* for every soul, not only those the player meets. Simulation depth is
managed by **temporal** LOD over real agents, never by replacing anyone with a
statistic.

---

## 2. Professions: 20 is far too few. The principle: populate the chains.

A region "runs" only if **every good has producers and every producer's inputs
have producers**, *and* the social roles that hold a settlement together exist.
Professions should be **demand-derived** — each settlement spawns the trades its
goods-chains, size, region, and people require — not drawn from a flat global list.
That keeps it organic: a fishing coast grows divers, net-makers and salt-boilers; a
forge-valley grows charcoal-burners, ore-readers and smiths; a grain plain grows
plowmen, millers and bakers.

### The 25 goods imply their chains (each arrow is a trade)

```
Wood: woodcutter → sawyer(planks) → carpenter/joiner; → charcoal-burner(Charcoal)
Charcoal + Iron-ore: miner → charcoal-burner → smith(Iron, Nails, Tool) → farrier/cutler
Stone: quarryman → mason; lime-burner(mortar); → builder/roofer/thatcher
Clay: clay-digger → potter(Pottery), brickmaker; glassblower(Glass)
Grain: plowman/farmer → miller(flour) → baker(Food/bread) + brewer(Ale)
Beasts: herder/shepherd/swineherd/cowherd → butcher(meat,Hide) + dairymaid(cheese)
Hide: hunter/trapper + herder → tanner(Leather) → cobbler/saddler/coat-maker(Coat)
Flax/wool: grower → spinner → weaver(Cloth) → dyer → fuller → tailor/furrier
Fibre: grower → rope-maker(Cordage) → net-maker/sail-maker
Herb: forager/herbalist → healer + salve-maker(Salve) + apothecary(Bandage)
Branches/Tinder/Thatch: gatherer/thatcher; chandler(candles)
Trap: trap-maker; Tool: smith + bowyer/fletcher
Water: well-digger/well-keeper; Salt: salt-boiler → preserves fish/meat
Coin: trader/merchant/moneychanger/factor; carried by caravan-master/carter/ferryman
```

### Full taxonomy (target library ~100–140; ✅ in sim today)

**Food & forage** — farmer ✅, plowman, herder ✅, shepherd ✅, swineherd, cowherd/
drover ✅, dairymaid, beekeeper, fisher ✅, diver (Mëräk), fowler, hunter ✅, trapper,
forager ✅, reindeer-herder (Khör/Porokansa), salt-boiler (✅ salt-*), miller, baker ✅,
butcher ✅, brewer ✅, mead/vintner, cheesemaker, cook/innkeeper ✅.

**Extraction & materials** — woodcutter/logger, sawyer, charcoal-burner, miner ✅,
quarryman, peat-cutter, lime-burner, clay-digger, resin/tinder-gatherer.

**Crafts** — smith ✅, farrier, cutler/bladesmith, nailer, tinker, mason ✅, carpenter ✅,
joiner, cooper, wheelwright, cartwright, boatwright/shipwright (coast/Mëräk),
thatcher, roofer, dauber, glassblower, potter ✅, brickmaker, spinner, weaver ✅,
dyer ✅, fuller, tailor, furrier, tanner ✅, cobbler ✅, saddler, rope-maker ✅,
net-maker, sail-maker, basketmaker, chandler, bowyer/fletcher, trap-maker,
salve-maker, bandage/linen-maker.

**Trade, movement, hospitality** — trader/merchant ✅, peddler, caravan-master,
drover/teamster/carter, ferryman, porter, guide, innkeeper ✅, stabler, moneychanger
(Väylä), toll/bridge-keeper (canon Kael), harbor-master.

**Health, death, faith, knowledge** — healer ✅, herbalist ✅, midwife ✅, bonesetter,
apothecary, grave-tender (Metsik), cairn/word-keeper (Khör), priest ✅/shrine-keeper
(per god), pilgrim-warden, scribe ✅ (Arkit), story-holder (Metsik), lawspeaker,
rune-carver, teacher, archivist/loremaster, memory-keeper (Tzäkhar).

**Governance, order, defense** — headman/reeve/chief, elder ✅, council-member,
warden ✅, watch/guard ✅, soldier ✅, border-ranger, bailiff/tax-collector, crier,
beacon-keeper, well-keeper, gatekeeper.

**The margins (no fixed trade — vital for realism)** — labourer/day-worker,
servant, apprentice, child, retired elder, beggar, hermit, exile, pilgrim,
outlaw/bandit (the bands), vagrant.

### The fantasy layer — no wizards

Magic is **withdrawn-gods + the Conservation cost**: there is no mage class.
Instead the "magical" roles are **vow-keepers and shrine-tenders** — a smith under
Oltzed's vow gets craft-enhancement at a bodily cost; a Laakso still-keeper, a
god-vowed warden. Plus people-specific trades that *are* the fantasy: Mëräk
wave-merchant/tide-reader/diver, Khör warmth-keeper/word-keeper, Häl canopy-tender/
physic-keeper, Tzäkhar deep-miner/stone-memory-keeper, She'ar steppe-walker/
water-finder/message-runner, Sepät forge-master/ore-reader/water-rights adjudicator,
Väylä factor/arbiter/tally-keeper, Metsik story-holder/hedge-warden. These give each
people a distinct economic *and* mythic signature.

---

## 3. How the sim should spawn them (recommendation)

Keep professions demand-driven, three tiers:

1. **Core (~25–30):** every settlement needs food, water, shelter, defense, a
   hearth, a faith-site, basic craft. Always present, scaled by size.
2. **Chain (~40–60):** unlocked by what the settlement/region produces — a tannery
   town has tanner→cobbler→saddler; only places with a forge get charcoal-burners
   and farriers. Driven by the existing goods/recipe graph (the living economy).
3. **People & region flavour (~30–40):** the canon-people trades above + the margins.

A settlement of N people draws a profession histogram from these tiers (most are
farmers/labourers/herders; specialists appear past size thresholds — a miller needs
a grain surplus, a glassblower a town). This is the organic version: the trade list
*emerges* from the economy and the place, and the world reads as alive because the
chains are actually staffed.

**Bottom line:** ~25–30 core + a library of ~100–140 total, spawned by demand. 20 is
enough for a hamlet; a living *region* needs the whole supply web populated, plus the
people-specific and marginal roles that make it feel inhabited rather than staffed.
