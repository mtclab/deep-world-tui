# Changelog

## Unreleased

**Sim tick, lighter (perf)** — the daily settlement tick was **cloning the whole sector** (every region, every day) just so it could read the terrain for its capacity check while it updated the settlements. It now reads the live tiles through a disjoint borrow instead — the loop only touches a different field — so the copy is gone. No behaviour change; the world ticks the same, with one fewer 16,000-tile allocation per region per day.

**The stick walks (#484)** — the left stick now steers the four-way walk, not just the d-pad: push it and you step that way, hold it and you keep stepping at a walking pace (a step every few frames, not a sprint of one per frame), centre it and you stop. The direction logic (`stick_direction`, dead-zone and dominant-axis) is pure and CI-tested; the gilrs backend tracks the stick and paces the steps. Feature-gated like the rest of the controller code.

**Controller, live — plug one in and play (#484)** — the `gilrs` backend lands behind a `gamepad` cargo feature: build with `--features gamepad`, plug in a controller (the new Steam Controller via Steam Input, or any pad), and it just plays — the backend reads the device, the tested mapping turns each press into the keystroke it means, and the same handlers the keyboard uses do the rest. Feature-gated so the default build and CI never pull `gilrs` (which needs the system `libudev` to compile); validated to build and test clean with the feature on, and to change nothing with it off. The d-pad and face/bumper/trigger buttons are read now; the Steam Controller's grips and trackpads arrive through whatever Steam Input binds them to.

**Controller dispatch — a button drives the whole game (#484)** — the second half of the controller foundation: `App::handle_gamepad_button` turns a controller button into the keystroke it maps to and runs the *same* event path the keyboard uses, so a gamepad reaches every screen — the map, menus, market, talk — with no per-screen controller code. Proven end to end **without any device**: a test feeds the d-pad and the player walks, feeds a face button and the game acts, feeds an unbound trackpad and nothing happens. Layers 2 and 3 of the plan are now done and tested; all that's left for live play is a backend to *produce* the buttons (gilrs needs the system `libudev` to build — noted in `docs/STEAM_CONTROLLER.md`).

**Worldgen, lighter on its feet (perf)** — `carrying_capacity` and `trade_factor` now read the terrain by borrowing the tile slice instead of taking a whole `TerrainMap`, so worldgen no longer **clones the entire sector twice per settlement** just to probe the land — it reads the live tiles in place. No behaviour change (same numbers, same world); it just stops paying for a copy of a 16,000-tile map on every capacity check, which the taller sectors had made dearer.

**Taller sectors, room to sprawl (#480/#378)** — region sectors grow from 160×80 to **160×100**: with #480 having freed a great town's horizontal berth, this gives the rare Tier-II city the vertical room to actually sprawl past the towns beside its river, instead of being clipped by the sector's edge. Old saves are untouched — the upscale chain is width-driven and still lands them at their own 160×80, and a region reads its own dimensions, so old and new worlds both just work.

**The worked country is lived-in (#458)** — the open land between the towns is no longer empty: a scatter of **rural homesteads** now stands on the arable country — a dwelling, an outbuilding, and a worked field apiece — well clear of the towns, the roads, and the water. They use the real building layout (walls, a door you can walk through, a field beside), so the countryside reads as settled land, not blank ground. Knock at a holding's door and its folk are out in the fields; the holdings are map texture and life, not towns. (Wires in the homestead layout that was built but never placed.)

**Controller support — the foundation (#484)** — groundwork for playing with a gamepad, targeting Valve's new (2025) Steam Controller first. Because every screen already runs on `KeyCode` events, controller support is a thin layer that turns a button into the same keystroke the game already understands — so the whole game becomes controller-playable without per-screen changes. This lands the pure, fully-tested heart of it: `input::gamepad` maps a device-agnostic `GamepadButton` → game intent → key (the d-pad walks, the face buttons gather/rest/confirm/cancel, the Steam Controller's back grips carry the journey and the wait, the trackpads left free for the player's own binds). The live `gilrs` backend and the Steam Input ship config are planned in `docs/STEAM_CONTROLLER.md` and gated behind a feature flag, so this changes nothing in the default build.

**Balance, made organic (docs)** — the balance philosophy is reconciled to how the world actually plays: there is no death-rate quota and no targeted survival window the systems steer toward. A life's end emerges from how it was lived and the luck it was dealt — a careful life organically reaches old age, an exposed or unlucky one does not. BALANCE.md now records the death-soak census (cautious and bold cohorts land close — ~85% old age, mean ~day 300 — because the town's dangers and the wild's organically balance) as an *observed* reference, not a goal, and the stale "40-200 day average survival" target is corrected to describe the early pressure window it really is.

**Sit a while in prayer (#457)** — the gods have been withdrawn since the Fall, so faith is devotion, not a summons. **`p`** (or `pray` in the REPL) gives a quiet hour to the god you most keep — or, if you keep none yet, your people's patron (and the godless pray to Kukri, lonely god of the long road). It deepens that bond, but the practice plateaus: the more you already keep a god, the less an hour adds, and no amount of prayer ever makes a god of you. No miracle answers in the world — only the one who prays is steadied, and the prayer comes in the same deniable, half-remembered voice as the patron's dreams.

**Journey to the great cities (#456)** — the named cities of the continent (Sampa Crossing, Vessenath, Halkess, Velkarath, Keuramark) have always reached you only in rumour and caravan manifest; now you can **go to one**. From a town on the roads, provision the way with Food and set out (**`J`**, or `journey` in the REPL): three days there and back, and you come home with that city's own goods — Basin grain, Vessenath steel and furs, Velkarath salvage, Keuramark timber — and word of the wider world. The playable map stays a province slice; the journey is the road out of it.

**A great town sprawls (#454/#378)** — the rare Tier-II city (the land could carry fifteen thousand and more) is no longer held to its even share of the sector: it gets a **wider berth**, sprawling past the spacing that keeps ordinary towns in line, while the map bounds and the placement cursor still keep districts from overlapping. (Full vertical sprawl still wants a taller sector — a save-touching rescale — for another day.)

**Each people builds in its own hand (#454)** — an enclave no longer looks like a human town with different names: the Five raise their settlements in their own character. The **Tzäkhar** raise grand, deep-carved halls; the **Häl** keep to modest canopy-floor huts; the **Khör** to long herder-houses. The building styles lean to the dominant people, while the hash still varies them within that lean so no two settlements are twins. Human towns are unchanged.

**The trade floor shows its measures (#454)** — the market screen at an enclave no longer shows misleading coin prices: it names itself a trade floor of the Five and lists, for each good you can lay down, exactly what they give for it in kind ("1 Tool (have 2) → 2 Hide, 2 Food"), with the goods they want none of greyed out. What you see is the rate; the rate is the rate.

**The Five take no coin — barter in kind (#454)** — an enclave's market is a **trade floor, not a stall**: the Five take no coin and do not haggle. Lay down a good and they trade it for theirs at a fixed rate — a tool to the Khör for härkä-leather and steppe-butter, cloth to the Mëräk for deep-fish, food to the Tzäkhar for worked iron, and so on, the same coin-free measures as their roadside traders. Trying to buy with coin is refused; offer a good instead.

**The Five keep their own craft (#454)** — an enclave is no longer just a renamed town: each of the Five keeps its signature work on its own ground — the **Tzäkhar** deep-forge, the trading floors of the **Mëräk** and **She'ar**, the **Häl** physic-shrine of Keuru's wood, the **Khör** word-keepers' shrine. So an enclave you walk into has the building and the service that people are known for.

**The Five keep their own ground (#454)** — enclaves no longer depend on the dice alone: on a people's home terrain, now and then a settlement is **founded as theirs** — the Mëräk to the tideline, the Khör to the steppe, the Häl to the deep wood. Their people, their names, their trades, on the ground that is truly theirs. (Seeded with the settlement's own deterministic rng, so no other settlement's people or place is disturbed.)

**Enclaves of the Five, named for what they are (#454)** — a settlement whose people are one of the canon Five non-human peoples — the Tzäkhar deep-smiths, the Mëräk tideline fishers, the She'ar dry-country walkers, the Häl of the canopy, the Khör of the steppe — is now recognised as an **enclave**, not just another town: it names itself for what it is ("Vaskiluuri, a Tzäkhar enclave") in the menus and the world. Derived from the dominant people, nothing new persisted. Groundwork for the Five keeping their own ground and bartering in kind (the in-kind trade floor and home-terrain founding come next).

**The household gathers at the fire (#458)** — at night, people no longer scatter to the far corners of their buildings: they gather **by the hearth**, filling the warmest tiles first. Walk into a lit house after dark and you'll find them around the fire, not standing in a cold back room.

**A key for the door-signs (#458)** — the map legend now spells out the new service signs (`+` door, `T`avern, `C`hapel…), so a newcomer reading the map knows the bright letters over the doors are services to walk to.

**Signs over the doors (#458)** — you can tell the tavern from the temple from a plain home without knocking on every door: each service building now wears a **sign over its doorway** on the map — `T` tavern, `C` temple, `F` forge, `W` trap-workshop, `A` archive, `$` trade post, `S` shrine, `H` hearth-house — in a bright hand, single-width so the tile grid stays true. Walk to the sign you want.

**A fire at the heart of every home (#458)** — buildings now have a **hearth** at their centre (new `Terrain::Hearth`, a `*` glowing warm on the map). Walk in and stand by it: resting on the hearth is the warmest rest in the world — a roof, walls, and a fire — an inn-grade night even in the deep cold that leaves the open country shivering. The hearth is where the household gathers; in time it is what the home is built around.

**Everyone goes home at night (#458)** — the streets no longer simply empty after dark: the settlement's people go **indoors**, onto the floors of their own buildings, by the hearth. Now that you can walk in through a door, a lit window at night is someone you can still meet — step inside and greet them. Deterministic as ever (the same soul keeps the same room all night), and the renderer and the bump-to-talk layer read the same positions, so what you see indoors is exactly who you can meet.

**Walk into the building (#458)** — the door is a way *in*, not a doorstep you bounce off. Step through it and you stand **inside, on the floor**, free to walk the rooms of the real walled structure — the interior is part of the one world map, not a separate screen. Crossing the threshold from the street still does what it always did — the tavern serves, the temple blesses, a plain home answers the knock — but **once, as you enter**, not again as you move about inside or step back out into the street. (First step toward lived-in interiors: hearths, occupants and furnishings come next.)

**Real towns — the load-bearing integration (#458)** — settlements are no longer a smear of ground dotted with single-tile houses on the even/even grid: every town, village and hamlet is now laid as **real walled buildings on walkable streets**, in the world, at human scale (the Stoneshard mould — one continuous map, no overworld, no minimap entry). Worldgen, promotion, foundings and old-save fixup all paint the same district through `gen::town` → `gen::building::district_buildings` — the single deterministic source of truth, recomputed from the anchor so walls, service-doors and the streets between always agree without anything new persisted. A building is a `Wall` border around a walkable `Floor` interior with one `Door`; you **walk in through the door** (no menu) — the tavern serves, the temple blesses, a plain home answers the knock — and the townsfolk stand in the streets between the buildings. Plot pitch (and so building density and variety, hut→manor) scales with the district; the footprint sizing now leaves each building room and a street (≈4 tiles of edge per plot-row). `Wall`/`Floor`/`Door` go from defined-but-unplaced to load-bearing. The old even/even house grid (`house_cells` and the grid painter) is gone. *Next: the great-town sprawl wants wider settlement spacing (sector rescale) to grow past the packed-valley clamp.*

**Rumours of the uncanny (#455)** — the myth-creatures are now *heard of* before they are ever met: tavern talk sometimes carries a deniable word of them — a hunter who chased an elk three days, a fire kept at a cave-mouth and an old woman wanting paying, the scree that moved on its own, a back that broke the far water, a grey shape upwind in the deep wood, a pale figure in the shallows, a dark bird on the still water the morning a man died, a stag that stood wrong at the treeline. Each ends in its sober explanation. New UNCANNY_RUMORS bank, surfaced through the tavern rumour channel.

**Mythic Phase 2 — the mountain stirs (#455)** — a new mountain rare encounter: a vast, slow presence on the high slope, far too large for any rockfall, that goes still the moment you stop to look. **Wait it out** (Calm) and you pass safe, telling yourself it was only the mountain settling; **cross beneath it** (PushThrough) and the loose scree is treacherous — the day's strength spent, and on a poor turn a turned ankle (a Sprain) on the shifting slope. Deniable: the mountain does not say. **Mythic Phase 2 complete** — näšvyly fever · spectral-elk chase · threshold toll · the mountain stirs.

**Mythic Phase 2 — the threshold toll (#455)** — a new mountain rare encounter: an old woman keeping a fire at a cave-mouth in the high rock, where no one keeps a fire, who asks a price for the road past — "bread, or herb, or coin, your choosing." **Pay** (Trade) and the road past is easy, the dark kindly, and the threshold-keeper (Kukri) marks it; **refuse** (Flee) and you take the long way round the mountain — hours and strength lost to the detour and the cold, but no harm. Come empty-handed and she waves you on, uneasily. Deniable as a strange hermit. An *offering* encounter, the richer figure-type of Phase 2.

**Mythic Phase 2 — the spectral elk (#455)** — a new forest rare encounter: a great elk at the tree-line that is a *sending*, not prey — antlers too wide, eyes too still, and it does not tire. **Flee** and you break away clean; **follow it** (PushThrough) and the wood takes the day from you — chased to exhaustion, and on a poor turn led truly astray, run down to nothing (handing you to the collapse funnel). Deniable as old light through the trunks. The elk you *chase* (distinct from the HollowStag you only glimpse).

**Mythic Phase 2 — the näšvyly's miasma (#455)** — the forest-fever shape (#450) now earns its name: meeting it carries a fortune-leaned chance to leave a wood-fever behind (it keeps upwind, the air where it stood smells of rot and wet ash). Deniable as ever — you were deep in cold wet forest, of course you took a fever — but it ties the myth creature into the disease/mortality system (#448): the fever can be tended (#451) or, untreated, run its course. First step of Mythic Phase 2.

**Real homesteads — the rural layout (#458, groundwork)** — `gen::building::lay_homestead`: a single country holding — a dwelling (cottage or longhouse) and an outbuilding (barn/shed) around a trodden walkable yard, with a worked field beside them. Deterministic; returns the placed buildings. The scattered-holding counterpart to `lay_district`, for the open country, not a town. Additive — not yet wired into worldgen / the founding system.

**Real towns — the district layout (#458, groundwork)** — `gen::building::lay_district`: lays a block of real buildings on walkable streets — varied `lay_building` structures (hut→manor) on plots with yard/street margins, every door opening onto a street, some plots left as open yards, deterministic per seed. Returns the placed buildings so the town/enclave generators can map services and occupants onto their doors. Still additive — not yet wired into worldgen (migrating towns onto this is the next, load-bearing step); the existing grid-town system is untouched and green.

**Real buildings — the primitive (#458, groundwork)** — toward a true open-world map (one continuous map, human scale, buildings *in* the world like Stoneshard): new terrain `Wall` (impassable) / `Floor` (walkable interior) / `Door` (passable entry), and `gen::building::lay_building` — a structure is a wall border around a walkable floor you walk into through a doorway, in varied styles (hut, cottage, longhouse, hall, manor), not a 1-tile token. Foundation primitive only; the richer procedural town/enclave layouts that place these come next.

**Healing & herbalism (epic) — plague-year play (#457)** — the closing step. In a plague year the choices invert and bite: **crowd contagion** makes a settlement — the safe haven any ordinary season — the *most* likely place to catch it (×2.2), so the empty wild becomes the safer place to wait the plague out, at the cost of the town's healer and treatment. And the **healer's hazard** — tending the sick through a plague (#454) is how it finds the healer (fortune-leaned). Healing now carries real risk in the season it matters most. **Healing & herbalism epic complete** (tend self · treat villagers · forage supply · plague-year).

**Healing & herbalism (epic) — biome herblore (#456)** — the supply half: **`f` — forage** for medicinal herbs, biome-true to real boreal physic. The deep wood and the mire give freely, the open country less, the cold heights and sand and bare stone almost nothing; Frost thins it everywhere, a storm thins it more, and luck leans the haul — now and then turning up a stand of true physic (a rare potent find in rich ground). Foraging the forest and mire honours the forest-keeper. Gives the herbalist a reason to range, and a steady source of the remedies that tend the sick. Headless via `PlayerChoice::ForageHerbs` / REPL `forage`.

**Healing & herbalism (epic) — treat the sick (#454)** — the herbalist's work reaches the world: on the Talk screen, **`h` — heal** a sick villager with the remedies you carry (the same that tend your own sickness). It eases their case, and a healer is remembered — local standing rises, the bond warms (NPC memory), and the river-keeper Masa marks the mercy; the root-eye heals true here too. Refused only by those set hard against your kind. Makes NPC illness matter to the player and healing a way to build standing.

**Healing & herbalism (epic) — tend the sick (#451)** — the mortality work (#448/#449) made disease a real killer but left the player's counters passive (auto-tend on rest; a salve that answered only wounds). Now there's an active counter: **`t` — tend your sickness**. A brewed **herb-physic eases any fever** (the field answer a salve never gave — the gap that let plague and fever kill unchecked in the wild), a **salve** answers the wound-illnesses strongest, a bandage dresses what it can; each eases the case *and shortens its course*, so fewer days sick is fewer death-rolls. Fortune leans the brew. The **root-eye gift** heals true — it doubles the easing and can break a mild fever outright, paying the body like any working of the gift. Wired through the headless action surface (`PlayerChoice::TendSelf`, REPL `tend`). First step of the healing epic.

**Myth-tranche 1 — the old dreads, adapted (#450)** — four deniable uncanny `WildSpecies` drawn from Finno-Ugric myth, reskinned into Sorethel (never the myth's own name; every sighting keeps a sober explanation to hand, and reaches the journal in the Scar voice): **mõvali** (the death-river swan — an omen on still water, danger 0), **aludda** (the drowning bank-wight of the mire), **vuolma** (the deep-leviathan far off the shore, sibling to the silt-whale), and **näšvyly** (the forest-fever shape). Two mundane beasts of the same waters and woods — **otter** and **pine marten** — keep the strange a clear minority; per-biome roll-frequency is test-guarded (swamp/coast/forest stay mundane-majority). Roster 35 → 41.

**Mortality & the hard age (#449)** — a death-rate soak showed ~96% of lives ending in old age, disease killing no one. That is wrong for the setting: a pre-modern, post-Fall world (no medicine, the empire shattered into squabbling successor polities). Grounded the mortality in the era:
- **Disease can kill (`DeathCause::Sickness`)** — untreated illness now rolls a once-a-day `daily_mortality`: plague, a wound gone bad, a fever, a birth gone wrong. Gentler when fed, sheltered, or near a healer who can tend it; deadlier starving and in a plague year; fortune-leaned. Treatment is the counter, never immunity. Disease is the great leveller it was historically.
- **The roads have teeth (`check_turmoil`)** — the post-Fall peace is thin and unevenly kept; the ungoverned spaces between the forming nations are full of raiders. A night unsheltered in the open country risks a raid — a lawless baseline everywhere off a settlement, trebled when the province's polity and its rival are at open tension. It costs goods, and the worn and unlucky their lives (`DeathCause::Wounds`); a palisade or a guardian companion shortens the odds, a settlement's walls end them.
- Soak after: old age ~74%, sickness ~16%, raids/wounds ~7%, exposure ~3% — old age stays the common end of a careful, settled life, but the age now thins its people.

**The gift in the world, and the canon Five** — the gift system reaches every craft-sense and every NPC, and the five non-human peoples enter play as in-kind traders.

- **The deep and the tide (#439)** — all four senses now load-bearing: scale-hand aids trade (buys under / sells over the spread), still-sense settles the Calm encounter; every gift act shares the bodily cost (flame-fever, rupture).
- **NPCs carry the gift (#441)** — `Person.gift` rolled ~2.5% at generation; the gifted-crafter rumor names a real person; a settlement with a gifted smith/herbalist makes those goods truer and cheaper there.
- **The Khör rendezvous (#443)** — the first non-human people: cold-steppe folk on Tundra/Mountain barter härkä Hide+Food for metal, take no coin, do not haggle.
- **The Mëräk exchange (#445)** — the deep-sea people at the Coast tideline barter deep-fish + deep-glass for cloth and tools, never coin.
- **The Five complete (#447)** — the last three canon peoples enter as in-kind traders on their own ground: the **Tzäkhar** (Cave-mouth deep-smiths, worked iron + tools for surface food), the **Häl** (Forest canopy, salve + herbs + fruit for cloth/tools), the **She'ar** (desert edge, desert-game + succulent-physic for sun-warding cloth/tools). None take coin; the rate is the rate.

## v0.9.0 (2026-06-13)

**The Gift and Its Price (epic #424)** — the Deep World's signature magic finally enters the game: the rare, innate, body-costing craft-sensitivity of the novels.

- **The hidden Gift (#426)** — a per-life `Gift` rolled from the life-seed like Fortune; ~2.5% carry one of four senses (iron-ear/root-eye/still-sense/scale-hand → Oltzed/Keuru/Kukri/Masa), the rest craftless. Hidden, persisted, shows young or never.
- **The craft act and its cost (#427)** — a gifted crafter masters the work their sense answers (no botch, +1 yield) but pays the body: gift-strain past a day brings the flame-fever (lieska-kuume), three worked-to-the-bone days the chronic iron-ache (rauta-särky). Fortune-leaned.
- **Burnout & rupture (#428)** — reaching for the gift while doubly spent (flame-fever AND iron-ache) risks the rauta-huuta: the sense is gone forever, irreversible.
- **Heredity (#429)** — the gift runs in the blood: a gifted parent's heir is gifted ~35% (vs base 2.5%), usually the same sense, but the line can still go quiet.
- **Craftless worth (#430)** — the ~97.5% are not lesser: the undivided, un-taxed hand is steadier (craft-botch ×0.55) and never pays the gift's price.
- **Surfacing (#431)** — the gift reveals itself on first use; the cost and rupture speak in the journal; a rare gifted NPC crafter is heard of on the road.

## v0.8.0 (2026-06-13)

**Depth & consequence (epic #411)** — close the loops between luck, gear, wildlife, and polity.

- **Finish luck's reach (#412)** — fortune leans the last rolls: craft botch, NPC childbirth complication (by the mother's fortune), NPC aid in encounters.
- **Wildlife as a resource (#413)** — hunt & trap: `ItemType::Hide`, a Hunt action on danger ≤1 non-uncanny wildlife, drawing down (and recovering) region game-richness.
- **Gear that closes loops (#414)** — Hide → Leather → Warm Coat (softens harsh-weather decay) and Herb → Salve (speeds Infection/Venom recovery).
- **Polity depth + canon currency (#415)** — paired rivalries + deterministic seasonal tension → war-rumors, road-watch travel penalty, war-levy; residency-revoked gates new field claims; per-polity coin acceptance (no universal currency: merchant leagues full value, Remnant debased, grain/in-kind economies discount coin).
- **Wild species tranche 3 (#416)** — roster 26 → 35: water/high/geothermal/coast/deep-desert species + the uncanny sand-spirit.
- **A felt world calendar (#417)** — seasonal events (market fair, hard winter, plague year), deterministic, announced on the wind, each moving one mechanic.

## v0.7.0 (2026-06-13)

- **The luck system (#397–#409)** — a hidden per-life Fortune (one star, never shown, surfaced only as uncertain omens) leans the consequence rolls across the whole game: flee outcomes (#398, fleeing is a gamble not an exit), mortal run-downs + collapse-death reprieve (#400, #402), illness contraction / wound infection / venom (#404), gather yield + trade prices (#407), and weather exposure — the cursed bear the cold worse (#409). Cautious is not safe; you never know your luck.
- **Versioned growth epic (#386, all 6 items)** — journal + voice banks moved to `data/` RON (#388); signal fire holds the dark off, Infrastructure III (#390); canon crops, the Bronze Road four with flax→Cloth and winter-rye surviving frost (#392); wild species tranche 2 for desert/steppe/cave/mountain (#394); death-rate soak census instrument (#395); polity layer — the province pays its dues via hearth-tax and a debt ladder (#396/#405).

## v0.6.0 (2026-06-12)

- **Canon scale epic (#378, PRs #379–#381)** — carrying-capacity population model (water/arable/trade factors), canon tiers to 15k+ cities, hinterland grain imports + famine on road failure, 160×80 sectors with half-hour walking, CANON_CITIES wired into caravans/rumors, 72-tile city sprawl. SCALE.md binding.
- **Walkable towns (#372, PRs #373–#377)** — town streets/houses on the one map, walk-in doors (tavern serves on step-in), street life, gate on the map (menu retired), 80×40 sectors.
- **Lore epics (#363–#367)** — household children + blood-before-friendship, marriage, wild species, settlement footprints, infrastructure II (well, waymarker, palisade), elder-esteem balance fix (#371).
- **Building arc (#343–#358)** — player farming, stash + residency, homestead→hamlet growth, world re-population + land-taking, structure world-effects (hearth, waystation, shrine), infrastructure tier (trail, footbridge), penance-as-restitution tone fix.
- **Living world (#312–#341)** — settlement food economy with real farms, NPC construction, profession depth; real Tool/Bandage/Trap items, disease severity, discovery effects; NPC lifecycle (births, aging, deaths, inheritance); weather fronts; festivals + rumors; growth/decline (promotions, famine, ghost towns); crime witnesses + inter-people escalation; bonds, grief, chosen heirs; voice banks +80%; canon naming baseline; recorded-choice AI-play API.
- **Refactor (#382)** — ui/app.rs (5,669 lines) split into 11 per-domain modules; public API unchanged.

## v0.5.x

- v0.5.0 — roguelike playability: movement, fog, quit safety, wait key (#240–#245)
- v0.5.1 — glyph collision fix, weather indicator, expanded footer (#247, #248)
- v0.5.2 — settlement matching uses actual terrain positions (#249)

## v0.4.0

- World screen IS the roguelike map — @ walks terrain with hjkl (#239)

## v0.3.x

- v0.3.0 — campaign arc milestones (#213), trade routes + merchant caravans (#214), settlement politics (#215), crafting quality tiers + tool degradation (#216), companion AI depth (#217), fog of war (#218), perf profiling (#219)
- v0.3.1 — fmt fix, logo, README run instructions, CI workflow, audio events wired into gameplay (#201)
- v0.3.2 — charts.ron embedded via include_str!; no external data files

## v0.2.3

- **#222 Death scene enrichment** — DeathCause enum (Starvation, Exposure, Exhaustion, Wounds, Unknown) with cause-specific flavor text. Elder death ceremony header. Memorial stats now show settlements visited and quests completed. `settlements_visited`/`quests_completed` counters in MilestoneTracker.
- **#223 Encounter variety** — 4 rare encounter types (GodShrine, AncientRuin, HermitCamp, TravelingBard) at 3% base chance. 3 seasonal encounters (SpringBloom, HarvestMarket, WinterSurvivor). Settlements excluded from random encounters. `Encounter::roll` now takes `day` param for season.

## v0.2.2

- **#211 Audio feature-gate stub** — Split `audio` (hound, pure Rust WAV synthesis) from `audio-playback` (rodio, needs ALSA). `cargo build --features audio` works without ALSA.
- **#220 Accessibility audit** — Stance/reputation labels now show symbol+text (`++ ally`, `~ neutral`, `-- hostile`). Focus cursor uses `▸`. Help screen documents accessibility. Color never sole signal.
- **#221 UI animations** — Pulse on low vitals (<30%), flash border on encounter. `reduced_motion` setting + [p] toggle. `tick_count`/`flash_frames` in App, `pre_draw()` per-frame.
- **#212 Save migration tests** — v1 migration test, v2 RON roundtrip, data preservation test, migrate_v2_to_v3 stub. Save versioning policy documented in ARCHITECTURE.md.
- **#224 Config file support** — `~/.config/deep-world-tui/config.toml` (TOML) for display options (monochrome, high_contrast, reduced_motion). Zero-config: missing file = defaults. Invalid = fallback. OR-merged with settings.ron.

## v0.2.1

- High-contrast white-on-black theme + [h] toggle
- Balance harness v2 (3 tests, no SimState dependency)
- Remove dead code (`App.first_run`)
- Cargo.toml cleanup: reqwest default-features=false + rustls-tls; `PeopleKind::name()` endonyms
- Determinism audit (5 tests)