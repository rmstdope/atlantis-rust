# Testing and fixtures in the original

Purpose: what verification assets the C++ repository has and which of them the Rust replica can
reuse.

## Snapshot tests (`snapshot-tests/`) — the oracle

- `run-snapshots.sh` runs `run-game-snapshots.sh` for `standard` (`turns/`) and `neworigins`
  (`neworigins_turns/`). Each has `turn_0` … `turn_13` (`turn` file contains `13`). Sizes: 13 MB and
  5.8 MB.
- Each `turn_N/` holds inputs `game.in`, `players.in`, `orders.3`, and expected outputs `game.out`,
  `players.out`, `report.1` (GM report, large), `report.3`, `template.3`, `engine-output.txt`
  (stdout), and in neworigins `times.<k>` files.
- The runner copies the inputs, runs `./<game> run`, moves outputs into `output/turn_N`, and compares
  with `diff -ur`. Byte-for-byte, including stdout.
- neworigins turns chain: each `turn_N+1/game.in` is byte-identical to `turn_N/game.out`.
  **Standard turns do not chain**: every standard `game.in` carries engine version `328196` (5.2.4),
  a different seed and other differences from the previous `game.out` (`328197`). The standard inputs
  were recorded with an older engine and only the outputs were regenerated. Every turn in both
  rulesets is still independently replayable, because the seed is in its own `game.in`.
- `turns/new-turn.sh` appends a turn: copies `game.out`, `players.out`, and `template.3` forward as
  inputs; you edit `orders.3`; it runs `../../standard/standard run`. Hard-coded to standard.
- One player faction (3) in both.
  - standard covers work, produce, sell, buy, tax, guard, study, move, give, form, claim, forget. No
    battles.
  - neworigins adds attack, `cast gate random`, `cast Phantasmal_entertainment`, teach, withdraw,
    behind. Battles appear in `neworigins_turns/turn_8` and `turn_13` `report.3`.
- Not covered: sailing, building, ships, most spells, stealing, assassination, transport, quests
  completing, monster lairs fights, pillage, exchange, many order errors.

## Unit tests (`unittest/`)

- Boost.UT 2.0.0 (`external/boost/ut.hpp`), `make unittest && ./unittest/unittest`. About 300 tests:
  16 `aregion_*` suites, plus `alist`, `faction`, `unit`, `landmark_order`.
- Built as a third ruleset (`unittest/rules.cpp` "UnitTest Atlantis", stub world). Not a rules
  oracle for standard or neworigins, but useful as a source of edge cases for geometry, region lists,
  reports, savegame and lifecycle.

## Smoke test (`smoketest/`)

Python 2 legacy (`smoketest.py`, `bot.py`, `checkpassword.py`); random orders; not in CI. Not reused.

## CI (`.github/workflows/build.yml`)

`compile` on ubuntu and windows (`make all`); `test` runs snapshots with the **ubuntu** binaries;
`unittest` under gdb (comment notes ISAAC misaligned loads as known UB).

## `review/`

`make_review.py` builds neworigins genrules HTML for two trees and highlights differences. Local aid;
the idea can be reused to compare Rust and C++ `genrules` output.

## What the replica reuses

1. All 28 snapshot turns as byte-exact fixtures, copied to `fixtures/<ruleset>/turn_N`.
2. The snapshot procedure itself, reimplemented as a Rust integration test, and runnable against the
   C++ binary to prove the harness.
3. New fixtures recorded per feature area with a patched reference build (RNG trace, `new` seed
   override), never hand-edited.
4. `report.1` (GM report) covers every item, object and skill description at all levels, which makes
   it a strong check of the effective tables and `skillshows`.

## Porting notes

Targets: `crates/atlantis-engine/tests/replay.rs` (fixture harness), `fixtures/`,
`tools/reference/` (build and record scripts), `scripts/gate-full.sh`.
