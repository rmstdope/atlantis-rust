# at-ljg.1 — retrospective

- **Implementer:** Cyclops
- **Date:** 2026-10-13
- **PR:** #1

## The producer skill was not available through the session skill loader

**What happened.** Calling the skill loader for `produce-bead` returned “Skill not found,” even though `.cerebro/cerebro/skills/produce-bead/SKILL.md` was present in the prepared worktree. I located and read the checked-in skill directly before beginning the design.
**Why.** The session's available skill list did not include this checked-in project skill; why it was omitted was not established.
**Cost.** Extra tool calls to locate and read the skill before implementation could start.
**Prevent by.** Ensure role skills declared by `agents/producer.md` are exposed in the session's available skill catalog, including `produce-bead`, before starting producer work.
**Seen before.** none found
