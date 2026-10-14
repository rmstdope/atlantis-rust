# at-pko.2 — retrospective

- **Implementer:** Cyclops
- **Date:** 2026-10-14
- **PR:** #9

## The purity gate rejects an enum name as a clock API

**What happened.** `./scripts/gate-full.sh` failed its engine-purity check on `DestroyBehavior::Instant` and the matching enum declaration after the focused tests and fast gate had passed.
**Why.** `scripts/gate-full.sh` searches for the bare substring `Instant`, so the C++-matching enum name was mistaken for `std::time::Instant`.
**Cost.** One full-gate run failed and required a rename plus another focused test and gate run; CI was not pushed while red.
**Prevent by.** In `crates/atlantis-engine/src/defs/enums.rs`, map C++ `INSTANT` to `Immediate` while the gate still searches for an unqualified `Instant` substring.
**Seen before.** none found
