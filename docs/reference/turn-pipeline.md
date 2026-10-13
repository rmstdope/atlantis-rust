# Turn pipeline

Purpose: the exact sequence a turn runs, so the Rust `turn` module can reproduce phase order and
random-number draw order.

## Process entry (`main.cpp`)

1. `initIO()` seeds the RNG with 1783 (`gameio.cpp:54`).
2. Prints `Atlantis Engine Version: <ver>` and `<RULESET_NAME>, Version: <ver>` then a blank line.
   These lines are part of `engine-output.txt` in the fixtures.
3. With no arguments prints usage and exits 0.
4. `game.ModifyTablesPerRuleset()` runs before any subcommand (`main.cpp:67`).
5. `run` = `OpenGame()` → `RunGame()` → `SaveGame()`.

## `OpenGame` (`game.cpp:316`)

- Reads `game.in`, checks engine and ruleset versions (see `savegame-format.md`).
- `seedrandom(<saved int>)` at `game.cpp:400`, after year and month are read.
- Reads factions, regions, quests; calls `SetupUnitNums`.
- Prints `Saved Game Engine Version`, `Saved Rule-Set Version`, `Reading the regions...`,
  `Setting up the neighbors...` (visible in `engine-output.txt`).

## `RunGame` (`game.cpp:1045`)

| Step | Call | Notes |
|---|---|---|
| 1 | `PreProcessTurn` (`game.cpp:1108`) | `month++`, wrap to `year++` after month 11; `SetupUnitNums`; `Faction::DefaultOrders` (clears activity, numshows); per region `SetWeather` if `WEATHER_EXISTS`, `SetGateStatus` if `GATES_NOT_PERENNIAL`, `ARegion::DefaultOrders` → `Unit::DefaultOrders`. **RNG:** wandering-monster default moves (`unit.cpp:716-834`). |
| 2 | `ReadPlayers` (`game.cpp:515`) | `players.in`; may create factions (`AddFaction` → `SetupFaction`), units, items, skills, orders. |
| 3 | game status | Aborts if FINISHED, else sets RUNNING. |
| 4 | `ReadOrders` (`game.cpp:1145`) | For each non-NPC faction, parses `orders.<n>`. Instant orders execute here. `DefaultWorkOrder()` runs after **each** faction's file. |
| 5 | `RemoveInactiveFactions` | If `MAX_INACTIVE_TURNS != -1`. |
| 6 | `RunOrders` (`runorders.cpp:30`) | Phase table below. |
| 7 | `WriteWorldEvents` | If `WORLD_EVENTS`. **RNG:** `Events::Write`, and `WriteTimesArticle` file naming. |
| 8 | `WriteReport` | `MakeFactionReportLists`, `CountAllSpecialists`, `CountItems` (if statistics), `report.<n>`. |
| 9 | `WriteTemplates` | `template.<n>`. |
| 10 | `battles.DeleteAll`, `EmptyHell` | Dead units are freed only now; reports still reference them. |
| 11 | `WritePlayers` | `players.out`. |
| 12 | `DeleteDeadFactions` | |

Then `SaveGame` (`game.cpp:434`) writes `game.out`, drawing `getrandom(10000)` as the next seed
(`game.cpp:449`). This is the last draw of the turn.

## `RunOrders` phases (`runorders.cpp:30-119`)

| # | Call | RNG |
|---|---|---|
| 1 | `RunFindOrders` | |
| 2 | `RunEnterOrders(0)` | |
| 3 | `RunPromoteOrders` | |
| 4 | `DoAttackOrders`, `DoAutoAttacks` | yes: monster targeting (`runorders.cpp:1411-1413`), all of combat, battle facts |
| 5 | `RunStealOrders` | yes: AMTS chance (`unit.cpp:1941`) |
| 6 | `DoGiveOrders` | |
| 7 | `RunEnterOrders(1)` | |
| 8 | `DoExchangeOrders` | |
| 9 | `RunDestroyOrders` | |
| 10 | `RunPillageOrders` | yes: `Pillage()` |
| 11 | `RunTaxOrders` | |
| 12 | `DoGuard1Orders` | |
| 13 | `ClearCastEffects`, `RunCastOrders` | yes: spells |
| 14 | `RunSellOrders` | yes: contested market rounding (`runorders.cpp:1558`) |
| 15 | `RunBuyOrders` | yes: (`runorders.cpp:1696`) |
| 16 | `RunForgetOrders` | |
| 17 | `MidProcessTurn` → `MonsterCheck` if `CHECK_MONSTER_CONTROL_MID_TURN` | yes (`game.cpp:1672-1773`) |
| 18 | `RunQuitOrders` | |
| 19 | `DeleteEmptyUnits` | |
| 20 | `DoWithdrawOrders` | if `ALLOW_WITHDRAW` |
| 21 | `RunMovementOrders` (`monthorders.cpp:38`) | yes: movement attacks, gateways |
| 22 | `SinkUncrewedFleets`, `DrownUnits` | |
| 23 | `FindDeadFactions` | |
| 24 | `RunTeachOrders` | |
| 25 | `RunMonthOrders` (`monthorders.cpp:1043`) | yes: harvest quest checks (`quests.cpp:262`), `monthorders.cpp:1809` |
| 26 | `ProcessEconomics` (`runorders.cpp:1195`) | yes: `Grow` on visited regions, if `DYNAMIC_POPULATION || REGIONS_ECONOMY` |
| 27 | `RunTeleportOrders` (`spells.cpp:2109`) | yes |
| 28 | `CheckTransportOrders`, `RunTransportOrders` | if `TRANSPORT & ALLOW_TRANSPORT` |
| 29 | `AssessMaintenance` (`runorders.cpp:2015`) | yes: starvation (`unit.cpp:1621,1654`) |
| 30 | `ProcessMigration` | body starts with `return;` (`runorders.cpp:1212`) |
| 31 | `PostProcessTurn` (`runorders.cpp:1232`) | yes: `CheckVictory`, `ARegion::PostTurn`, `AdjustCityMons`, `MonsterCheck` (if not mid-turn), `GrowWMons`, `GrowLMons`, `GrowVMons` |
| 32 | `DeleteEmptyUnits`, `RemoveEmptyObjects` | |

`PostProcessTurn` calls `EndGame` if no faction exists, otherwise `CheckVictory` unless `OPEN_ENDED`.

## Order lifecycle

1. Parse in `parseorders.cpp`; `Parse1Order` maps the verb, `ProcessOrder` dispatches.
2. Phased orders are stored on the unit (single slot or list).
3. Instant orders change state during parsing, so line order matters for them.
4. Phases drain the slots; `Unit::ClearOrders` resets; `TurnOrder` blocks are re-emitted into the
   next template rather than executed.

## Porting notes

Target: `crates/atlantis-engine/src/turn/` (skeleton and phases), `api.rs` (`TurnInput` /
`TurnOutput`). Keep the phase list as one function in this order. The stdout lines are part of the
contract, so the engine returns them in `TurnOutput.stdout`. Times file naming needs the set of
existing `times.*` names as input (see `rng-and-determinism.md`).
