# Quirks to preserve

Purpose: one list of bugs and odd behaviours that affect output. A byte-identical replica must copy
them. Fix none of them in the replica milestone; if one turns out to be undefined behaviour, establish
what the reference binary actually does and record the decision here.

Status column: **V** = confirmed by reading source for this note, **A** = from the analysis reports,
not re-read.

## RNG and ordering

| Quirk | Where | Status |
|---|---|---|
| 64-bit ISAAC with unaligned 8-byte reads | `i_std.h:41`, `i_rand.cpp:48` | V |
| `getrandom` modulo bias, negative ranges negate | `gameio.cpp:61` | V |
| Times file name drawn with retries while the file exists | `game.cpp:2130` | A |
| `forlist` prefetch: moving the prefetched element continues iteration in the other list | `alist.h:69` | A |
| Item order changes after combat (remove + append) | `items.cpp:1755` | A |
| Monster default moves draw before orders are read | `unit.cpp:716-834` | A |
| `DefaultWorkOrder` runs after each faction's order file | `game.cpp:77`, `game.cpp:1145` | A |
| `unordered_set<Unit*>` iteration names fortification in battle facts | `battle.cpp:449-476` | A (fidelity gap) |
| Practice counts once per unit per turn; first call wins | `unit.cpp:1384` | A |
| Lowest free unit number is reused | `game.cpp:1366` | A |

## Combat

| Quirk | Where | Status |
|---|---|---|
| `tactics_bonus` reset by `Army::Reset`, so it only applies in round 1 | `army.cpp`, `battle.cpp:508` | A |
| Shields accumulate across rounds (only FreeRound clears) | `battle.cpp:285` | A |
| Attack-level adjustments accumulate across attacks in `DoAnAttack` | `army.cpp:1334` | A |
| Amulet of invulnerability still counts as a hit in stats | `army.cpp:1513` | A |
| Typo `cruhsing`; `plural(att.killed, "hit", "hits")` | `battle.cpp:40+` | A |
| `FX_DONT_COMBINE` uses `results[dam]`; effects never increment `dam` | `specials.cpp:226` | A |
| Incomplete-ship spoils branch is dead; compares `getrandom(100)` to a 0..1 float | `battle.cpp` `GetSpoils` | A |
| Healing loop bound grows as soldiers heal | `DoHeal` | A |
| `GetAFacs` erases from the list it iterates | `battle.cpp` | A |

## Magic

| Quirk | Where | Status |
|---|---|---|
| Clear skies applies to the caster's region | `spells.cpp:1577` | A |
| `val` uninitialised for spells missing from the switch | `spells.cpp:682` | A |
| Transmutation `source` uninitialised for unlisted item types | `spells.cpp` | A |
| Random gate jump can loop forever | `spells.cpp:1710` | A |
| Teleport to ocean returns 1 (still practises) | `spells.cpp` | A |

## Economy and upkeep

| Quirk | Where | Status |
|---|---|---|
| Coastal race index can read `coastal_races[-1]` | `economy.cpp:91` | A |
| Entertainment `+1` in setup, `+10` in `SetIncome` | `economy.cpp:156`, `economy.cpp:317` | A |
| `improvement` not persisted; starts at 0 each turn | `aregion.cpp:135` | A |
| Taken-over start city leader price lacks ×10 | `economy.cpp:1544+` | A |
| `float space = 1/2` is 0 (migration disabled anyway) | `economy.cpp` | A |
| `TownHabitat` tests `IT_FOOD` on flags; may index `ItemDefs[-1]` | `economy.cpp` | A |
| `Short` loops to `i <= NITEMS`; break exits only object loop; negative `eat` adds items | `unit.cpp:1588` | A |
| Ally-borrow functions call `SetNum` twice | `runorders.cpp` maintenance | A |
| `ProcessMigration` returns immediately | `runorders.cpp:1212` | A |

## NPCs and world

| Quirk | Where | Status |
|---|---|---|
| `GrowWMons` bounds check `>` instead of `>=` | `npc.cpp:54` | A |
| `AdjustCityMon`: `mantype` uninitialised without men; `armor` may be -1 | `game.cpp:1891-2114` | A |
| Gateway destination not reassigned at candidate level 4 | `monthorders.cpp:1746` | A |
| `EnableItem(O_BOG)`, `EnableItem(O_ICECAVE)`, `O_ILAIR`, `O_GIANTCASTLE`, `O_DARKTOWER`, `O_MAGETOWER` pass object ids to the item table | `neworigins/extra.cpp:941-958` | V |
| `MakeUWMaze` loops `i <= NDIRS` (one past the neighbour array) | `neworigins/map.cpp:2517` | V |
| Standard `MakeShaft` builds the second shaft with `new Object(reg)` though it is added to `temp` | `standard/map.cpp:1066` | V |
| neworigins underworld `GetRegType` `case 11` falls through to ocean | `neworigins/world.cpp` | A |
| `simplex.cpp` and river shuffle seeded from `time(0)` | `simplex.cpp:89`, `aregion.cpp:2936` | V |

## Text and I/O

| Quirk | Where | Status |
|---|---|---|
| Order lines keep trailing `\r` | `fileio.cpp:225` | A |
| Unmatched quote wipes the rest of the line | `astring.cpp:181` | A |
| `_` equals space in all comparisons, including passwords | `astring.cpp:136` | A |
| `Trunc` writes `\0` without updating length | `astring.cpp:330` | V |
| Output files are refused if they already exist and are non-empty | `fileio.cpp:100` | A |
| Version string adds ` (beta)` for even minor numbers | `helper.h:39-42` | V |

## Porting notes

Each quirk gets a comment at its Rust site naming the C++ location and saying it is deliberate.
Unit tests pin the quirk where it can be isolated; fixtures pin the rest.
