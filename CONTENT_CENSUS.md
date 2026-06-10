# Content Census

A full inventory of the game's content axes against the design baseline:
**everything has a purpose, the world is living and breathing, decisions have
consequences.** Updated 2026-06-10 after the wiring pass (PRs #307–#309).

Verdicts: **load-bearing** (mechanically distinct and applied in play),
**flavor** (sampled/displayed but no mechanical effect), **dead** (defined but
unreachable). The wiring pass moved most former dead/flavor content to
load-bearing; what remains is listed under *Open gaps*.

| Axis | Count | Load-bearing | Notes |
|---|---|---|---|
| Peoples (PeopleKind) | 25 | 25 | patron god, terrain gather bonus, inter-people bias, trade modifier, fetch-quest item, voice bank, wants table |
| Gods | 5 | 5 | collapse weighting, gather boon (>0.5), encounter actions, service affinity, titles, profession bias |
| Animals (companions) | 16 | 16 | gather (Dog/Hound), travel (Horse), carry (Ox/Donkey), scout sight (Falcon/Crow), milk (Goat/HighlandGoat), per-animal upkeep rates (incl. zero-upkeep Eel/Crane/Lizard); rest yields, mood, departure |
| Professions | 21 | 9 | farmer/herder/smith/trader/scribe/priest schedules, healer+herbalist (illness), trader (trade); 12 remain schedule-default flavor — see gaps |
| Personality traits | 19 | 19 | all traits now hit personality/trade/encounter modifiers |
| Diseases | 10 | 10 | terrain contraction, recovery window, vitals-decay rate; childbirth gated to those who can give birth; `severity` field unused — see gaps |
| Encounter kinds | 12 | 12 | terrain/season/rarity-gated spawn incl. MerchantCaravan (roads); distinct action sets |
| Encounter actions | 8 | 8 | time/energy/hunger costs, god affinity, reputation + NPC-memory deltas |
| Collapse outcomes | 10 | 10 | distinct losses/restores/hours, god-affinity weighted, all reachable |
| Weather | 11 | 11 | travel time, forced shelter, gather yield, encounter rate, vitals decay, NPC mood |
| Seasons / TimeOfDay | 3 / 7 | all | gather mult, decay mult, bias, festivals / darkness gates, service hours |
| Quest kinds / rewards | 5 / 3 | all | deterministic gen, distinct checks, fetch consumes goods |
| Milestones | 11 | 11 | all fire (verified call sites) |
| Journal voices | 6 | 6 | Encounter, Travel, Rest, Scar, Dream (Kukri), Rumor (taverns) |
| Items | 14 | 14 | price/trade/gather/craft/structure costs; Cloth is trade-only (no gather source) |
| Settlement services | 8 | 8 | all generated: Tavern/Temple (size), Forge=Sepat, Hearth=Ahjo, TrapWorkshop=Metsik, Archive=Arkit, TradePost=Väylä, Shrine=Laakso |
| Build kinds (player) | 8 | 8 | cost/hours/decay/maintain + rest tier (Tarp→Campfire … Home→Inn) |
| Terrains | 13 | 13 | passability, travel hours, gather item, people bonus, patron god, encounters, disease |
| Region types / sizes | 6 / 4 | all | chart-driven terrain mix, settlement count/size, services, companion capacity |
| Craft recipes | 6 | 6 wired | 3 outputs semantically off (Bandage→Food, Tool→Iron, Trap→Herb) pending Tool/Bandage/Trap item types |
| Discoveries | 12 | flavor | observable once, journal lore only (by design, for now) |
| charts.ron | 17 sections | all | generation fully data-driven |

## Open gaps (tracked, deliberate)

- **Farm/CropType system (3 crops)** — fully modeled (growth stages, weather
  bonus, yields), zero instances created in the world. Wire into settlement
  food economy or remove. (#310)
- **BuildingType (5, NPC construction)** — modeled with materials/ticks/energy,
  never instantiated; NPCs don't build. (#310)
- **12 schedule-default professions** — generate and color voice lines but have
  no mechanical hook (fisher, weaver, soldier, singer, …). Profession-depth
  pass. (#310)
- **Disease.severity** — persisted field, always 1.0, never read. Apply as a
  decay scalar or drop in a save-version bump. (#310)
- **Discovery rewards** — currently pure lore; candidate: small god-affinity
  or reveal effects per kind. (#310)
- **Craft output item types** — Tool/Bandage/Trap as real items (deferred with
  lore/content work).

## Consequence chains (the "decisions matter" audit)

Player actions feed: god affinity (gather terrain, encounter actions, services,
collapse outcomes) → collapse weighting + gather boons + talk bonuses;
reputation (encounter conduct ×witness, quests, elder status) → prices,
engagement, outside help in encounters; inter-people bias (tension events,
festivals) → encounters, service refusal, prices; NPC memory (per-person trust)
→ talk/trade success; lineage (death cause, heirs) → generational play;
illness (terrain/hunger/shelter choices) → vitals drain → collapse risk;
weather (when to travel/gather/rest) → time, yield, encounters, decay;
structures (build/maintain) → rest tier + decay upkeep. Each loop closes.
