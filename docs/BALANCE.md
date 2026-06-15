# Balance Parameters

## AI Playtest Harness (v2)

The balance test suite includes an AI-driven harness:
- 15 seeds, 600 ticks (~25 days) per run
- AI gathers, eats, rests based on vitals thresholds
- Starting inventory: 3 food, 1 herb
- Assertions: AI survives ≥5/15 seeds, starvation kills within 500 ticks

## Vital Decay Rates (per hour)

| Vital | Base Rate | Frost Multiplier | Thaw/Green |
|-------|-----------|-------------------|------------|
| Hunger | 0.05 | x1.3 | x1.0 |
| Thirst | 0.06 | x1.3 | x1.0 |
| Energy | 0.02 | x1.3 | x1.0 |

At base rates, a player with full vitals (1.0) and no food/water in inventory:
- Hunger reaches 0 in ~20h (Thaw/Green) or ~15h (Frost)
- Thirst reaches 0 in ~17h (Thaw/Green) or ~13h (Frost)
- Energy reaches 0 in ~50h (Thaw/Green) or ~38h (Frost)

Auto-consumption triggers at 0.3: food restores +0.3 hunger, water restores +0.4 thirst.

## Season Multipliers

| Season | Gather Yield | Decay Rate | Bias Modifier |
|--------|-------------|------------|----------------|
| Thaw | 1.0x | 1.0x | 0.0 |
| Green | 1.2x | 1.0x | +0.05 |
| Frost | 0.3x | 1.3x | -0.05 |

## Rest Recovery (8h rest)

| Quality | Energy/h | Morale/h | Encounter Risk/h |
|---------|----------|----------|------------------|
| OutInCold | 0.04 | 0.02 | 0.18 |
| Campfire | 0.08 | 0.04 | 0.08 |
| LeanTo | 0.12 | 0.06 | 0.04 |
| SettlementFloor | 0.10 | 0.04 | 0.02 |
| Inn | 0.18 | 0.10 | 0.005 |

Base rest recovers +0.6 energy over 8h. Structure bonus adds up to +0.80 (Home).

## Collapse System

Collapse triggers when hunger <= 0 OR energy <= 0. On collapse:

- Death probability: 1.2% base (12/1000)
- Recovery hours: 6-16 per outcome
- Hostile outcomes reset hunger/energy to 0.15/0.1
- Player continues as related NPC on death

### Collapse Outcome Weights (base)

| Outcome | Weight | Type |
|---------|--------|------|
| Ditch | 150 | Harsh |
| BeastNest | 120 | Neutral |
| StrangerHut | 100 | Mild |
| WaysideShrine | 50 | Divine |
| Riverbank | 30 | Neutral |
| HostileBeast | 30 | Hostile |
| FestivalBench | 5 | Mild |
| BeastGuarded | 5 | Neutral |
| SettlementBed | 10 | Mild |
| GodCampsite | 3 | Divine |

God affinity and inter-people bias shift weights. High bias (>0.05) upgrades Ditch to StrangerHut. Low bias (<-0.15) downgrades mild outcomes to Ditch.

## Settlement Services

| Service | Cost (coins) | Effect | Time |
|---------|-------------|--------|------|
| Tavern | 2 | +0.4 energy, +0.2 hunger | 2h |
| Temple | 3 | +0.5 hunger, +0.3 energy | 3h |
| Forge | 2 | +2 Iron, repair tools | 3h |
| Hearth | 1 | +0.6 hunger, +0.5 energy | 2h |
| TrapWorkshop | 1 | +2 Herb | 2h |
| Archive | 2 | +0.4 energy, Sampsa +0.02 | 3h |
| TradePost | 1 | +2 Coin, Masa +0.02 | 2h |
| Shrine | 1 | +0.3 hunger, +0.3 energy, Kukri +0.03 | 2h |

Prices scale with inter-people bias and personality modifiers.

## Balance Philosophy

0. **Outcomes are organic, not targeted.** There is no death-rate quota and no artificial survival window the systems steer toward. A life's end *emerges* from how it was lived and the luck it was dealt — the hidden per-life star (#399) is the thumb on every consequence roll, not a per-event floor. A careful, well-fed life organically reaches old age; an exposed, worn, or unlucky one organically does not. We tune the *systems* (decay, exposure, illness, the luck lean), never the *result*. See "Measured outcomes" below — those numbers are observed, not designed-to.

1. **Survival should be achievable but not trivial.** The pressure window — where an ordinary, moderately-skilled life feels the squeeze and a careless one can die — sits around the first 40-200 days; that is when frost, hunger, the road, and bad luck bite hardest. Surviving *that* is the test. It is not a death target: a life that clears it and keeps its habits organically grows old (the soak's mean death day is ~300, of old age). Early death (<10 days) still indicates broken balance.

2. **Vitals create urgency, not tedium.** The 0.05 hunger/h decay means ~6h of activity before auto-consumption kicks in at 0.7h remaining. This gives 3-4 gather/rest cycles per waking period.

3. **Frost is the pressure valve.** 1.3x decay + 0.3x gather yield creates a genuine resource squeeze. Players must store food before Frost or rely on settlement services.

4. **Collapse is a safety net, not a softlock.** The 1.2% death rate means most collapses are recoverable. Hostile outcomes are harsh but survivable. Continue-as-NPC ensures permadeath doesn't end the session.

5. **Services are the economic sink.** Coin drains through services, tools decay, and encounter bribes. The economy must circulate: gather, trade, service, repeat.

6. **Diversity through seeds.** Different seeds produce different terrain, settlements, NPCs, and encounter patterns. The 50-seed harness validates that no seed produces impossible or trivially easy configurations.

7. **Inter-people tension is the long-term challenge.** Bias modifiers compound over time. A player among hostile peoples pays more for services and gets worse collapse outcomes. Migration or reputation-building are the counter-strategies.

## Measured Outcomes (organic, observed — not designed-to)

`tests/death_rate_soak.rs` (run `--ignored --nocapture`) plays whole lives to their end across 30 seeds each, in two cohorts: a **cautious** nester and a **bold** traveller who crosses real country worn. The point is not to hit a number — it is to confirm the systems produce an *organic* spread of fates. Most recent census (2026-06-15, 400-day cap):

| Cohort | OldAge | Sickness | Wounds | Exposure | Survived cap | Mean death day |
|--------|--------|----------|--------|----------|--------------|----------------|
| Cautious | 24 | 2 | 2 | 1 | 1 | 308 |
| Bold | 26 | 3 | 0 | 1 | 0 | 308 |

Reading it: a life that keeps its habits mostly reaches old age (~age 60-72), and a clear minority fall to sickness, the road, or the cold. **Cautious and bold land close together** — not because the road is safe, but because the town has its own dangers (plague, tension) and the wild has beasts: the risks are *different*, and they organically balance. That closeness is a feature of an organic world, not a knob to "fix": neither the careful life nor the bold one is the wrong way to play, and luck (#399) keeps either from being a guarantee. We change this table only by changing the *systems* it emerges from, and we read it after, never toward.

## Test Harness

`tests/balance_test.rs` runs 50 deterministic seeds through 200 ticks of simple AI:
- Gather when hunger < 0.4
- Rest when energy < 0.3
- Use settlement services when vitals are critical and coins available
- Random move/gather/rest otherwise

Asserts (`ai_survival_reasonable`): a simple gather/rest/service AI survives **at least 5 of 15** seeds — the world is livable but not a given. Companion checks: `starvation_works` (no food → vitals reach 0 within 500 ticks) and `gather_produces_resources` (gathering yields food and herbs). Whole-life fates are not asserted here — they are *observed* in the soak above, since an organic spread is a thing to read, not a thing to gate.