# at-pko.5 — retrospective

- **Implementer:** Storm
- **Date:** 2026-10-14
- **PR:** #10

## The parallel table port added the same module root

**What happened.** `git rebase origin/main` stopped with an add/add conflict in `crates/atlantis-engine/src/defs/mod.rs`. This branch and the newly merged `at-pko.2` both added that file; the conflict was resolved by retaining both sets of module declarations and exports.
**Why.** `at-pko.2` merged after this branch was created and established the shared `defs` module root.
**Cost.** One manual conflict resolution and reruns of the fast and full gates before pushing the rebased branch.
**Prevent by.** Merge the shared `defs/mod.rs` scaffold before independent table-port children begin, or defer module-root registration to a single integration change after the child modules exist.
**Seen before.** None found.
