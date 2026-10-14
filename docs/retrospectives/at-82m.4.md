# at-82m.4 — retrospective

- **Implementer:** Storm
- **Date:** 2026-10-14
- **PR:** #28

## Parser conflict resolution dropped existing handlers

**What happened.** While integrating the order parser change with newer main, the conflict resolution dropped existing dispatch for STUDY, TEACH, FORGET, ATTACK, EVICT, STEAL, ASSASSINATE, PROMOTE, DESTROY, ENTER, and LEAVE. An independent review caught the omission before merge. The handlers, their supporting ruleset-table cache and target parser, and a STUDY regression test were restored before the full gate was rerun.
**Why.** The resolution retained the new order handlers but did not compare the complete dispatcher and its helper set against the updated mainline.
**Cost.** One independent-review/fix cycle, restoration of the existing handlers and helpers, a regression test, and another full-gate run before delivery.
**Prevent by.** For parser conflicts, compare the complete dispatcher and helper dependencies against updated mainline as a unit before accepting the resolution, then run focused tests for preserved upstream handlers.
**Seen before.** None found; `at-pko.5` records a different add/add conflict in the definitions module root.
