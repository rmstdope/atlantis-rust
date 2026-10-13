# Combat, stealing and assassination

Purpose: the battle algorithm with every random draw and ordering dependency, so the Rust `battle`
module reproduces outcomes and battle text exactly.

## Starting battles

- `DoAttackOrders` (`runorders.cpp:1417`) walks region → object → unit.
  - Wandering monsters: `CheckWMonAttack`, `rand = max(100, 300 - targets)`, attack if
    `getrandom(rand) < Hostile()`, target picked by weighted man count (`runorders.cpp:1411-1413`).
  - Other units resolve deduplicated ATTACK targets in order via `AttemptAttack` (alive, `CanSee`,
    `CanCatch`, `ONLY_ROUT_ONCE`).
- `DoAutoAttacks` (`runorders.cpp:1278`) attacks hostile units.
- Movement attacks: `DoMovementAttack(s)` (`runorders.cpp:1293-1368`) after each movement phase;
  `DoMoveEnter` fights units forbidding entry (`monthorders.cpp:1684`).

## Sides (`Game::RunBattle`, `battle.cpp:1098`; `GetSides`, `battle.cpp:882`)

- `GetDFacs`: target's faction plus allies (with `ALLIES_NOAID`). `GetAFacs`: attackers; removes
  entries from `attackorders->targets` while iterating them.
- `GetSides` walks the battle region and its six neighbours (`i = -1..NDIRS-1`). Building capacity =
  `ObjectDefs.protect`; neighbour buildings get capacity 0 unless `EXTENDED_FORT_DEFENCE`.
- City guards and guard mages defend only in the battle region and only if the attacker is not
  advancing. GUARD_AVOID units defend only if targeted, or same faction, in region and catchable.
  Noaid stops help from neighbours. Assassination places exactly the two units on dummy objects.

## Army (`Army::Army`, `army.cpp:673`)

- Tactics = highest "tactics" attribute; that tactician practises.
- Soldier array of `count` slots: units with `FLAG_BEHIND` fill from the back (`--y`), others from
  the front (`x++`), in unit order then item-list order. Assassination: only the first IT_MAN item,
  `count = 1`. If nobody is in front, the behind soldiers become the front.

## Soldier (`army.cpp:178`)

- Hits from toughness. Fleet protection converts the next ship into object protection when capacity
  runs out (`capacity = protect * ship->num`). Building bonus into `protection[]` with
  `ADVANCED_FORTS`, else into `dskill[]`. Runes raise energy and spirit defence. One capacity point
  per soldier.
- Monsters take `MonType` stats and return early (no healing, spells, items, armour, weapons).
- Men: `SetupHealing` (`specials.cpp:28`); `SetupSpell` (combat spell, practises); `SetupCombatItems`
  over `BattleItemDefs[1..]` with PREPARE, MAGEONLY, EXCLUSIVE, SPECIAL rules and shields raising
  `dskill`; armour (ready list, then `ArmorDefs[1..]`); mount if terrain allows (race-as-mount or
  `MountDefs[1..]`, bonus halved rounding up under `HALF_RIDING_BONUS`); weapon via `GetWeapon`
  (`unit.cpp:2386`): attack = skill + weapon bonus, attack count with NUM_ATTACKS_SKILL=100 and
  NUM_ATTACKS_HALF_SKILL=50, damage with NUM_DAMAGE_SKILL=1000 and NUM_DAMAGE_HALF_SKILL=500; without
  weapon attack = defence = combat + riding bonus. Every soldier `PracticeAttribute("combat")`.
- Default defences: combat 0, energy/spirit/weather -2, riding 0, ranged 0.
- Equipment is removed from the unit; `RestoreItems` puts it back. Removal and re-add reorders the
  unit's item list (see `quirks-to-preserve.md`).

## Soldier array partitions (`army.cpp:1134-1184`, `Kill` `army.cpp:1513`)

`[0,canfront)` front can attack; `[canfront,canbehind)` behind can attack; `[canbehind,notfront)`
front attacked; `[notfront,notbehind)` behind attacked; `[notbehind,count)` dead. `GetAttacker`,
`Kill`, `Reset`, healing and `GetTargetNum` move soldiers with specific swaps; random draws pick
indices, so the swap sequence must be copied exactly. `GetTargetNum` targets `NumFront()` or
`NumAlive()` when overwhelmed; collapses behind into front mid-round when the front is empty;
specials with target flags draw among valid targets, else `getrandom(tars)` mapped onto the two
front ranges.

## Running (`Battle::Run`, `battle.cpp:508`)

- Assassination: FreeRound for attacker.
- `ADVANCED_TACTICS`: tactics difference capped at 3 becomes `tactics_bonus`; `Army::Reset` zeroes it
  each round so it applies only in round 1. Without it, the higher-tactics side gets a FreeRound.
- Rounds `while !Broken && round < 101`. `Broken` (`army.cpp:1113`): fewer than half figures alive,
  or half hits in HITS rout modes. `IsArmyOverwhelmedBy` (`battle.cpp:136`) with `OVERWHELMING`.

## Round (`NormalRound`, `battle.cpp:285`)

`UpdateShields` for both sides (only FreeRound clears the shield list, so shields accumulate;
FX_DEFBONUS only at `round == 0`). Loop: `num = getrandom(aatt + batt)` picks side and attacker,
`GetAttacker` swaps it out, `DoAttack`. After: `Regenerate` per side (prints lines), loss lines,
verbose stats, `Reset`.

## Soldier turn (`DoAttack`, `battle.cpp:207`)

1. `DoSpecialAttack` (`specials.cpp:180`): per damage slot `times = value * (slevel if FX_USE_LEV)`,
   `realtimes = minnum + getrandom(times) + getrandom(times)`.
2. Mount special (front rank only), same formula.
3. Weapon attacks: negative counts = once every N rounds (`round % N == 1`); assassin attacks capped
   by `MAX_ASSASSIN_FREE_ATTACKS`; behind rank only ranged unless `canAttackFromBehind`.
4. `ClearOneTimeEffects`.

## Attack (`Army::DoAnAttack`, `army.cpp:1334`)

- Shields stop only ranged, energy, spirit, weather; `Hits(level, shieldskill)` must pass;
  non-damage effect spells remove the shield they pass.
- Per attack: pick target; defence level = `dskill[type]` (-2 with FX_NOBUILDING in building, 0 if
  NODEFENSE); melee length +1 to longer weapon; mount bonus vs mounted target; bonus/malus tables.
  These adjustments accumulate in `attackLevel` across attacks of the loop.
- Unless ALWAYSREADY, fails if `getrandom(2 + (protection+1)/2 if ADVANCED_FORTS)` is non-zero.
- `Hits(a, d)` (`army.cpp:1289`): `getrandom(tohit + tomiss) < tohit`, powers of 2 on the level
  difference, defender's `tactics_bonus`.
- Armour: `saves[class] > getrandom(from)`. Amulet of invulnerability ignores the hit but stats count
  it. Undead contagion: `getrandom(100) < UNDEATH_CONTAGION`.

## Aftermath

- Loser routs: winner gets a FreeRound. Loser: `Lose`, `GetSpoils`. Item loss uses
  `percent = dead/(alive+dead)` as float; drop `num2 = (num + getrandom(2)) / 2`; IT_ALWAYS_SPOIL and
  IT_NEVER_SPOIL override. Incomplete-ship branch is dead code (compares `getrandom(100)` to a 0..1
  float). Monster spoils (`GetMonSpoils`, `battle.cpp:817`): random silver and item type, reduced by
  `MONSTER_NO_SPOILS` / `MONSTER_SPOILS_RECOVERY`.
- Winner: `Win`; `DoHeal`: magical healing 5→1, healing potions at 70%, normal healing 5→1; loop bound
  `NumAlive()` grows as soldiers heal. Losses ≥5% give WIN_DEAD (no further attack, `nomove = 1`).
  Spoils dealt per item in spoils-list order to units that `CanGetSpoil`: weightless items in equal
  chunks, remainder one at a time with `getrandom(ns)`, ships to one random unit.
- Draw: `Tie` both sides. Then `KillDead`, raise-undead rolls, uncontrolled undead become a wandering
  monster unit (`battle.cpp:1053-1241`).
- Quests: kill targets checked in `GetSpoils`.

## Text

- `WriteSides` uses the best enemy observation. `WriteStats` (`battle.cpp:40`) for VERBOSE and
  DETAILED; keep the typo `cruhsing` and `plural(att.killed, "hit", "hits")` (kill count used).
- `FX_DONT_COMBINE` bug (`specials.cpp:226`): uses `results[dam]` not `results[i]`; effect entries
  never increment `dam`.
- Delivery (`battle.cpp:745`): after a successful assassination non-attackers see only the short
  text. Battles go to every faction on either side or present in the region.
- `AddBattleFact` (`battle.cpp:449-476`) iterates an `unordered_set<Unit*>` for the fortification
  name; ties depend on pointer hashing (NewOrigins Times only).

## Steal and assassinate (`runorders.cpp:249-422`)

Any faction that sees the thief and is the target's faction, an ally, or the guard faction gets the
caught message. Ring of invisibility vs target's AMTS blocks (`AmtsPreventCrime`; random under
`PROPORTIONAL_AMTS_USAGE`, `unit.cpp:1941`). Stealing silver takes half below 400, else 200.

## Porting notes

Target: `crates/atlantis-engine/src/battle/` (army, soldier, round, specials, shields, spoils,
text). Model the soldier array as a `Vec` with the five partition indices and port every swap
literally. `Soldier::effects` keyed by effect id is acceptable only if no iteration over it reaches
output (unverified; check during porting).
