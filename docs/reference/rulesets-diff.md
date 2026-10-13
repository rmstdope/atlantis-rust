# Ruleset differences: standard ("Wyreth") vs neworigins

Purpose: every known behavioural difference between the two rulesets the replica supports, and the
complete hook surface a ruleset provides.

## Hook surface (link-time in C++; nothing is virtual)

| File | Provides |
|---|---|
| `rules.cpp` | `GameDefs *Globals` (positional aggregate); `allowedMages`, `allowedApprentices`, `allowedTaxes`, `allowedTrades`, `allowedQuartermasters`, `allowedTacticians`, `allowedMartial` with `*Size` |
| `extra.cpp` | `Game::SetupFaction` (from `AddFaction`, `game.cpp:1291`, and genrules), `Game::CheckVictory` (`runorders.cpp:1249` unless `OPEN_ENDED`), `Game::ModifyTablesPerRuleset` (`main.cpp:67`); neworigins also quest creation |
| `monsters.cpp` | `Game::CreateVMons` (`NewGame`), `Game::GrowVMons` (`runorders.cpp:1275`) |
| `world.cpp` | `SetupNames`, `CountNames`, `AGetName`, `AGetNameString` (also used by `economy.cpp:800,815`), `Game::CreateWorld`, `ARegionList::GetRegType`, `GetLevelXScale`, `GetLevelYScale` (used by `GetPlanarDistance`), `CheckRegionExit`, `GetWeather`, `GetStartingCity`, `ARegion::CanBeStartingCity`, `MakeStartingCity`, `IsStartingCity`, `IsSafeRegion` |
| `map.cpp` | `ARegion::CheckSea`; `ARegionList::Create{Abyss,Nexus,Surface,Island,Underworld,Underdeep}Level`, `MakeRegions`, `SetupNeighbors`, `MakeIcosahedralRegions`, `SetupIcosahedralNeighbors`, `MakeLand`, `MakeCentralLand`, `MakeIslands`, `MakeOneIsland`, `CleanUpWater`, `RemoveCoastalLakes`, `SeverLandBridges`, `SetRegTypes`, `SetupAnchors`, `GrowTerrain`, `RandomTerrain`, `MakeUWMaze`, `AssignTypes`, `UnsetRace`, `RaceAnchors`, `GrowRaces`, `FinalSetup`, `MakeShaft`, `MakeShaftLinks`, `SetACNeighbors`, `InitSetupGates`, `FixUnconnectedRegions`, `FinalSetupGates` |

Header comments in `game.h:420`, `game.h:444`, `unit.h:175` claiming `Allowed*`, maintenance/hunger and
attribute functions live in `extra.cpp` are stale: they are in the engine (`game.cpp`, `unit.cpp`).

## Faction-limit arrays

| Array | standard | neworigins |
|---|---|---|
| Mages | 0,1,2,3,5,7 | 1,2,3,4,5,6 |
| Apprentices | 0,2,4,6,10,14 | 1,3,5,7,10,15 |
| Taxes | 0,10,24,40,60,100 | 0,15,30,50,75,100 |
| Trades | 0,10,24,40,60,100 | 0,15,30,50,75,100 |
| Quartermasters | 0,2,4,8,12,20 | 0,2,5,9,14,25 |
| Tacticians | 0,1,2,4,6,10 | same |
| Martial | 0,10,25,40,60,90 | same |

## `GameDefs` values that differ

| Field | standard | neworigins |
|---|---|---|
| RULESET_NAME / VERSION | Wyreth / 2.0.0 | NewOrigins / 3.0.0 |
| WORLD_NAME | Wyreth | Atlantis |
| FACTION_ACTIVITY | DEFAULT (war/trade/magic) | MARTIAL_MERGED (martial/magic) |
| LEADER_COST | 20 | 50 |
| START_MONEY | 5020 | 10000 |
| WORK_FRACTION / ENTERTAIN_INCOME | 3 / 20 | 5 / 30 |
| WHO_CAN_TAX | TAX_NORMAL | TAX_NORMAL \| HORSE_AND_RIDING_SKILL \| MAGE_DAMAGE |
| TAX_PILLAGE_MONTH_LONG | 0 | 1 |
| CITY_POP | 20000 | 10000 |
| WMON_FREQUENCY / LAIR_FREQUENCY | 10 / 10 | 30 / 30 |
| TIMES_REWARD | 50 | 200 |
| SKILL_LIMIT_NONLEADERS | 1 | 0 |
| WEATHER_EXISTS | 1 | 0 |
| MORE_PROFITABLE_TRADE_GOODS | 0 | 1 |
| LASTORDERS_MAINTAINED_BY_SCRIPTS | 0 | 1 |
| START_CITIES_EXIST | 1 | 0 |
| APPRENTICES_EXIST | 0 | 1 |
| BATTLE_FACTION_INFO | 0 | 1 |
| CITY_RENAME_COST | 0 | 10000 |
| UNDERWORLD_LEVELS | 0 | 1 |
| TOWN_PROBABILITY / TOWN_SPREAD / TOWNS_NOT_ADJACENT | 100 / 0 / 0 | 90 / 100 / 100 |
| OCEAN / CONTINENT_SIZE / ARCHIPELAGO | 60 / 16 / 16 | 55 / 14 / 20 |
| ODD_TERRAIN | 0 | 70 |
| IMPROVED_FARSIGHT | 0 | 1 |
| LIMITED_MAGES_PER_BUILDING | 0 | 1 |
| TRANSIT_REPORT | REPORT_NOTHING | SHOW_REGION \| ROADS \| BUILDINGS \| GUARDS |
| USE_PREPARE_COMMAND | NONE | NORMAL |
| MONSTER_ADVANCE_MIN_PERCENT / HOSTILE_PERCENT | 15 / 0 | 0 / 100 |
| HAVE_EMAIL_SPECIAL_COMMANDS (FIND) | 1 | 0 |
| PROPORTIONAL_AMTS_USAGE | 0 | 1 |
| MONSTER_NO_SPOILS / MONSTER_SPOILS_RECOVERY | 0 / 0 | 2 / 2 |
| CHECK_MONSTER_CONTROL_MID_TURN, DETECT_GATE_NUMBERS, ONLY_ROUT_ONCE, FULL_TRUESEEING_BONUS, FULL_INVIS_ON_SELF | 0 | 1 |
| SKILL_PRACTICE_AMOUNT | 0 | 5 |
| UPKEEP_FOOD_VALUE | 10 | 50 |
| PREVENT_SAIL_THROUGH / ALLOW_TRIVIAL_PORTAGE | 0 / 0 | 1 / 1 |
| TRANSPORT / LOCAL_TRANSPORT | 0 / 1 | ALLOW \| QM_AFFECT_COST \| QM_AFFECT_DIST / 2 |
| ALLIES_NOAID, DISPERSE_GATE_NUMBERS, REGIONS_ECONOMY, ADVANCED_TACTICS, STRICT_GUARD | 0 | 1 |
| UNDEATH_CONTAGION | 0 | 33 |
| OCEAN_GUARD | 1 | 0 |
| BATTLE_LOG_LEVEL | NORMAL | VERBOSE |
| EXTENDED_FORT_DEFENCE, SPOILS_NO_TRADE, WORLD_EVENTS, FACTION_STATISTICS, BUILD_NO_TRADE, TRANSPORT_NO_TRADE, HALF_RIDING_BONUS | 0 | 1 |
| DESTROY_BEHAVIOR | INSTANT | PER_SKILL |

Same in both (selection): MAX_SPEED 8, PHASED_MOVE_OFFSET 7, FLEET_WIND_BOOST 2, STUDENTS_PER_TEACHER
10, MAINTENANCE_COST 10, STARVE_PERCENT 33, TAX_BASE_INCOME 50, FACTION_POINTS 5, GM_REPORT 1,
OPEN_ENDED 0, MAX_INACTIVE_TURNS 10, ABYSS_LEVEL 0, UNDERDEEP_LEVELS 0, NEXUS_IS_CITY 0,
START_CITIES_START_UNLIMITED 1, DECAY 0, REQUIRED_EXPERIENCE 0, MIN_DESTROY_POINTS 200,
MAX_DESTROY_PERCENT 34, OVERWHELMING 0, DYNAMIC_POPULATION 0. The full field list with both values is
in `gamedefs.h:90-803` and the two `rules.cpp` files; port it from source, not from this table.

## Ruleset code differences

| Aspect | standard | neworigins |
|---|---|---|
| `SetupFaction` silver | START_MONEY + turn × 50 | START_MONEY + turn × 300 |
| `SetupFaction` leader | mage with PATT, SPIR, GATE 30 days each; more skills from turn 25 | mage with OBSE, FORC, PATT, SPIR, GATE, FIRE 30 each, COMB 180, BEHIND set; placed in Nexus; unfriendly to monsters |
| `CheckVictory` | Black Keep empty → first guarding unit's faction wins (no abyss, so never fires) | always NULL; drives quests and Times articles |
| `CreateVMons` / `GrowVMons` | Balrogs in the Black Keep, +2 per turn up to 200 | no-ops |
| `ModifyTablesPerRuleset` | ~50 calls (`standard/extra.cpp:126-228`): tools, extra weapons/armour, roads, temples, sea lairs, QM/caravanserai, `ModifyItemProductionBooster(AXE, HAMMER)` | ~390 calls (`neworigins/extra.cpp:738-1235`): race roster (MAN, HILLDWARF, ICEDWARF, HIGHELF, WOODELF, GNOME, CENTAUR, LIZARD, GOBLIN, GNOLL, ORC, DROW, UNDERDWARF), terrain races and economy, adamantium at level 5, magic outputs, escape rules, shield/artifact/food skills, roads 30 stone at BUIL 2, many lairs and monsters, two `ModifyObjectName` calls |
| `GetRegType` underworld | ocean, cavern, underforest, tunnels at 1/4 each | out of 14: ocean 4, cavern 3, underforest 3, tunnels 2, chasm 2 (`case 11` falls through to ocean) |
| `GetRegType` surface arctic | 24/64 tundra | 32/64 tundra, plus tundra in the cold band |
| `CheckRegionExit` chasm | 60% | 40% |
| Map size / generator | fixed 64×64, classic | prompted; generator 1 or 2 |
| Naming | fixed table; "Unnamed" | syllables, `names.out`; "The Void" |
| `RemoveCoastalLakes` | surface only | also island, underworld, underdeep |
| `InitSetupGates` / `SetACNeighbors` | | exclude volcanoes |
| `FixUnconnectedRegions` | not run | run |

## Porting notes

Target: `crates/atlantis-engine/src/ruleset/` (`trait Ruleset`, `standard.rs`, `neworigins.rs`,
`Ruleset::from_name`) plus `src/defs/` for `GameDefs` values. Bugs in ruleset code (see
`quirks-to-preserve.md`) are part of the ruleset's behaviour.
