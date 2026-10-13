# Savegame and players files

Purpose: the exact on-disk layout of `game.in`/`game.out` and `players.in`/`players.out`, so the Rust
`save` and `players` modules can read and write them byte-identically.

## Low-level I/O (`fileio.cpp`)

- `Aoutfile::PutInt` / `PutStr` write the value then `'\n'` (`fileio.cpp:192-206`).
- `Ainfile::GetInt` is `file >> x`. `GetStr` skips all whitespace including blank lines, CR and NUL,
  then reads up to 1023 chars to end of line. Empty strings and leading whitespace cannot round-trip.
- `OpenByName` refuses to write a file that already exists and is non-empty (`tellp() != 0`,
  `fileio.cpp:100`). Reports behave the same way.

## `game.out` layout

Header (`Game::SaveGame`, `game.cpp:434-470`):

```
atlantis_game
<CURRENT_ATL_VER>       MAKE_ATL_VER(5,2,5) = 328197 (game.h:38)
<RULESET_NAME>          Wyreth | NewOrigins
<RULESET_VERSION>       131072 (2.0.0) | 196608 (3.0.0)
<year>
<month>
<seed>                  getrandom(10000), drawn here; next turn's seed
<factionseq>
<unitseq>
<shipseq>
<guardfaction>
<monfaction>
<#factions>  Faction records
WriteRegions
WriteQuests
```

Version macros (`helper.h:31-45`): `MAKE_ATL_VER(x,y,z) = (x<<16)+(y<<8)+z`. `ATL_VER_STRING`
appends ` (beta)` when the minor number is **even**, hence `5.2.5 (beta)`.

## Version gates on load (`game.cpp:341-396`)

- Engine major or minor mismatch: `Incompatible Engine versions!`, abort.
- Saved engine patch newer than the binary: abort. An **older** patch is accepted silently: every
  standard fixture `game.in` carries `328196` (5.2.4) and every `game.out` carries `328197`.
- Ruleset name must match (`AString==`, case-insensitive): else `Incompatible rule-set!`.
- Older ruleset version calls `UpgradeMajorVersion` (returns 0, abort), `UpgradeMinorVersion` and
  `UpgradePatchLevel` (return 1, no transformation).
- The `ATL_VER v` parameter passed to every `Readin` is unused.

## Faction record (`faction.cpp:179-202`)

`num`; one int per `FactionTypes` entry in vector order (3 in standard: war, trade, magic; 2 in
neworigins: martial, magic; NPC factions write -1); `lastchange`, `lastorders`, `unclaimed`; `name`,
`address`, `password`; `times`, `showunitattitudes`, `temformat`; `SkillList` (shown skills);
`ItemList` (discovered items); `defaultattitude`; `#attitudes` then `(faction, attitude)` pairs. On
read, an attitude toward the faction itself is dropped.

Example from `snapshot-tests/turns/turn_1/game.in`: `1 / -1 / -1 / -1 / -6 / 0 / 0 / The Guardsmen (1)
/ NoAddress / none / 1 / 0 / 2 / ...`.

## Regions (`ARegionList::WriteRegions`, `aregion.cpp:2012`)

`Num()`, `numLevels`; per level `x`, `y`, `name` or `none`, `levelType`; `numberofgates`; every region
record; the literal `Neighbors`; then 6 ints per region in list order (neighbour `num` or -1).

Region record (`aregion.cpp:977-1023`):
`name`, `num`, terrain `type` string or `NO_TERRAIN`, `buildingseq`, `gate`, **`gatemonth` only if
gate > 0**, race `abr` or `NO_RACE`, `population`, `basepopulation`, `wages`, `maxwages`, `wealth`,
`elevation`, `humidity`, `temperature`, `vegetation`, `culture`, `habitat`, `development`,
`maxdevelopment`, town flag 0/1 then `name`, `pop`, `hab` if 1, `xloc`, `yloc`, `zloc`, `visited`,
`ProductionList`, `MarketList`, `#objects` then Object records.

On read, `buildingseq` is recomputed from object numbers and `town->dev` is recomputed. Weather,
gate-open state, spell effects and `improvement` are not stored.

- Production: `abr`, `amount`, `baseamount`, skill `abbr` **only if item is `I_SILVER`**,
  `productivity`.
- Market: `type`, `abr`, `price`, `amount`, `minpop`, `maxpop`, `minamt`, `maxamt`, `baseprice`.

## Object record (`object.cpp:102-158`)

`num`; type name (fleet name if `IsFleet()`); `incomplete`; `name`; `describe` or `none`; `inner`;
`prevdir` (written as -1 unless `PREVENT_SAIL_THROUGH && !ALLOW_TRIVIAL_PORTAGE`); `runes`; `#units`
then Unit records; if fleet, `#ships` then Item lines. Units whose faction is missing are skipped on
read.

## Unit record (`unit.cpp:171-210`)

`name`; `describe` or `none`; `num`, `type`, faction num, `guard`, `reveal`, `free`; `readyItem` abr or
`NO_ITEM`; 4 × (`readyWeapon`, `readyArmor`); `flags`; `ItemList`; `SkillList`; `combat` abbr or
`NO_SKILL`; `savedmovement`, `savedmovedir`; `#visited` then strings in sorted order. On read, guard
ADVANCE becomes NONE and SET becomes GUARD.

## Lists

- `ItemList` (`items.cpp:1555-1582`): count, then `<num> [i]<ABR>` (prefix `i` = illusion). On read,
  items with unknown type, `num < 1`, or `DISABLED` are dropped.
- `SkillList` (`skills.cpp:230-446`): count, then `<ABBR> <days>[ <exp>]`; `exp` only when
  `REQUIRED_EXPERIENCE`. Skills with days 0 and exp 0 are dropped.
- Quests (`quests.cpp:80-215`): count, then per quest a type and type-dependent fields (SLAY,
  HARVEST, BUILD, VISIT, DEMOLISH, default), then reward Item lines.

Tables persist symbolically: item `abr`, skill `abbr`, object `name`, terrain `type`. Lookups use
case- and underscore-insensitive comparison.

## `players.in` / `players.out` (`game.cpp:486-1012`, `faction.cpp:697`)

Header:

```
AtlantisPlayerStatus
Version: <engine ver int>      major and minor must match
TurnNumber: <n>
GameStatus: New|Running|Finished
```

Input keys: `Faction: <n>` or `Faction: new [noleader] [x y z]`, `Name:`, `RewardTimes`, `Email:`,
`Password:`, `Battle:`, `Template:`, `Reward: <n>`, `SendTimes:`, `LastOrders:` (honoured only when
`LASTORDERS_MAINTAINED_BY_SCRIPTS`), `FirstTurn:`, `Loc: x y z`, `NewUnit: <alias>`,
`Item: <unit|gmN> <n> <item>`, `Skill: <unit|gmN> <skill> <days>`,
`Order: <unit|gmN> <order>` or `Order: quit`. Unknown lines are kept in `extraPlayers` and echoed.

Output per faction (`WriteFacInfo`): `Faction`, `Name`, `Email`, `Password`, `LastOrders`,
`FirstTurn`, `SendTimes`, `Template`, `Battle: na`, then pass-through lines.

## Porting notes

Target: `crates/atlantis-engine/src/save/` (legacy reader/writer and serde JSON) and
`src/players/`. Writers take `&Tables` and `&GameDefs` because conditional fields depend on them. A
round-trip test over every fixture `game.in` must reproduce the file except the seed line and the
engine-version line (see the 5.2.4 case above).
