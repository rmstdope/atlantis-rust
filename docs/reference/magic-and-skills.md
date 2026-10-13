# Magic and skills

Purpose: skill levels, study, teaching, practice, CAST orders, spell effects and teleportation, so the
Rust `magic` module and the study/teach parts of `turn` behave identically.

## Skill levels (`skills.cpp`)

- `GetLevelByDays(d)` (`skills.cpp:154`): level steps cost 30, 60, 90, ... days per man, so the
  cumulative thresholds are 30, 90, 180, 300, 450. `GetDaysByLevel(l)` (`skills.cpp:166`) is the
  inverse sum.
- `SkillMax` uses the race's special and default levels (`ManDefs`); `MANI` as a racial specialty
  covers all magic skills.
- `StudyRateAdjustment` (`skills.cpp:177+`) only acts when `REQUIRED_EXPERIENCE` is set; it is 0 in
  both rulesets.
- Skill descriptions for reports: `ShowSkill::Report` (`skillshows.cpp:87`), about 83 `case S_`
  blocks of hand-written text plus `DescribeEscapeParameters`.

## Study (`Do1StudyOrder`, `monthorders.cpp:1422`)

- Checks: cost, mage/apprentice/quartermaster limits (`Game::Allowed*`), the 1-man rule for leaders
  and mages.
- Rate = `GetStudyRate * men + taught`. Magic study above level 2 is halved outside a finished
  building, or when the building's mage slots are used up (`LIMITED_MAGES_PER_BUILDING`, NewOrigins).
  SLOWSTUDY halves it.
- A STUDY-to-level goal is re-queued through `turnorders`.

## Teach (`monthorders.cpp:426`)

30 × men × `STUDENTS_PER_TEACHER` days split across students in order, each capped at 30 × men. The
teacher practises.

## Practice (`unit.cpp:1384`)

- Once per unit per turn (`practiced` flag), so the first practice call of the turn wins.
- Amount = `SKILL_PRACTICE_AMOUNT` days per man: NewOrigins 5, standard 0 (no-op).
- Recurses into prerequisite skills; item-granted skills practise their source skill.

## CAST

- Parsing: `ProcessCastOrder` (`spells.cpp:45`) into cast or teleport orders.
- `RunCastOrders` (`runorders.cpp:135`) runs after TAX and GUARD1, after `ClearCastEffects`.
- `RunACastOrder` (`spells.cpp:682`): checks mage/apprentice and level, dispatches; on success
  `Practice` and `r->NotifySpell`. `val` is uninitialised for spells missing from the switch.
- Range: `GetRegionInRange` (`spells.cpp:856`) with `RangeType` classes and `GetPlanarDistance`.

## Spell behaviour (`spells.cpp`)

| Spell family | Rule |
|---|---|
| Summons | `(level * mOut + getrandom(100)) / 100`; imps, demons, eagles, wolves then `RandomiseSummonAmount` (2n coin flips, min 1) |
| Dragon lore | one roll `getrandom(100) < chance` |
| Construct gate | 1000 silver, success `level * 20`%; with `DISPERSE_GATE_NUMBERS` the number is random, rerolled while used |
| Engrave runes | 600 silver; runes 5 on magical buildings, else 3 |
| Earth lore | `Wages * level * 2 / 10` silver; sets `r->earthlore` |
| Phantasmal entertainment | capped by region entertainment |
| Clear skies | sets `r->clearskies` on the **caster's** region (`spells.cpp:1577`) |
| Transmutation | `source` uninitialised for item types not in the switch |
| Blasphemous ritual | random sacrifices, one Times article each (`spells.cpp:2094`) |

## Teleportation (`RunTeleportOrders`, `spells.cpp:2109`)

Runs after `ProcessEconomics`. For each object it repeatedly processes the first unit with a teleport
order.

Gate lore (`spells.cpp:1710`) weight limits:

| Level | Random jump | Chosen gate |
|---|---|---|
| 1 | 15 | 0 |
| 2 | 500 | 15 |
| 3 | 1500 | 500 |
| 4 | 3000 | 1500 |
| 5 | 6000 | 3000 |

"Random level" jumps use the next level down. A random jump loops on `regions.FindGate(-1)` until an
acceptable gate is found (can loop forever). Teleportation: limit `level * 50`; an ocean target
returns 1 so the skill still practises. Portal lore: limit `800 * level`, needs a friendly target mage
holding a portal.

## Porting notes

Target: `crates/atlantis-engine/src/magic/` (cast execution, teleport) and `src/turn/` for
study/teach/practice. Where the C++ reads an uninitialised value, establish what the reference binary
actually produces before choosing a value, and record it in `quirks-to-preserve.md`.
