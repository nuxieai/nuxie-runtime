# Rung 3 deferred rows

Range: `86d2a9dc..f1f121dc` (official Luau 0.726 → 0.727).

## deferred: runtime-inliner infrastructure

Rows J01–J23 are deferred. The vendored luaur fork has no translated `Inliner/` crate or module.

The new Inliner/ machinery is reachable only via flags our keep-OFF profile pins dark; it becomes a port obligation the day such a flag is enabled.

## deferred: untranslated Require subsystem

Rows Q01 `CyclicDependencyIndexError` and Q02 `CyclicDependencyNewIndexError` are deferred because the `Require/` subsystem is not translated in the vendored crates. Both C++ hunks remove unreachable `return 0` statements after non-returning `luaL_error` calls, so there is no current runtime behavior to mirror outside that absent subsystem.

## Dependency verification

The rung-2 prerequisites were verified before implementation:

- `CallInfo` already contained the `repr(C)` savedpc/errfunc union; rung 3's `Proto* p` was inserted immediately before it, matching `VM/src/lstate.h` at `f1f121dc`.
- The attribute-CST parser signatures were present.
- `Parser::parseAttributedFunction` was present in Rust; its rung-3 hunk is recorded as a no-op rather than a dependency mismatch.

No dependency mismatch blocked an in-scope rung-3 port row.
