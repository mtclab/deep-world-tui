# World Depth: Content Scale & Missing Mechanics

Companion to [CONTENT_CENSUS.md](CONTENT_CENSUS.md). The census asked *does
every piece have a purpose* (now: yes). This asks two harder questions:
**are the numbers enough**, and **are there enough mechanics** for a world
that reads as livable, organic, breathing — where decisions have consequences.

## 1. Are the numbers enough?

Measure content count against **exposure rate** — how often a player sees the
same entry repeat. A life-RPG with lineage loops replays its surfaces many
times per save.

| Axis | Count | Exposure | Verdict | Target |
|---|---|---|---|---|
| Journal/voice templates | ~5/situation | **every action** | repetition within days — the single most exposed surface | 3–4× banks, moved to data/ (chart-driven, per-people flavor) |
| Encounter kinds | 12 | every few tiles | repeats within 1–2 game days | 24–36, terrain-unique (mirage, river flood, cave-in, funeral procession, lost child, escaped livestock, plague wagon, pilgrim band, beast migration, border watch…) |
| Quest kinds | 5 | board always visible | samey within one life | 9–12 (Deliver-to-settlement, Escort, Cure-the-sick, Raise-a-building, Visit-discovery, Donate-to-stores, Vigil) |
| Discoveries | 12 | exploration driver | adequate but thin for multi-life play | 24 |
| Craft recipes | 8 | crafting loop | thin | 12–16 (+ item quality tiers already exist to build on) |
| Collapse outcomes | 10 | rare event | fine | +6 terrain-specific, later |
| Diseases | 10 | rare event | fine | hold |
| Animals 16 / Weather 11 / Terrains 13 / Peoples 25 / Gods 5 / Traits 19 / Professions 21 | — | various | healthy | hold |

Rule of thumb adopted: **content seen hourly needs dozens of variants; content
seen weekly needs ten.** The thin axes are exactly the hourly ones.

## 2. Are there enough mechanics?

What a "living, breathing" world still lacks, ranked by alive-feeling per
effort. Each has a tracking issue.

1. **NPC lifecycle** — nobody is born, ages, or dies of age; populations only
   move. The deepest "frozen world" tell. Yearly age-band progression, elder
   deaths (memorials already exist), births in spouse-households, profession
   inheritance. The player should return to a town and find the old healer
   gone and her daughter keeping the herbs.
2. **Festivals & a felt calendar** — season festival_chance exists but nothing
   happens. Festival state on a settlement: days of cheaper services, swapped
   encounter tables, god-affinity gains, journal color. Seasons become events,
   not just multipliers.
3. **Rumors that mean something** — the rumor channel now exists (taverns,
   building completions). Feed it real sim events: caravan arrivals, deaths,
   famines, migrations, festivals. Rumors become *information the player can
   act on* ("grain is dear in Aamukkäport" → carry food there, sell high).
   The food economy (#312) makes this profitable already.
4. **Weather fronts** — weather is an hourly hash; sun→blizzard→sun. A small
   Markov state per region/day with fronts drifting between neighbors makes
   sky continuity real. Cheap, large believability gain.
5. **Player social bonds** — NPC memory exists, but no named friendship,
   romance, marriage, household. Bond levels from repeated dealings →
   lodging, gifts, grief entries on their death; an heir who is your child
   rather than a stranger.
6. **Settlement growth & decline** — population should cross size thresholds
   (hamlet→village brings a Temple), and famine (food stock at zero for
   weeks) should empty a settlement — ghost towns the player can witness
   happening and even avert by delivering food.
7. **Wildlife ecology (light)** — regional game richness that Trap/Hunt yields
   and Wildlife encounter rates draw down and that recovers seasonally;
   over-trapping a valley empties it.
8. **Crime & justice** — the witness system exists; add theft as a choice,
   bounties, making-amends. Decisions get a darker consequence branch.
9. **Inter-people escalation** — TensionEvent only nudges bias. An escalation
   ladder (embargo → border watch → skirmish) with player-relevant effects
   (markets closed to you, road encounters) and resolution hooks (mediation
   quests, festivals).
10. **Legacy depth** — heirs inherit structures already; add a keepsake item,
    partial local reputation, and a family name the world remembers.

## 3. Already-closed loops (for orientation)

god affinity ← gather/encounters/services/discoveries → collapse weighting &
boons · reputation ← conduct×witness/quests → prices/engagement/rescue ·
food economy ← professions/farms/weather → stores → NPC needs → market prices ·
construction ← builder professions → buildings → needs/preservation/rumors ·
illness ← terrain/hunger/shelter → vitals → collapse → lineage · weather →
travel/gather/encounters/decay/mood · structures → rest tiers + decay/maintain.
