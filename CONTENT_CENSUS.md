# Content Census

A full inventory of the game's content axes against the design baseline:
**everything has a purpose, the world is living and breathing, decisions have
consequences.** Updated 2026-06-13 for v0.7.0 — added Crops (10), Polities (6),
Venom (Diseases 10→11), and the hidden Fortune/omens axis; the living-settlements
and real-items passes (#312–#313) had already closed the original wiring gaps.

Verdicts: **load-bearing** (mechanically distinct and applied in play),
**flavor** (sampled/displayed but no mechanical effect), **dead** (defined but
unreachable). The wiring pass moved most former dead/flavor content to
load-bearing; what remains is listed under *Open gaps*.

| Axis | Count | Load-bearing | Notes |
|---|---|---|---|
| Peoples (PeopleKind) | 25 | 25 | patron god, terrain gather bonus, inter-people bias, trade modifier, fetch-quest item, voice bank, wants table |
| Gods | 5 | 5 | collapse weighting, gather boon (>0.5), encounter actions, service affinity, titles, profession bias |
| Animals (companions) | 16 | 16 | gather (Dog/Hound), travel (Horse), carry (Ox/Donkey), scout sight (Falcon/Crow), milk (Goat/HighlandGoat), per-animal upkeep rates (incl. zero-upkeep Eel/Crane/Lizard); rest yields, mood, departure |
| Professions | 21 | 21 | schedules + illness/trade hooks; fisher/sailor/herder/beast-handler feed the food economy, soldier/fence-builder/path-finder grant Safety, singer grants Presence, carpenter/labourer/forester/miner/weaver drive NPC construction (#312) |
| Personality traits | 19 | 19 | all traits now hit personality/trade/encounter modifiers |
| Diseases | 11 | 11 | terrain contraction, recovery window, vitals-decay rate; childbirth gated to those who can give birth; `severity` grows while untreated and scales vitals decay (#313); Venom carried in on a bite, not the land (#404), fortune-leaned contraction/infection |
| Encounter kinds | 25 | 25 | terrain/season/rarity-gated spawn, distinct action sets; **#443** adds KhorTrader — the first non-human people (Khör), cold-terrain-only (Tundra/Mountain), barter härkä goods (Hide+Food) for metal (Tool/Iron), no coin, no haggle |
| Encounter actions | 8 | 8 | time/energy/hunger costs, god affinity, reputation + NPC-memory deltas |
| Collapse outcomes | 10 | 10 | distinct losses/restores/hours, god-affinity weighted, all reachable |
| Weather | 11 | 11 | travel time, forced shelter, gather yield, encounter rate, vitals decay, NPC mood |
| Seasons / TimeOfDay | 3 / 7 | all | gather mult, decay mult, bias, festivals / darkness gates, service hours |
| World events (WorldEvent) | 3 | 3 | seasonal calendar (#417), deterministic per seed+season+year, announced in rumor: Market Fair (cheaper market), Hard Winter (Frost only — deeper weather decay), Plague Year (illness contraction up); each moves one mechanic and reverts at the season turn |
| Quest kinds / rewards | 5 / 3 | all | deterministic gen, distinct checks, fetch consumes goods |
| Milestones | 11 | 11 | all fire (verified call sites) |
| Journal voices | 6 | 6 | Encounter, Travel, Rest, Scar, Dream (Kukri), Rumor (taverns) |
| Items | 18 | 18 | price/trade/gather/craft/structure costs; Cloth gathers from flax (#392, no longer trade-only); Hide (hunt/trap, #413) → Leather → Coat (softens harsh-weather decay), Herb → Salve (speeds Infection/Venom recovery) — the gear chain (#414) |
| Wild species (WildSpecies) | 35 | 35 | terrain/season-true roster across all biomes; tranche 3 (#416) adds Pike/BrookTrout (water), AlpineVole/GoldenEagle/ForgeLizard (high & geothermal), PillarCrab/SiltWhale (coast/deep), SandSwimmer + the uncanny SandSpirit (desert); danger 0/1/2, huntable where danger ≤1 & not uncanny, per-species hunt_yield + encounter line; uncanny stays rare |
| Wildlife as resource | danger 0/1 huntable | load-bearing | a Hunt encounter action (danger ≤1, non-uncanny) and the set-snare on rest yield Hide + Meat (Food), fortune-leaned, scaled by and drawing down region `game_richness` (recovers seasonally); danger-2 stays a fight (#413) |
| Settlement services | 8 | 8 | all generated: Tavern/Temple (size), Forge=Sepat, Hearth=Ahjo, TrapWorkshop=Metsik, Archive=Arkit, TradePost=Väylä, Shrine=Laakso |
| Build kinds (player) | 8 | 8 | cost/hours/decay/maintain + rest tier (Tarp→Campfire … Home→Inn) |
| Terrains | 13 | 13 | passability, travel hours, gather item, people bonus, patron god, encounters, disease |
| Region types / sizes | 6 / 4 | all | chart-driven terrain mix, settlement count/size, services, companion capacity |
| Craft recipes | 9 | 9 wired | Tool/Bandage/Trap (#313) + Leather/Warm Coat/Salve — the gear chain that bridges hunting → warmth and herbs → healing (#414); craft can botch, fortune-leaned (#412) |
| Discoveries | 12 | 12 | per-kind effects: god affinity, thirst/energy refresh, map reveal (#313) |
| Crops (CropType) | 10 | 10 | per-crop terrain/season growth + yield; the Bronze Road four (Flax→Cloth closes the trade-only gap, WinterRye survives Frost, DroughtMillet, SnowPea) plus Grain/RootVegetable/Flatroot/Berry/Herb/Mushroom (#392) |
| Polities (Polity) | 6 | 6 | province ownership, hearth-tax + debt ladder (#396/#405); **#415**: paired rivalries + deterministic seasonal tension → war-rumors, road-watch travel penalty, war-levy on the hearth-tax; residency-revoked gates new field claims; per-polity coin acceptance (merchant leagues full value, Remnant debased, grain/in-kind economies discount coin) — canon "no universal currency" |
| Gift (CraftSense) | 4 senses + none | load-bearing | the rare innate craft-gift (#426, v0.9): ~2.5% of lives carry iron-ear/root-eye/still-sense/scale-hand (→ Oltzed/Keuru/Kukri/Masa), the rest craftless; rolled once from the life-seed like Fortune, hidden, persisted. **#427**: a gifted crafter masters the work their sense answers (no botch, +1 yield) but pays the body — gift-strain past a day's measure brings flame-fever (lieska-kuume), three worked-to-the-bone days running the chronic iron-ache (rauta-särky); fortune-leaned. **#428**: reaching for the gift while doubly spent (flame-fever AND iron-ache) risks the rauta-huuta — the boundary breaks and the sense is **gone forever** (gift → none, persisted, irreversible), fortune-leaned. **#429**: the gift runs in the blood — an heir of a gifted parent is gifted ~35% (vs base 2.5%), usually the same sense, but the line can still go quiet ("the children do not hear"); a craftless line keeps the rare base chance (`Gift::roll_heir`). **#430**: the craftless (~97.5%) are not lesser — they never pay the gift's bodily cost, and their undivided, un-taxed hand is steadier at ordinary work (craft-botch ×0.55 vs the gifted reaching outside their sense). **#431**: the gift surfaces to its bearer the first time it's used (`CraftSense::revelation`, once), the cost/rupture already speak in the Scar journal voice, and a rare gifted NPC crafter is heard of on the road (`npc_rumor`, deterministic per settlement). **Gift epic (v0.9) complete.** **#439**: all four senses now load-bearing — iron-ear/root-eye aid crafting, **scale-hand** aids trade (buys under / sells over the spread, gift-free clamp), **still-sense** aids the Calm encounter action (the beast stills); every act shares `App::use_gift` (reveal + bodily cost), so trade/calm flame-fever & rupture like crafting. **#441**: NPCs carry the gift too — `Person.gift` rolled ~2.5% at generation; the gifted-crafter rumor names a *real* gifted person; a settlement with a gifted iron-ear smith / root-eye herbalist makes those goods truer & cheaper there (×0.85). |
| Fortune / omens | 1 star | load-bearing | one hidden per-life Fortune (deterministic from life-seed, never shown), leans flee outcomes, mortal run-downs, collapse-death reprieve, illness/infection/venom, gather yield, trade prices, weather exposure; surfaced only as uncertain omens that polarity-lean but never lock (#397–#409) |
| charts.ron | 17 sections | all | generation fully data-driven |

## Open gaps

None. The #310 deferred list was closed by #312 (settlement food economy with
real farms/CropTypes, NPC construction of BuildingTypes, profession depth) and
#313 (Tool/Bandage/Trap items, Disease.severity, discovery effects).

## Consequence chains (the "decisions matter" audit)

Player actions feed: god affinity (gather terrain, encounter actions, services,
collapse outcomes) → collapse weighting + gather boons + talk bonuses;
reputation (encounter conduct ×witness, quests, elder status) → prices,
engagement, outside help in encounters; inter-people bias (tension events,
festivals) → encounters, service refusal, prices; NPC memory (per-person trust)
→ talk/trade success; lineage (death cause, heirs) → generational play;
illness (terrain/hunger/shelter choices) → vitals drain → collapse risk;
weather (when to travel/gather/rest) → time, yield, encounters, decay;
structures (build/maintain) → rest tier + decay upkeep;
fortune (hidden, per-life) → a thumb on every consequence roll above (flee,
run-down, reprieve, illness/infection/venom, gather, trade, exposure), known
only as omens — cautious is not safe, you never know your luck. Each loop closes.
