# at-cnj.1 — retrospective

## Guards on unported phases passed turns that the C++ would have changed

**What happened:** The earlier attempt left an uncommitted draft that guarded each unported
`RunOrders` phase with a per-order predicate (FIND orders, attack orders, guard == Advance, items
> 0, ...). Five phases had guards that could never fire (`|_, _| false`). A Nexus-only test world
passed through all of them. The C++ would have drawn `getrandom(100)` in
`PostProcessTurn → AdjustCityMons(Nexus)`, set `visited`/`needed` on every unit in
`AssessMaintenance`, and acted in `ClearCastEffects`/`DoAutoAttacks`. The cold review then found
the same gap on neworigins: `WriteWorldEvents` and `CheckVictory` act in an empty world.

**Why:** The predicates followed each phase's order slots, not its C++ body. Most phases walk
every unit or region and change state beyond those orders.

**Cost:** One redesign of every guard and of the end-to-end test world (a world with no regions
replaced the Nexus-only one), plus one review round.

**Prevent by:** A `.cerebro/traps.md` entry: "An unported phase acts on more than its orders.
Read the C++ body before writing any skip or guard. `PostProcessTurn`, `AssessMaintenance`,
`ClearCastEffects`, `DoAutoAttacks` and neworigins `CheckVictory`/`WriteWorldEvents` act on
every turn or every unit."

**Seen before:** No.
