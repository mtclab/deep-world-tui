# Balance Parameters

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

1. **Survival should be achievable but not trivial.** A moderately skilled player should survive 40-200 days on average. Early death (<10 days) indicates broken balance.

2. **Vitals create urgency, not tedium.** The 0.05 hunger/h decay means ~6h of activity before auto-consumption kicks in at 0.7h remaining. This gives 3-4 gather/rest cycles per waking period.

3. **Frost is the pressure valve.** 1.3x decay + 0.3x gather yield creates a genuine resource squeeze. Players must store food before Frost or rely on settlement services.

4. **Collapse is a safety net, not a softlock.** The 1.2% death rate means most collapses are recoverable. Hostile outcomes are harsh but survivable. Continue-as-NPC ensures permadeath doesn't end the session.

5. **Services are the economic sink.** Coin drains through services, tools decay, and encounter bribes. The economy must circulate: gather, trade, service, repeat.

6. **Diversity through seeds.** Different seeds produce different terrain, settlements, NPCs, and encounter patterns. The 50-seed harness validates that no seed produces impossible or trivially easy configurations.

7. **Inter-people tension is the long-term challenge.** Bias modifiers compound over time. A player among hostile peoples pays more for services and gets worse collapse outcomes. Migration or reputation-building are the counter-strategies.

## Test Harness

`tests/balance_test.rs` runs 50 deterministic seeds through 200 ticks of simple AI:
- Gather when hunger < 0.4
- Rest when energy < 0.3
- Use settlement services when vitals are critical and coins available
- Random move/gather/rest otherwise

Asserts:
- All seeds complete without panic
- No seed dies before day 10
- Average survival 40-200 days
- At least one seed has vitals drop below 0.5 (game isn't trivial)