# Reference notes on the C++ original

Condensed analysis of the C++ Atlantis engine at `../Atlantis`, written for implementers of the
Rust replica. Paths in citations are relative to `../Atlantis`. Line numbers were taken from the
checkout at commit `51d5fc0`; re-grep if they have drifted. The original repository also ships its
own orientation notes in `../Atlantis/.claude/skills/atlantis-codebase/references/`.

Read `rng-and-determinism.md` and `quirks-to-preserve.md` before porting any behaviour.

| Note | What it covers |
|---|---|
| [turn-pipeline.md](turn-pipeline.md) | `run` from `OpenGame` to `SaveGame`, the 32 `RunOrders` phases, where RNG is drawn |
| [data-model.md](data-model.md) | `Game`, `ARegion`, `Object`, `Unit`, `Faction`, lists, global tables, `GameDefs` |
| [savegame-format.md](savegame-format.md) | `game.in`/`game.out` positional layout, version gates, `players.in`/`players.out` |
| [orders-and-parsing.md](orders-and-parsing.md) | The 66 order verbs, order classes, tokenizer, order-file state machine, `check` |
| [reports-and-templates.md](reports-and-templates.md) | Report, template, GM report, Times, line wrapping, stdout |
| [combat.md](combat.md) | Battle setup, army/soldier arrays, rounds, hits, spoils, battle text, steal/assassinate |
| [magic-and-skills.md](magic-and-skills.md) | Skills, study/teach/practice, CAST, spells, teleport |
| [economy.md](economy.md) | Region economy setup and growth, markets, production, tax, trade, maintenance |
| [npc-and-quests.md](npc-and-quests.md) | City guards, wandering and lair monsters, monster control, quests, faction types |
| [movement.md](movement.md) | Phased movement, move points, sailing, fleets, gateways |
| [world-generation.md](world-generation.md) | `new`, `CreateWorld`, classic and natural generators, gates, shafts, names |
| [rulesets-diff.md](rulesets-diff.md) | Every difference between standard and neworigins, and the ruleset hook surface |
| [rng-and-determinism.md](rng-and-determinism.md) | LP64 ISAAC, seeding, draw sites, non-reproducible spots |
| [quirks-to-preserve.md](quirks-to-preserve.md) | Bugs and odd behaviour the replica must copy |
| [testing-and-fixtures.md](testing-and-fixtures.md) | Snapshot fixtures, unit tests, CI, what can be reused for verification |
