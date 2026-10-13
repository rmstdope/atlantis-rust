# Traps

The facts this project has already paid for, read by producers before they start.
One entry per trap: what happened, what it cost, and what to do about it.

Seeded from the analysis of the C++ original (`docs/reference/`). Each would cost a full
fixture-replay mismatch hunt if rediscovered.

- **The original's RNG is not textbook ISAAC.** `ub4` is `unsigned long`, 64-bit on Linux/macOS,
  and the `ind()` macro reads 8 bytes at 4-byte-aligned offsets. The fixtures were made on Linux.
  Emulate u64 words and unaligned little-endian reads (`docs/reference/rng-and-determinism.md`).
- **`initIO` seeds 1783 before anything else**, then `OpenGame` reseeds from the save. Draws made
  before `OpenGame` (none today) would use the 1783 stream.
- **`times.N` file names cost random draws.** `WriteTimesArticle` draws `getrandom(10000)` and
  redraws while that file exists. The turn API takes the existing names as input.
- **`forlist` prefetches the next element.** Loops that move units between lists (movement,
  combat) visit or skip units because of it. Port each such loop deliberately, with a test.
- **Item order inside a unit changes after combat.** `ItemList::SetNum(t, 0)` removes the entry
  and re-adding appends. Report text shows the new order.
- **`AddBattleFact` iterates an `unordered_set<Unit*>`.** On ties the fortification name depends on
  pointer hashing: a known non-determinism of the original (neworigins Times only).
- **Saves are positional with conditional fields.** `gatemonth` only if gate > 0; production skill
  only for silver; skill `exp` only with `REQUIRED_EXPERIENCE`; `prevdir` gated by two globals;
  faction-type ints are 3 (standard) or 2 (neworigins).
- **`AString ==` ignores case and treats `_` as space** — for every lookup, including passwords.
