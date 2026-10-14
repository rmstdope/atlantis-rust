# CLAUDE.md

Guidance for working in this repository. The fleet's roles live in `.claude/agents/` and
`.claude/skills/` (linked from `.cerebro/cerebro`); this file says what is true about this project.

## The project

A Rust rewrite of **Atlantis PBEM**, the play-by-email fantasy strategy engine whose C++ original
lives at `../Atlantis` (always assume that path). The first goal is a **replica**: the same rules
and the same behaviour as the original for exactly two rulesets, **standard** ("Wyreth") and
**neworigins**. "Working" means that replaying a recorded turn produces output **byte-identical**
to the C++ engine: `game.out`, `players.out`, every `report.N`, `template.N`, `times.N` and the
console output. The users are game masters running turns and, later, players and tools in a browser
via WebAssembly. The architecture is Rust-native, not a transliteration.

Decisions already taken with the navigator — do not re-open them without asking:

| Topic | Decision |
|---|---|
| Fidelity | Byte-identical replay of every fixture turn |
| Rulesets | standard and neworigins only; one `atlantis` binary, ruleset read from the save file |
| Interfaces | `atlantis-engine` (pure library) + `atlantis-cli`; `atlantis-wasm` later |
| Save formats | Legacy 5.2.5 positional format **and** a serde JSON format, auto-detected on load |
| `new` | Non-interactive: flags/config instead of the original's stdin prompts |
| `edit` | Deferred; later exposed as library operations |
| `genrules` | Part of the replica, byte-identical HTML |
| World generation | Structural + statistical fidelity (the original is time-seeded); byte-exact is a stretch goal |
| Fixtures | Extended per feature area by recording turns with the C++ reference |
| Licence | GPL-2.0-or-later, the original's terms: the replica is a translation of GPL code. Dependencies must be GPL-2.0-compatible (MIT, or MIT for dual MIT/Apache-2.0); see `NOTICE` |

Any **deviation** from the original's behaviour, file formats or command line goes to the
navigator first. Fixing an original bug is a deviation.

## Architecture

```
crates/
  atlantis-engine/   pure core: no std::fs, no stdin, no clock, no global state
    rng              LP64 ISAAC emulation (getrandom, seedrandom, make_roll)
    text             tokenizer, case/underscore-insensitive compare, report & rules writers
    defs             GameDefs, enums, static tables, modify API -> owned Tables
    ruleset          trait Ruleset; standard.rs, neworigins.rs
    model            Game, Faction, Region, Object, Unit, lists, Market, Production, Quest
    save             legacy positional reader/writer, serde JSON
    players          players.in / players.out
    orders           order model, parser, check mode
    turn             RunGame, PreProcessTurn, the RunOrders phases, month orders, maintenance
    battle magic npc economy report events world rules_html
    api              run_turn(TurnInput) -> TurnOutput, new_game, check_orders, view_map, gen_rules
  atlantis-cli/      `atlantis new|run|check|map|mapunits|genrules` with the original file names
  atlantis-wasm/     (later) wasm-bindgen wrapper over the api
fixtures/<ruleset>/[<area>/]turn_N/   recorded C++ turns: inputs + expected outputs
tools/reference/     builds a patched copy of ../Atlantis (RNG trace, seed override, table dump)
docs/reference/      analysis of the original: read the note for your area before porting it
scripts/gate-full.sh the gate (--fast for the fast gate)
```

State is held in id-indexed arenas with explicit ordered `Vec<Id>` membership where the original
used `AList`. Randomness is an explicit `&mut Rng` parameter. The turn API takes everything as
in-memory data (including the set of existing `times.*` names, which the original probes on disk)
so the same core runs in the CLI, tests and the browser.

## Fidelity rules

These exist because every one of them has silently broken byte-identity before in the original.

1. **Draw order is the contract.** Never add, remove or reorder a random draw. `getrandom` is called
   during default orders, combat, spells, buy/sell rounding, economy, maintenance, NPC growth,
   quests, Times composition and file naming, and the save-file seed.
2. **Iteration order is the contract.** Use ordered containers wherever the original walks a list.
   No `HashMap`/`HashSet` iteration may reach output. Reproduce `forlist` prefetch semantics where
   a loop moves or removes elements, and the item-list reorder that `SetNum(…, 0)` causes.
3. **Numbers behave like C++.** `f32` for `float`, `f64` for `double`, the same expression order,
   truncating integer division, explicit `wrapping_*` where C++ overflow is actually reached.
4. **Text is copied verbatim**, typos included ("cruhsing"). Reports wrap at 70 columns with
   two-space tabs; rules HTML at 78/70.
5. **The engine crate touches no filesystem, stdin or clock.** The gate checks for it.
6. **A fixture diff is read, never regenerated away.** Fixtures come from the C++ reference via
   `tools/reference`, never from the Rust engine and never by hand edit. This rule is rewritten
   when the reference is retired (see below).

Quirks and bugs of the original that must be preserved are listed in
`docs/reference/quirks-to-preserve.md`. Known places where the original itself is
non-deterministic are in `docs/reference/rng-and-determinism.md`.

## The C++ reference is temporary

Comparing against the original is a one-time check per area, not a permanent dependency.

- **The gate never builds or runs C++.** Everything the reference produces is committed as data:
  fixture turns, table dumps, RNG traces, `check` outputs, world-generation statistics. CI compares
  against those files only.
- `tools/reference` is a development tool for recording new fixtures and for tracing a mismatch.
  Nothing under `crates/` or `scripts/gate-full.sh` may call it.
- When every replica area is verified, a final bead removes `tools/reference`. From then on the
  fixtures are owned by the Rust engine: a change to them is a reviewed diff, and a deliberate
  departure from the original (approved by the navigator) is the only reason to accept one.

## Porting a piece of the original

1. Read the `docs/reference/` note for the area, then the C++ it cites in `../Atlantis`.
2. `../Atlantis/.claude/skills/atlantis-codebase/references/` holds the original project's own
   orientation notes; they are accurate except where `docs/reference/` says otherwise.
3. Write the Rust tests first: unit tests from the C++ logic, then a fixture replay.
4. On a replay mismatch, compare RNG traces (`--features rng-trace` against the patched reference)
   to find the first divergent draw before reading anything else.
5. Never modify `../Atlantis`. Patches for instrumentation live in `tools/reference/patches/`.

## Producer review

Nothing merges red. Before delivery, a producer obtains and addresses one independent, full review
of the complete diff and bead. If its changes are substantial enough to make another review useful,
the producer chooses the right follow-up scope and obtains it; minor, self-contained answers need
not create a review loop. Unresolved findings, a red or missing check, or a reviewer that cannot
produce a usable result go to a person. Whether the navigator also takes part in the plan, the
review or the merge is `navigator_gates` in `.cerebro/project.conf`; absent means none.

A review here also checks the fidelity rules above: a new or reordered random draw, an unordered
container reaching output, or a regenerated fixture is a blocking finding.

## Work tracking

Planned work is tracked in beads (`bd`), prefix `at`; `beads-workflow` carries the commands. GitHub
issues are the inbox for outside requests and bug reports only. Every bead is created unranked and
ranked later with the navigator; a bead is planned in one session and implemented in another.

The board syncs through the Dolt remote, not git. The opening plan is twelve epics (`bd list
--type epic`); each child bead names the C++ files to port, the reference note to read and the
fixture replay that proves it.

## Development practices

- Work is delivered in small increments that stand on their own.
- Code is written test-first.
- Prefer the simple design; say so when you decline a more general one.
- A ported function carries a doc comment naming its C++ origin (`// C++: runorders.cpp RunTaxOrders`)
  and explains any quirk it preserves on purpose.

## Commands

```bash
./scripts/gate-full.sh --fast        # fmt, clippy, unit tests, fast fixture subset
./scripts/gate-full.sh               # everything, incl. all fixtures and the no-fs check
cargo test -p atlantis-engine --locked replay      # fixture replay tests
tools/reference/build.sh             # build the patched C++ reference into a scratch dir (dev only)
tools/reference/record-turn.sh <ruleset> <area>    # record a new fixture turn
```

These scripts are created by the first epic (Foundations); until then the gate does not exist.
