# at-1m2.4 — retrospective

- **Implementer:** Storm
- **Date:** 2026-10-15
- **PR:** #32

## The 50-seed generation test took over nine minutes before endpoint searches were bounded

**What happened.** `natural_statistics_match_cpp_reference_over_50_seeds` took 560.49 seconds before the river endpoint search was bounded. The full gate repeated that workload. A focused follow-up review identified that `best_river_endpoints` ran full-graph Dijkstra for every source-region candidate, even after a nearest eligible endpoint was reachable.

**Why.** The port computed distances over the full map for each source candidate and each selected water-body connection, although the result only needed the cheapest positive-cost target endpoint.

**Cost.** A single acceptance test took about 9 minutes 20 seconds before the optimization. After stopping at the nearest eligible endpoint while preserving ties, the same 50-seed test took 189.89 seconds and still matched the C++ statistics.

**Prevent by.** When adding repeated fixed-seed world-generation checks, profile the complete acceptance test before the full gate and inspect full-graph searches nested under source/target loops. Keep a differential nearest-endpoint test in `crates/atlantis-engine/src/world/natural/rivers.rs` to ensure pruning preserves endpoint selection.

**Seen before.** none found
