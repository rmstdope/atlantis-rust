# at-ljg.4 — retrospective

- **Implementer:** Cyclops
- **Date:** 2026-10-14
- **PR:** #7

## Review attributed criteria to a bead that had none

**What happened.** The cold review asserted that the bead required 100,000 mixed-range draws for at least 20 seeds and committed traces under `fixtures/rng/`. A fresh `bd show at-ljg.4 --json` returned `acceptance: null`, and neither the description nor design contained those requirements. I checked the source and posted a documented answer on PR #7.
**Why.** Not established; the reported criteria were not present in the bead or the reviewed plan.
**Cost.** One additional investigation and PR response, plus a retrospective commit and its CI cycle.
**Prevent by.** `.cerebro/cerebro/agents/reviewer.md`, “Does it do what it is meant to do?”: findings that say an acceptance criterion requires something should quote or cite that exact bead text; otherwise frame it as a test-coverage recommendation.
**Seen before.** None found.
