# Consequences — freedom with a price

> The spine of the game. The world never blocks your choice; it makes you live
> with it. Choices ripple through relationships, dependents, reputation, and the
> world's memory. Stories are emergent, not authored.

## Principle

**You can do anything. Everything costs something.** (The Conservation Principle,
applied to a life.) The game offers freedom — keep your trade, switch trades,
take to the road, abandon your household — and then simulates the fallout honestly.
No "are you sure?" hand-holding, no quest-fail screen. Just consequences that
arrive later, sometimes much later.

## What the world tracks (state the consequences pull on)

- **Relationships** — per-NPC bonds (spouse, children, kin, friends, rivals,
  patrons) with a strength/trust value and a *history* of what you did to them.
- **Dependents** — people who rely on you (a spouse, children, an apprentice, an
  aged parent). They have **needs** that decay without you (food, money, care,
  presence). Neglect/abandonment degrades their state — and your standing.
- **Reputation** — local + by faction/people; spreads by word of mouth (travels
  with traders, decays with distance + time). What you did follows you unevenly.
- **Obligations** — debts, contracts, promises, a guild membership, a lord's levy.
  Breaking them has creditors, courts, grudges.
- **World memory** — events are logged to entities + places; NPCs reference them;
  later encounters reflect them (a child grown up; a creditor's hired finder).

## How a choice becomes a consequence

1. **Act freely** (e.g. *leave town to go adventuring*).
2. **Immediate effects** — you gain freedom/opportunity (new road, new trades to
   learn) and incur immediate costs (lose your trade's income, leave dependents).
3. **Deferred effects** — over time, neglected dependents' needs decay; reputation
   shifts as word spreads; obligations come due; relationships cool or break.
4. **Re-encounter** — the world surfaces it later: a letter, a changed NPC, a
   creditor, a grown child, a closed door, an open one. The cost lands when it lands.

### Worked example — "leave the pregnant wife and go wandering"
- *Immediate:* freedom to travel + re-skill; lose household income + her support;
  spouse-bond takes a hit; reputation in your town drops ("he left her with child").
- *Deferred:* her + the child's needs decay without your support; the bond may
  break (she takes another, or hardship befalls them); your town reputation sours
  and travels via traders; if you return, the world has moved on — a child who
  doesn't know you, a wife who does, a town that remembers.
- *Player agency intact:* you might send money home (mitigate), come back (partial
  repair), or never look back (the cost stands). Or hardship at home was *why* you
  left — the consequences still apply; the game doesn't judge, it simulates.

## Design rules

- **Never block; always cost.** The game permits the choice and applies the price.
- **Costs are legible but not nagged.** Surface state (relationships/dependents
  panels, a journal) so the player *can* see the fallout — but don't moralize.
- **Deferred + re-encountered** beats instant punishment — consequences should
  arrive as *story*, later, where you'd half-forgotten.
- **Mitigation is possible** — most consequences can be softened by effort (send
  money, return, make amends), never trivially erased.
- **Emergent, not scripted** — consequences come from the simulation of needs +
  relationships + reputation + obligations, not from authored quest branches.

## Build note

Implement as a small **event/effect system** over the entity state: a choice emits
effects (immediate + scheduled/deferred); a tick advances time, decays needs,
spreads reputation, fires due events. Keep it data-driven + deterministic
(seeded), and `cargo test` the decay/spread/scheduling math. See
`ARCHITECTURE.md`.
