# Luau fork rung 6 deferred rows

Diff: `6e9b580e..e8ae48c4` (official Luau 0.729 to 0.730)

No semantic ledger row is blocked or deferred in this rung.

The sole `Inliner/` hunk, `JitInliner::createInlinedProto`, removes trailing
whitespace only. The Rust fork does not translate that subsystem, but there is
no behavior to defer; the row is recorded as a verified no-op in
`rung6-noop.md`. No scoped `Require/` file changed in this diff.
