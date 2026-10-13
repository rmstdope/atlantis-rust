# at-ljg.2 — retrospective

- **Implementer:** Cyclops
- **Date:** 2026-10-14
- **PR:** #4

## `git diff --check` flags byte-exact fixture whitespace

**What happened.** Running `git diff --cached --check` across the new fixtures exited nonzero on trailing spaces and blank final lines in recorded output files. Direct comparison confirmed the fixture bytes match the C++ snapshot source; normalizing them would have broken the byte-identity requirement.
**Why.** The snapshots preserve C++ output whitespace that Git's whitespace checker reports.
**Cost.** One plan amendment and a scoped whitespace check; no fixture content was changed.
**Prevent by.** In fixture-changing bead designs, scope whitespace checks to implementation, documentation, and metadata paths, excluding byte-exact fixture outputs.
**Seen before.** none found.
