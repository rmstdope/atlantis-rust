# at-1m2.1 — retrospective

- **Implementer:** Rogue
- **Date:** 2026-10-14
- **PR:** #20

## The fixture manifest validator rejected the new fixture family

**What happened.** `./scripts/gate-full.sh` failed at `replay_fixtures_match_sha256_manifest` after unit tests passed, reporting `unexpected file in fixture manifest: "fixtures/new-game/neworigins.rng"`. Extending the test to scan and allow `fixtures/new-game/` made the full gate pass.
**Why.** The manifest test's allowed path prefixes and inventory scan only covered `standard/turn_*` and `neworigins/turn_*`; the new oracle snapshots live outside those turn directories.
**Cost.** One full local gate run failed, followed by a test update and another full gate run.
**Prevent by.** When adding a fixture family outside `turn_*`, update both the path validation and inventory scan in `crates/atlantis-engine/tests/replay_harness.rs::replay_fixtures_match_sha256_manifest`.
**Seen before.** None found.
