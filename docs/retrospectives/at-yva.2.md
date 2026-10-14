# at-yva.2 — retrospective

- **Implementer:** Cyclops
- **Date:** 2026-10-14
- **PR:** #15

## The rebased review range included upstream code

**What happened.** After rebasing PR #15 onto `main`, the follow-up review compared the previous reviewed commit with the rebased head. That range included `players.rs` changes already merged to `main` in PR #16, so the reviewer flagged out-of-scope work. Comparing the actual PR diff against its `main` base showed those files were not part of PR #15; a full review of the actual PR diff confirmed the scope.
**Why.** A commit-to-commit range across a rebase includes commits newly present in the base branch, even when the pull request diff does not.
**Cost.** One additional full review round and several minutes to distinguish upstream changes from this bead's diff; no additional CI cycle.
**Prevent by.** Before a follow-up review after rebase, check the current PR base OID and inspect the PR diff against that base. Tell the reviewer which files in the old-head-to-new-head range are already in the base, and judge scope from the actual PR diff.
**Seen before.** None found.
