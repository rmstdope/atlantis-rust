# World generation (`new`)

Purpose: how `atlantis new` builds a world, per ruleset, and which parts are reproducible, so the Rust
`world` module can port the algorithms and choose a verification strategy.

## `Game::NewGame` (`game.cpp:275+`)

`factionseq=1`, `guardfaction=0`, `monfaction=0`, `unitseq=1`, `SetupUnitNums`, `shipseq=100`,
`year=1`, `month=-1`, status NEW; `seedrandomrandom()` (seed = `time(0)`); `CreateWorld()`;
`CreateNPCFactions()`; `CreateCityMons()` / `CreateWMons()` / `CreateLMons()` per flags;
`CreateVMons()`. `main` then calls `SaveGame` and `WritePlayers`.

## `Game::CreateWorld` (per ruleset)

Prompts (`Agetint` on stdin):
- Both: nexus size only if `MULTI_HEX_NEXUS` (0 in both, so nexus is 1×1).
- standard (`standard/world.cpp:2101`): width and height are initialised to 64, so the size prompts are
  dead code. No generator prompt.
- neworigins (`neworigins/world.cpp:376`): generator `1` (classic) or `2` (parametric, prompt text
  "Origianl - 1, Parametrical - 2"), then width and height (multiples of 8).

Sequence (neworigins shown; standard is the same minus noted parts):
1. `CreateLevels(2 + UNDERWORLD_LEVELS + UNDERDEEP_LEVELS + ABYSS_LEVEL)`.
2. `SetupNames()`.
3. `CreateNexusLevel(0, nx, ny, "nexus")`.
4. Surface: generator 1 `CreateSurfaceLevel(1, xx, yy, 0)`; generator 2 builds `Map(xx*2, yy*2)` with
   `redistribution 1.5`, `evoparation 0.75`, `mountainPercent 0.1`, `waterPercent 0.1` and calls
   `CreateNaturalSurfaceLevel(map)` (`aregion.cpp:3986`).
5. Underworld levels (`CreateUnderworldLevel`, scaled by `GetLevelX/YScale`), underdeep, abyss.
6. `CountNames()`.
7. `MakeShaftLinks`: odds 8 with one underground level, else 10 / `10*i-10` / more for deeper levels.
8. `SetACNeighbors(0, 1, ...)`: Nexus exits to starting cities, or "Gateway to <terrain>" objects
   when `START_CITIES_EXIST == 0` (neworigins).
9. `InitSetupGates` (up to 5 tries per 8×16 block), `FixUnconnectedRegions` (neworigins only,
   `neworigins/map.cpp:2851`), `FinalSetupGates` (random numbering, `gatemonth`).
10. `CalcDensities`, `TownStatistics`, `ResoucesStatistics` (stdout).

Level counts: standard `UNDERWORLD_LEVELS=0`, `ABYSS_LEVEL=0` → nexus + surface. neworigins
`UNDERWORLD_LEVELS=1` → nexus + surface + underworld.

## Classic surface generator (`<ruleset>/map.cpp`)

`MakeLand` (continents/archipelagos until ocean share reaches `OCEAN*(100+2*SEA_LIMIT)/100`),
`CleanUpWater`, `SetupAnchors` (`GetRegType` latitude bands, `getrandom(64)`, `TERRAIN_GRANULARITY`),
`GrowTerrain` (30 passes; lakes; `ODD_TERRAIN`), `AssignTypes` / `RandomTerrain`, `SeverLandBridges`,
`RemoveCoastalLakes`, `GrowRaces` (if enabled), `FinalSetup` (names, then `ARegion::Setup`:
production, population, towns via `economy.cpp:241`, dummy object, `LairCheck` against
`lairChance`). The `wages` field is reused as a name index during generation.

Underworld: `SetRegTypes(R_NUM)`, anchors, `GrowTerrain(growOcean=1)`, `MakeUWMaze` (removes exits by
`CheckRegionExit` percentages).

Function locations in neworigins: `neworigins/map.cpp:1646-3004` (`CreateAbyssLevel` …
`FinalSetupGates`). `neworigins/map.cpp:508-1630` (`MapBuilder`, `Zone`, `Province`) is dead code: its
only caller is commented out (`neworigins/map.cpp:1765-1786`).

## Natural generator (neworigins generator 2)

`Map::Generate` (`mapgen.cpp:418`): simplex elevation, sea and mountain thresholds, temperature, rain
simulation, biome matching. `makeRivers` (ocean chains broken by swamp), `cleanupIsolatedPlaces`,
`placeVolcanoes`, `GrowRaces`, `giveNames` (`namegen.cpp`, ethnic names), `economy` (Poisson-spaced
settlements via `getPoints`, `ManualSetup`), `AddHistoricalBuildings`. Prints to stdout; calls
`exit(1)` if no biome matches.

## Names

- standard: fixed `regionnames[]` table (about 2000 lines of `standard/world.cpp`); unnamed land
  "Unnamed".
- neworigins: syllable probability tables, 12 characters max, unique; writes `names.out`
  (`neworigins/world.cpp:234`); unnamed land "The Void".

## Randomness and reproducibility

- All classic-generator draws go through ISAAC `getrandom`. The seed comes from `time(0)`, so `new` is
  not reproducible as shipped, but a seed override in the reference copy makes generator 1
  reproducible, as far as this analysis found.
- Generator 2 also uses `std::default_random_engine` seeded with `time(0)` in `aregion.cpp:2936-2977`
  (river candidate shuffle) and `simplex.cpp:89-92` (permutation shuffle). Byte-exact reproduction
  would need those seeds fixed **and** libstdc++'s `minstd_rand0` and `std::shuffle` emulated.
- `std::random_device` / `mt19937` at `neworigins/map.cpp:1064` is in the dead `MapBuilder` code.

## Porting notes

Target: `crates/atlantis-engine/src/world/` (levels, classic generator, natural generator, names,
gates and shafts) plus ruleset hooks in `src/ruleset/`. `new` takes its answers as parameters
(generator, width, height, seed). Verification per the plan: structural invariants and statistical
comparison against the reference; byte-exact seeded generation is a stretch goal and looks feasible
for generator 1.
