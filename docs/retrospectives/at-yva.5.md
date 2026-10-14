# at-yva.5 — retrospective

- **Implementer:** Cyclops
- **Date:** 2026-10-14
- **PR:** #18

## Serde derives do not cover the 256-word RNG arrays

**What happened.** The first `cargo test --locked -p atlantis-engine --test save_json` compile failed after deriving Serde for `Rng`: `[u64; 256]` did not implement `Serialize` or `Deserialize`.
**Why.** The locked Serde version does not provide those trait implementations for this fixed-array length.
**Cost.** One failed compile and an additional implementation and validation iteration; no CI cycles.
**Prevent by.** Check fixed-array support before deriving Serde for state arrays; `crates/atlantis-engine/src/rng.rs::rng_words` uses a length-checked adapter for the RNG buffers.
**Seen before.** None found.
