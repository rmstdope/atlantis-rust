# NPCs, quests and faction types

Purpose: city guards, monsters, monster control, quests and faction-type limits, so the Rust `npc`
module and quest code reproduce them.

## NPC factions

`CreateNPCFactions` at `new`: Guardsmen = faction 1 (`guardfaction`), Creatures = faction 2
(`monfaction`). NPC factions have type values -1 (`faction.cpp:885-897`), do not get errors, and
faction 1 receives the GM report.

## City guards (`game.cpp:1891-2114`)

- Created at full strength at start; regenerated at 10% strength with probability `GUARD_REGEN`.
- `AdjustCityMon` adds 10% of the maximum per turn. Hazards: `mantype` uninitialised when there are no
  men; `armor` may be -1 when re-equipping plate.
- Leader guards when `LEADERS_EXIST`; non-leader guards via `MakeManUnit` (`npc.cpp:315`), random
  weapon choice.
- Guard mages: Force and Fire at `START_CITY_MAGES`, cast Fire in combat.
- Guards have HOLDING (no help from neighbours) and default order GUARD.

## Wandering monsters (`npc.cpp`)

- `GrowWMons` (`npc.cpp:54`): 8×16 blocks; `wanted = ((sum(wmonfreq)/10 - mons) * rate +
  getrandom(100)) / 100`; random placement up to 1000 tries; bounds check uses `>` instead of `>=`.
- `MakeWMon`: small monster, 50% humanoid, 1/8 big monster; group size `(n + getrandom(n) + 1) / 2`.
- Movement in `Unit::DefaultOrders` (`unit.cpp:716-834`), during `PreProcessTurn`: a direction list
  with 4 "stay" entries, 2 per preferred/allowed direction, 1 per neutral terrain bordering preferred
  terrain; one `getrandom` picks; `getrandom(100) < aggression` makes it ADVANCE. Monsters do not
  enter guarded towns (`monthorders.cpp:1940`).
- Attack choice: see `combat.md`.

## Lairs and other monsters

- `GrowLMons`: empty lair objects spawn with `getrandom(100) < rate`; `MakeLMon` has special mixes per
  monster type.
- `GrowVMons` / `CreateVMons`: standard grows Balrogs +2 per turn up to 200 in the Black Keep
  (`standard/monsters.cpp`); NewOrigins stubs return immediately. Standard has no abyss level, so the
  Black Keep does not exist in practice.
- `MonsterCheck` (`game.cpp:1655`): escapes and decay of summoned creatures; mid-turn in NewOrigins
  (`CHECK_MONSTER_CONTROL_MID_TURN = 1`), post-turn in standard.

## Quests (`quests.cpp`, `quests.h`, `neworigins/extra.cpp:118-736`)

- Types: SLAY, HARVEST, BUILD, VISIT, DEMOLISH, with item rewards; stored in the save file.
- Completion is checked in engine code: kill in `GetSpoils`, harvest in `RunAProduction`
  (`quests.cpp:262` draws), build, demolish, visit (in `AssessMaintenance`).
- Created only by NewOrigins `CheckVictory` (`neworigins/extra.cpp:406`), which always returns NULL:
  1. Tallies visited vs unvisited populated regions; prunes each unit's `visited` to current VISIT
     destinations.
  2. Once visited ≥ 30% of populated regions, spawns quests: `QUEST_SPAWN_RATE` 7 attempts at
     `QUEST_SPAWN_CHANCE` 70%, max 20 active, topped up to min 5.
  3. Writes Times articles: exploration nudge, percentage, hint at an under-explored region, active
     quest descriptions. These draw from the RNG.
- `CreateQuest` (`neworigins/extra.cpp:118`): SLAY 60%, HARVEST 20%, BUILD/VISIT temple 20%; reward a
  random advanced or magic item worth about 3000-4500 silver (`QUEST_MAX_REWARD` 3000); `MAX_DESTINATIONS`
  5; uses printf (stdout), `std::map<string,...>` ordering, and a temple reroll loop.
- Standard `CheckVictory`: if the Black Keep is empty, the first guarding unit's faction wins.

## Faction types and limits

- Types: standard war/trade/magic; NewOrigins martial/magic (`game.cpp:49-58`). `FACTION_POINTS` 5.
- Limits: `Game::AllowedMages`, `AllowedApprentices`, `AllowedQuarterMasters` (better of TRADE and
  MARTIAL), `AllowedTacticians` (better of WAR and MARTIAL), `AllowedTaxes`, `AllowedTrades`,
  `AllowedMartial` (`game.cpp:1557+`) index the `allowed*` arrays from `rules.cpp`, clamped. Values
  per ruleset are in `rulesets-diff.md`.

## Porting notes

Target: `crates/atlantis-engine/src/npc/` (guards, monsters, monster control), quest storage and
checks in `src/model/` and the relevant phases, NewOrigins quest spawning in
`src/ruleset/neworigins.rs`. `std::map` iteration in `CreateQuest` maps to `BTreeMap` with the same key
ordering (byte-wise `std::string` compare).
