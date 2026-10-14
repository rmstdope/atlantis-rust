# C++ reference tools

These tools build and run an instrumented copy of the C++ oracle at `../Atlantis`. They never
modify that checkout. `build.sh` copies it to a fresh temporary directory, applies the patches,
builds both supported rulesets, and prints the temporary build root as its only stdout value:

```bash
reference_root=$(tools/reference/build.sh)
"$reference_root/Atlantis/standard/standard" dump-tables > standard.jsonl
```

The reference build remains in the printed temporary directory for inspection and reuse. Remove
that specific directory when it is no longer needed.

## RNG tracing and seed override

Run a game from its turn directory and set `ATLANTIS_RNG_TRACE` to a trace file:

```bash
cd fixtures/standard/<area>/turn_0
ATLANTIS_RNG_TRACE=/tmp/atlantis-rng.trace \
  "$reference_root/Atlantis/standard/standard" run
```

Each draw appends one `sequence range result` line. The sequence starts at zero for each process;
use a fresh or empty trace file when comparing a single run.

`ATLANTIS_SEED=<integer>` overrides only the seed used by `seedrandomrandom`, which is called by
`new`. A normal `run` reseeds from the saved game and is unaffected by this override:

```bash
ATLANTIS_SEED=424242 ATLANTIS_RNG_TRACE=/tmp/new-game.trace \
  "$reference_root/Atlantis/standard/standard" new
```

Invalid or out-of-range seed values fail explicitly.

## Effective rules tables

`<ruleset>/<ruleset> dump-tables` applies that ruleset's `ModifyTablesPerRuleset` changes and writes
JSON Lines to stdout without the engine banner. Each line is an object with a `table` field:

- `metadata` identifies the dump format version.
- `globals` gives each effective `GameDefs` member as `name` and `value`.
- `counts` gives each table name and its row count.
- Indexed rows cover items, races, monsters, weapons, armor, mounts, battle items, objects, skills,
  terrains, regular and magical healing, specials, effects, ranges, and attribute modifiers.

Rows and their fixed-size arrays use source table order and numeric enum values. Strings are JSON
escaped; null C++ strings are JSON `null`. Check both rulesets' output after building:

```bash
tools/reference/check-tables.py "$reference_root"
```

The check validates JSONL, determinism, all `GameDefs` members, table row counts, and the known
`SBOW` ruleset difference.

`standard/standard dump-item-families-default` writes just the item, race, monster, weapon, armor,
mount, and battle-item tables before any ruleset modifications. Its output is the baseline for
`fixtures/tables/item-families-default.jsonl`; compare it byte-for-byte when updating that fixture.

## Recording a feature-area turn

Seed `fixtures/<ruleset>/<area>/turn_N` with a complete reference fixture containing the inputs,
outputs, `engine-output.txt`, at least one `orders.N`, `template.N`, and `report.N`. Then run:

```bash
tools/reference/record-turn.sh standard <area>
```

The first call stages `turn_N+1`: it copies `game.out` to `game.in`, `players.out` to `players.in`,
and each `template.N` to `orders.N`. Edit one or more staged order files and run the same command
again. Unchanged orders are rejected. The second call builds both reference rulesets, runs the
selected one in the new turn directory, then replays the same inputs in a clean temporary directory.
It keeps the generated game, player, report, template, Times, and console outputs only when every
output matches byte-for-byte. Staging starts in a clean directory, so earlier `times.*` files cannot
change the reference engine's random draw count.

The command validates the ruleset, area, turn sequence, staged inputs, and order/template names.
It requires a complete prior fixture; importing or generating a baseline fixture is a separate
step. The Rust replay harness can then verify the recorded turn when it is available.
