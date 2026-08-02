Implemented the C08 extraction.

- Moved B6-0265 into [lua_data_value.rs](/Users/levi/dev/worktrees/nuxie-mr-c12/crates/nuxie-scripting/src/vm/lua_data_value.rs:1).
- Renamed the Promise implementation to `vm/lua_promise.rs`, removed the old path, and redirected importers.
- Recorded B6-0269/B6-0288 as justified shared-lifecycle exceptions in both the manifest and move plan.
- Preserved public APIs and behavior; focused DataValue test and `cargo check -p nuxie-scripting --tests` pass.
- No rows were queued.

The complete residue, validation, exception, and handoff details are in [INT-report.md](/Users/levi/dev/worktrees/nuxie-mr-c12/INT-report.md:1).

The batch could not be committed: the supplied checkout is `levi/mr2-c02`, `levi/mr2-c08` does not exist locally, and the sandbox blocks creating its Git ref/index lock. The exact subject is:

`[MR-2/C08] Extract B6-0265,B6-0268 from crates/nuxie-scripting/src/vm.rs`

Repository-wide gates remain blocked only by missing fixtures and pre-existing/foreign-root ledger-attribution residue; no reported failure is caused by C08.