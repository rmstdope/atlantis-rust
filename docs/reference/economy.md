# Economy, production, trade and maintenance

Purpose: region economy setup and monthly change, production, taxing, trade and upkeep, so the Rust
`economy` module and the economic phases reproduce numbers exactly.

## Setup (world creation; `economy.cpp`)

- `SetupHabitat` (`economy.cpp:91`): race choice; coastal index is
  `coastal_races[n - noncoastal - 1]`, which reads index -1 when `n == noncoastal` (in practice the
  last `races` entry; verify against the reference). Habitat × (2/3 + random); population
  `habitat * (60 + 2*getrandom(6)) / 100`; development loop + `getrandom(25)`.
- `SetupEconomy` (`economy.cpp:156`): wealth `pop * (wages - 10 * MAINT) / 50`; work and entertainment
  use float with intermediate integer division, e.g. `wpfactor = (float)(120 / (61 - pp/50))`;
  entertainment `+1` here but `+10` in `SetIncome` (`economy.cpp:317`). Recruitment markets: race
  `pop / 25`, leaders `pop / 125`.
- `Wages()` (`economy.cpp:54`): triangular-number scan over development + roads + (earthlore +
  clearskies) × 12 + town term; stored ×10.
- `SetupPop` (`economy.cpp:229`): `TOWN_PROBABILITY`, `TOWN_SPREAD`, `TOWNS_NOT_ADJACENT`.
- `SetupCityMarket` (`economy.cpp:389`): long, many draws. `economy.cpp` has 59 `getrandom` calls.

## Monthly update

- `ProcessEconomics` (`runorders.cpp:1195`) calls `Grow()` on visited regions only if
  `DYNAMIC_POPULATION || REGIONS_ECONOMY` (NewOrigins only).
- `Grow` (`economy.cpp:1273`): basepop raise/lower with draws, long-int damped growth, `TownGrowth`
  with float `increase / limitingfactor`.
- `ARegion::PostTurn` (`economy.cpp:1544`): `DoDecayCheck`; `imp1 = improvement / 25` then up to 3
  random development rises; recovery rounds with draws; a taken-over starting city rebuilds markets
  (leader price ratio lacks the ×10); then `SetIncome`, `markets.PostTurn`, `UpdateProducts`
  (`economy.cpp:1055`; trade-building bonuses halve per building), unit `PostTurn`.
- `improvement` is not saved; the only reset is `aregion.cpp:135`, so it starts at 0 every turn.
- `Market::PostTurn` (`market.cpp:44-92`): men's prices from wages; other prices move 1/5 toward an
  activity target; amounts interpolated by population; float arithmetic; "hack: include wage factor
  of 10" (`market.cpp:63`, also `economy.cpp:214`).
- `MigrationAttractiveness`: `float space = 1/2` is 0; migration is disabled anyway
  (`ProcessMigration` returns immediately).
- `TownHabitat` tests `ItemDefs[...].flags & IT_FOOD` (a type bit on the flags field) and may index
  `ItemDefs[-1]`.

## Production (`monthorders.cpp`)

- `RunMonthOrders` per region: Idle, Study, `AddNewBuildings`, `RunBuildHelpers`, then per unit
  PRODUCE/BUILD, then `RunAProduction` per region product.
- `RunUnitProduce` (`monthorders.cpp:1055`): amount `men * level + GetProductionBonus` (tools), divided
  by `pMonths`; SKILLOUT gives men × pOut × level; SKILLOUT_HALF ×(level+1)/2; ORINPUTS consumes inputs
  in order; a PRODUCE target re-queues a turn order.
- `RunAProduction` (`monthorders.cpp:1290`): productivity `(int)(float)(men * level * prod / 10) +
  bonus`; each unit gets `(int)((double)amt * uatt / attempted)` in order; harvest-quest roll only if a
  matching quest exists.
- BUILD (`monthorders.cpp:516`): stone first, then wood; maintenance mode when `incomplete < 1`.
- Ships (`monthorders.cpp:659-1041`): incomplete ships are items; helpers decrement; the main builder
  creates the fleet.

## Tax, pillage, trade (`runorders.cpp`)

- Tax (`runorders.cpp:691-760`): `desired = max(total taxers, wealth)`; each unit gets
  `t * wealth / desired`, sequentially. `ActivityCheck` (`runorders.cpp:171`) enforces faction
  budgets; under MARTIAL_MERGED any recorded activity in a region counts once.
- Pillage: needs `pillagers * 2 * TAX_BASE_INCOME >= wealth`; pays 2 × wealth; `Pillage()` cuts
  development by a third with random population damage and a random development loop.
- Sell/buy (`runorders.cpp:1486-1740`): limited markets share stock with
  `(amount * num + getrandom(attempted)) / attempted`.

## Maintenance (`AssessMaintenance`, `runorders.cpp:2015`)

First pass sets `needed`, `hunger`, `visited` and checks visit quests. Then passes in order: unit
hunger, faction hunger, withdraw-food, ally hunger, unit food consumption, unit silver, faction
silver, unit food, faction food, unclaimed silver, ally silver, ally food. Then `Unit::Short`
(`unit.cpp:1588`): non-leaders then leaders, each man dies with `getrandom(100) < STARVE_PERCENT`.
Quirks: loop to `i <= NITEMS`; `if (u->needed < 1) break` only exits the object loop; negative
`needed` can make `eat` negative (adds items); ally-borrow functions call `SetNum` twice.

## Porting notes

Target: `crates/atlantis-engine/src/economy/` (setup, grow, markets, production pools) and the tax,
trade and maintenance phases in `src/turn/`. Use `f32` wherever C++ uses `float` and `f64` for
`double`, with the same evaluation order and the same int↔float casts.
