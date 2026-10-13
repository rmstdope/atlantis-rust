# Data model

Purpose: the C++ classes, their persistent and per-turn fields, ownership, and the global tables, so
the Rust `model` and `defs` modules can hold the same state.

## Primitives

**`AList` / `AListElem`** (`alist.h:25-98`, `alist.cpp`)
- Intrusive singly linked list: `list` (head), `lastelem` (tail), `num`. Elements carry `next`.
- One object can be in one list at a time; shared references use wrapper elements (`UnitPtr`,
  `FactionPtr`, `ARegionPtr`, `BattlePtr`, `Location`, `Farsight`).
- `Add` appends, `Insert` prepends, `Remove` unlinks (linear scan for the tail), `Empty` unlinks all,
  `DeleteAll` frees all.
- `forlist(l)` (`alist.h:69`) prefetches `next` before the body runs. Deleting or moving the current
  element is safe. If the body moves the **prefetched** element into another list, iteration
  continues inside that other list. An element appended during the loop is visited unless the loop
  is already on the last element.
- `forlist_safe` snapshots element pointers and uses `NextLive` (O(n²)) to skip removed entries.

**`AString`** (`astring.h`, `astring.cpp`): see `orders-and-parsing.md` for tokenizer and
comparison semantics.

## `Game` (`game.h:59-603`)

- Persistent: `factions`, `regions` (`ARegionList`), `factionseq`, `unitseq`, `shipseq`, `year`,
  `month` (0-11), `guardfaction`, `monfaction`.
- Transient: `newfactions`, `battles`, `ppUnits` (non-owning index by unit number, rebuilt by
  `SetupUnitNums`, `game.cpp:1322`), `gameStatus`, `doExtraInit`, `events`.
- Globals outside the class: `QuestList quests` (`quests.h:80`), `FactionTypes`, `Globals`, the
  `allowed*` arrays.
- `TurnNumber() = (year-1)*12 + month + 1` (`game.cpp:70`).
- `SetupUnitSeq` recomputes `unitseq = max(unit num)+1` on load. `GetNewUnit` (`game.cpp:1366`)
  reuses the **lowest free** unit number before incrementing `unitseq`.
- `NewGame` sets `factionseq=1`, `unitseq=1`, `shipseq=100`, `year=1`, `month=-1`.

## `ARegion` (`aregion.h:177-382`)

| Group | Fields |
|---|---|
| Identity | `name`, `num`, `type` (R_*), `xloc`, `yloc`, `zloc` |
| Buildings, gates | `buildingseq`, `gate`, `gatemonth`, `gateopen` |
| Weather | `weather` (not saved) |
| Town | `TownInfo* town` (nullable): `name`, `pop`, `activity`, `hab`, `dev` |
| Economy | `race` (item index), `population`, `basepopulation`, `wages`, `maxwages`, `wealth` |
| Geography | `habitat`, `development`, `maxdevelopment`, `elevation`, `humidity`, `temperature`, `vegetation`, `culture` |
| Migration | `migfrom`, `migdev`, `immigrants`, `emigrants`, `improvement` (not saved) |
| Spell effects | `clearskies`, `earthlore`, `phantasmal_entertainment` (not saved) |
| Topology | `neighbors[6]` |
| Contents | `objects`, `newfleets` (`map<int,int>`), `fleetalias`, `hell` (dead units) |
| Visibility | `farsees`, `passers` |
| Lists | `products` (`ProductionList`), `markets` (`MarketList`) |
| Scratch | `visited`, `distance`, `next` (A*) |

`GetDummy()` returns the first `O_DUMMY` object; every unit lives in an object even in open terrain.

**`ARegionArray`** (`aregion.h:384`): `x`, `y`, `regions`, `strName`, `levelType`
(NEXUS/SURFACE/UNDERWORLD/UNDERDEEP).

**`ARegionList`** (`aregion.h:447`): `numberofgates`, `numLevels`, `pRegionArrays`. `GetRegion(n)` is a
linear scan; `GetRegion(x,y,z)` wraps x and y modulo level size. Holds most world-generation code.

## `Object` (`object.h:68-145`)

Fields: `name`, `describe`, `region`, `inner` (shaft target), `num`, `type` (O_*), `incomplete`,
`capacity`, `flying`, `load`, `runes`, `prevdir`, `mages`, `shipno`, `movepoints`, `destroyed`,
`units`, `ships` (Items, one per ship type). `IsFleet()` is true if `type==O_FLEET`, or
`ObjectDefs[type].sailors>0`, or `ships` is non-empty.

## `Unit` (`unit.h:119-302`)

- Persistent: `name`, `describe`, `num`, `type` (U_NORMAL/MAGE/GUARD/WMON/GUARDMAGE/APPRENTICE),
  `faction`, `guard` (NONE/GUARD/AVOID/SET/ADVANCE), `reveal`, `free`, `readyItem`,
  `readyWeapon[4]`, `readyArmor[4]`, `flags` (`FLAG_*` 0x1-0x4000), `items`, `skills`, `combat`,
  `savedmovement`, `savedmovedir`, `visited` (`set<string>`, sorted).
- Per turn: `formfaction`, `object`, `alias`, `gm_alias`, `taxing`, `movepoints`, `canattack`,
  `nomove`, `routed`, `needed`, `hunger`, `stomach_space`, `losses`, `practiced`, `moved`, `phase`,
  `raised`, `oldorders` (template lines).
- Order slots: scalars `destroy`, `enter`, `build`, `promote`; lists `findorders`, `giveorders`,
  `withdraworders`, `bankorders`, `buyorders`, `sellorders`, `forgetorders`, `exchangeorders`,
  `transportorders`, `turnorders`; single slots `castorders`, `teleportorders`, `stealorders`,
  `monthorders`, `attackorders`, `evictorders`, `joinorders`; misc `advancefrom`, `inTurnBlock`,
  `presentMonthOrders`, `presentTaxing`, `former`, `format`.

## `Faction` (`faction.h:128-233`)

`num`; `type` (`unordered_map<string,int>` keyed by `FactionTypes`: `{war, trade, magic}` when
`FACTION_ACTIVITY == DEFAULT`, else `{martial, magic}`, `game.cpp:49`); `lastchange`, `lastorders`,
`unclaimed`, `bankaccount`, `interest`; `name`, `address`, `password`, `times`,
`showunitattitudes`, `temformat`, `battleLogFormat`; `exists`, `quit`, `numshows`, specialist
counters; `activity` (`unordered_map<ARegion*, unordered_set<FactionActivity>>`);
`present_regions`; `defaultattitude`, `attitudes`; `skills` (shown skills); `items` (discovered
items); `extraPlayers`, `errors`, `events`, `battles`, `shows`, `itemshows`, `objectshows`; `pReg`,
`pStartLoc`, `noStartLeader`, `startturn`. `IsNPC()` when type values are -1 (`SetNPC`,
`faction.cpp:885-897`).

## Smaller classes

- `Market` (`market.h`): `type` (M_BUY/M_SELL), `item`, `price`, `amount`, `minpop`, `maxpop`,
  `minamt`, `maxamt`, `baseprice`, `activity`.
- `Production` (`production.h`): `itemtype`, `baseamount`, `amount`, `skill`, `productivity`,
  `activity`. Constructor draws `getrandom(maxamt)` when `RANDOM_ECONOMY`.
- `Item`: `type`, `num`, `selling`, `checked`. `Skill`: `type`, `days`, `exp` (unsigned).
- Battle classes: see `combat.md`.

## Global tables (`gamedata.h`, `gamedata.cpp`)

Flat arrays indexed by enums. Counts below are from the analysis at this commit and should be
confirmed by the table-dump tool.

| Table | Index | Element |
|---|---|---|
| `ItemDefs` | `I_*`, NITEMS 229 | `ItemType` (`items.h:79`) |
| `SkillDefs` | `S_*`, NSKILLS 106 | `SkillType` |
| `ObjectDefs` | `O_*`, NOBJECTS 76 | `ObjectType` |
| `TerrainDefs` | `R_*`, R_NUM 61 | `TerrainType` (`aregion.h:68`) |
| `ManDefs`, `MonDefs`, `WeaponDefs`, `ArmorDefs`, `MountDefs`, `BattleItemDefs` | keyed by item `abr` | `items.h:270-412` |
| `SpecialDefs`, `EffectDefs`, `RangeDefs`, `AttribDefs`, `HealDefs`, `MagicHealDefs`, `ShowDefs` | key strings or level | `skills.h:160-329` |

- `ModifyTablesPerRuleset` mutates these in place through ~85 helpers in `modify.cpp`. Each helper
  bounds-checks and returns silently on bad input. The effective tables only exist after mutation.
- `Parse*` lookups honour `DISABLED`; `Lookup*` generally do not.
- `Find{Skill,Special,Effect,Range,Attrib}` return null for unknown keys.
- `Soldier::effects` keys are `const char*` (pointer identity).

## `GameDefs` (`gamedefs.h:90-803`)

Positional aggregate (~400 fields) instantiated in `<ruleset>/rules.cpp`. Full per-ruleset values
are in `rulesets-diff.md`.

## Porting notes

Target: `crates/atlantis-engine/src/model/` and `src/defs/`. Use id-indexed arenas with `Vec<Id>` for
every list whose order is observable. Replace `forlist` with explicit index loops, and reproduce the
prefetch semantics only where a loop moves units (document each such loop). Tables become an owned
`Tables` value built from static defaults plus the ruleset's modify calls.
