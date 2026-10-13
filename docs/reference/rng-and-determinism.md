# RNG and determinism

Purpose: the exact random number generator the fixtures were produced with, every way randomness and
ordering enter the output, and the known non-reproducible spots. Read before porting any behaviour.

## Generator: ISAAC with 64-bit words (`i_rand.cpp`, `i_rand.h`, `i_std.h`)

- Bob Jenkins' ISAAC, `RANDSIZL = 8`, `RANDSIZ = 256`. Context: `randcnt`, `randrsl[256]`,
  `randmem[256]`, `randa`, `randb`, `randc`, all of type `ub4`.
- **`ub4` is `unsigned long int` (`i_std.h:41`)**: 64 bits on Linux and macOS (LP64), 32 bits on
  Windows. The fixtures were generated and are checked on `ubuntu-latest`
  (`.github/workflows/build.yml:39`), so the replica must reproduce the **64-bit** behaviour:
  - All state words and arithmetic are `u64` with wrapping. `mix()` shifts and the golden-ratio
    constant `0x9e3779b9` are not truncated to 32 bits. `a<<13` etc. keep the high bits.
  - `ind(mm, x)` (`i_rand.cpp:48`) = `*(ub4*)((ub1*)mm + (x & ((RANDSIZ-1)<<2)))`. The byte offset is
    `x & 0x3FC` (a multiple of 4 in 0..1020), and the read is **8 bytes**. So it reads the
    little-endian `u64` made of bytes `off .. off+8` of the `randmem` array viewed as bytes, which
    straddles two 64-bit words when `off % 8 == 4`, and only ever touches the first 1 KiB (words
    0..128) of `randmem`.
  - `rngstep` (`i_rand.cpp:49-55`): `x = *m; a = (a ^ mix) + *(m2++); *(m++) = y = ind(mm,x) + a + b;
    *(r++) = b = ind(mm, y >> RANDSIZL) + x;` with mixes `a<<13`, `a>>6`, `a<<2`, `a>>16`.
  - `isaac` (`i_rand.cpp:57-79`): `b = randb + (++randc)`; two half-loops as in reference ISAAC.
  - `randinit(ctx, TRUE)` (`i_rand.cpp:94-142`): a..h = `0x9e3779b9`, 4 scramble `mix` rounds, two
    seeding passes over `randrsl` then `randmem`, then `isaac(ctx)`, `randcnt = 256`.
- `isaac_rand(r)` (`i_rand.h:70-74`): `!(randcnt--) ? (isaac(r), randcnt = 255, randrsl[255]) :
  randrsl[randcnt]`. Results are consumed from index 255 down to 0. (Post-decrement: the first call
  after init returns `randrsl[255]`.)
- The unaligned 8-byte loads are undefined behaviour in C++; the unit-test CI comment calls them
  "benign-on-x86". The replica emulates the x86-64 little-endian result. Verify against traces from
  the reference binary built on the same platform family.

## API (`gameio.cpp`)

- `seedrandom(num)` (`gameio.cpp:74`): `randa = randb = randc = 0`; `randrsl[i] = (ub4)num + i` for
  i in 0..256; `randinit(ctx, TRUE)`.
- `getrandom(range)` (`gameio.cpp:61`): 0 if `range == 0`; else `i = isaac_rand() % |range|` computed
  in 64-bit unsigned, negated if `range < 0`, cast to `int`. Modulo bias is part of the contract.
- `makeRoll(rolls, sides)`: sum of `getrandom(sides) + 1`.
- Seeds: `initIO()` → 1783 (`gameio.cpp:54`); `OpenGame` → saved seed (`game.cpp:400`); `NewGame` →
  `time(0)` (`seedrandomrandom`, `game.cpp:290`). `SaveGame` stores `getrandom(10000)` (0..9999).

## Where draws happen during `run`

In order (see `turn-pipeline.md` for line references): monster default orders in `PreProcessTurn`;
`ReadPlayers` (only with multi-hex nexus); combat and battle facts; steal/AMTS; pillage; spells;
sell/buy contested rounding; `MonsterCheck`; movement attacks and gateways; month orders (harvest
quests); `Grow`; teleport; maintenance starvation; `CheckVictory` (neworigins quests, Times);
`ARegion::PostTurn`; `AdjustCityMons`; `GrowWMons`/`GrowLMons`/`GrowVMons`; `Events::Write`;
`WriteTimesArticle` naming; finally the seed draw in `SaveGame`.

Draw counts by file (root and rulesets): economy 59, aregion 45, npc 28, army 16, spells 16, battle 8,
game 8, unit 6, edit 6, events-battle 5, events 4, runorders 4, monthorders 1, production 1, quests
1, specials 1; neworigins map 82, extra 19, world 16; standard map 43, world 11, extra 1.

## Filesystem-dependent draws

`WriteTimesArticle` (`game.cpp:2130`) draws `getrandom(10000)` for the file name `times.<k>` and
redraws while that file exists in the working directory. The number of draws therefore depends on
which `times.*` files already exist. The replica API must take the set of existing names as input.

## Ordering that changes output

- Every list iteration order (see `data-model.md`, `forlist`).
- `ItemList::SetNum(t, 0)` removes the element and re-adding appends (`items.cpp:1755`); combat strips
  and restores equipment, so item order inside a unit changes after a battle and shows in reports.
- `std::map` / `std::set` iterate in key order (`Unit::visited`, quest maps).
- `std::unordered_*` iteration: `Faction::activity` (verify no output-affecting iteration);
  `AddBattleFact` (`battle.cpp:449-476`) iterates `unordered_set<Unit*>` to name the fortification;
  on equal protection values the result depends on pointer hashing. This is a known fidelity gap
  (neworigins Times only).

## Non-ISAAC randomness

Only in world generation: `aregion.cpp:2936-2977` and `simplex.cpp:89-92` use
`std::default_random_engine` seeded with `time(0)` (natural generator only). `neworigins/map.cpp:1064`
(`random_device` + `mt19937`) is in dead code.

## Floating point

`Market::PostTurn`, `SetupEconomy`, `Grow`, `TownGrowth`, `RunAProduction`, battle item-loss
`percent` use `float`/`double` mixed with integer division. IEEE-754 results are reproducible in Rust
with `f32`/`f64` and the same operation order and casts (x86-64 SSE, no x87 extended precision, is
assumed; unverified for the fixture build flags `-g` without `-O`).

## Porting notes

Target: `crates/atlantis-engine/src/rng.rs`. Implement `Isaac64Quirk` with state `[u64; 256]` and
`ind` reading little-endian bytes from a byte view of `randmem`. Provide a `rng-trace` feature that
logs `(seq, range, result)` and compare with the patched reference. Pass `&mut Rng` explicitly; no
global state, so the engine stays usable from WASM.
