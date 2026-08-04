# In-house luaur fork (Luau scripting engine)

Decision (Levi, 2026-08-04): fork `luaur` — the pure-Rust Luau translation
that is Nuxie's scripting engine — and maintain the Luau engine port
in-house rather than waiting on upstream luaur releases. This document
records the fork point, the carried patches, and the port plan. It owns the
exit path for the standing WATCH `deferred-2026-07-19-luau-engine`
(docs/parity-gap-register.md): that row's exit criterion is now **fork
parity with the pinned C++ engine**, not "luaur publishes a newer base".

## Why

`deferred-2026-07-19-luau-engine` is the highest-urgency WATCH (rated 8/10,
3+ triage cycles stale). Upstream C++ runs Luau `rive_0_732` internals
(docs/sync/triage-2026-08-02-e0d4913f.md) while luaur 0.1.8 is based on
Luau 0.724-era sources, and luaur's release cadence is the bottleneck.
Owning the fork makes the engine bump our own port lane instead of an
external dependency.

## Fork point

The fork baseline is the complete luaur 0.1.8 crate family as published on
crates.io, vendored byte-for-byte under `vendor/`:

| crate | vendored at | modified? | provenance file |
|---|---|---|---|
| `luaur-ast` | `vendor/luaur-ast-0.1.8` | no | `NUXIE_PROVENANCE.md` |
| `luaur-bytecode` | `vendor/luaur-bytecode-0.1.8` | no | `NUXIE_PROVENANCE.md` |
| `luaur-common` | `vendor/luaur-common-0.1.8` | yes (Apple clock) | `NUXIE_PATCH.md` |
| `luaur-compiler` | `vendor/luaur-compiler-0.1.8` | no | `NUXIE_PROVENANCE.md` |
| `luaur-rt` | `vendor/luaur-rt-0.1.8` | no | `NUXIE_PROVENANCE.md` |
| `luaur-vm` | `vendor/luaur-vm-0.1.8` | yes (Apple clock) | `NUXIE_PATCH.md` |

- Upstream repository: `https://github.com/pjankiewicz/luaur`
- Upstream commit for all six packages:
  `f0eac7f7cce691d0cdb0b93c3eef9d599f71d739` (per each package's
  `.cargo_vcs_info.json`)
- Luau base: commit `8f33df91` of `luigi-rosso/luau` — the RIVE FORK's
  "Sync to upstream/release/724" commit. Note this tree is not official
  luau-lang 0.724: the fork line already carries the rive engine extensions
  (bytecode versions 7–11, `CALLFB`/call feedback, integer builtins,
  userdata-direct access, user-defined classes). luaur was translated
  against the fork tree, which is why those extensions exist in the Rust
  baseline. Verified by the 2026-08-04 baseline audit (see below).
- Original crates.io checksums are recorded in each vendored package's
  provenance file. `luaur-common` and `luaur-vm` predate this fork as
  vendored patched packages; the other four were added at the fork point,
  verified against the Cargo.lock checksums before extraction.
- `luaur-analysis` and `luaur-config` (optional `luaur-rt` dependencies)
  are not in Nuxie's dependency graph and are not vendored. If a future
  feature pulls them in, vendor them at the same fork point first.

## Workspace wiring

`[patch.crates-io]` in the workspace `Cargo.toml` redirects all six crate
names to the vendored packages; the consuming crates (`nuxie-scripting`,
`nuxie`) keep their `=0.1.8` crates.io-style pins. The vendored directories
are workspace-`exclude`d, same as the wgpu vendor packages. The switch is
behavior-preserving by construction: the four newly vendored packages are
the exact bytes cargo previously compiled from the registry cache, and the
Cargo.lock delta is only the source/checksum lines flipping from registry
to path for those four.

Verified at the fork switch (both green, no behavioral diff):

- `cargo test -p nuxie --features scripting`
- `make scripted-golden-compare` (byte-level side-channel differential
  against the pinned C++ scripted runner)

## Carried patches

1. **Apple Mach-clock widening** (`luaur-common`, `luaur-vm`; predates the
   fork). Widens the upstream Mach monotonic-clock branches from
   `target_os = "macos"` to `target_vendor = "apple"` so iOS device and
   simulator builds compile and keep macOS clock semantics. Files:
   `get_clock_timestamp.rs`/`get_clock_period.rs` (common),
   `clock_timestamp.rs`/`clock_period.rs` (vm). Details in each package's
   `NUXIE_PATCH.md`.

Every future engine change lands as a documented entry in the affected
package's `NUXIE_PATCH.md` (create one when a baseline package is first
modified) plus a row in this document, so fork-vs-crates.io drift stays
enumerable.

## Baseline audit findings (2026-08-04)

A full audit of the vendored translation against the `8f33df91` C tree
(codex read-only fan-out, adjudicated) resolved the standing "advertised
base lags" suspicion: **the base is confirmed** — bytecode constants,
all 89 opcodes, all 133 builtin IDs, userdata/typeinfo surface, and
compiler options match exactly. Six baseline TRANSLATION divergences are
carried (not version lag; all pre-date the fork and are gate-green today):

1. Every VM fast-call dispatch slot is `luauF_missing` — all fastcalls
   fall back to ordinary calls (semantically equivalent by Luau's design;
   a standing perf divergence; candidate post-ladder lane).
2. `LuauCompileDuptableConstantPack2` table-shape equality/hash baked ON;
   `LuauCompileNoOptNext` omitted (baked OFF); `LuauIntegerBufferFastcalls`
   conjunct baked ON.
3. `math.ldexp` constant folding uses `x * 2^exp` instead of `ldexp`
   (edge-case divergence, e.g. `ldexp(0, 2000)`).
4. Assertion handler's "do not trap" return value is ignored (Rust always
   traps).
5. `luaur-rt`'s effective flag profile: `set_all_flags(true)` with an
   explicit keep-OFF exception for `LuauExportValueSyntax`; also
   `FixMathNoisePrecision` ON (Luau CLI keeps it OFF).
6. `luaur-rt::Compiler` exposes a subset of engine `CompileOptions`.

## Oracle facts that bind the port

- The pinned rive C++ runtime (`ScriptingVM` in `src/lua/rive_lua_libs.cpp`)
  never touches Luau FFlags: the oracle runs the engine's **raw
  static defaults — every FFlag OFF**.
- The pinned oracle's engine is fork branch `rive_0_36`, which branches
  from the fork line at `81ac7c3c` — BEFORE the 0.724 sync. Corpus behavior
  stays invariant across engine versions because upstream ships new
  behavior dark behind FFlags; this is also why the exact-0.732 candidate
  runner completed the corpus (triage-2026-08-02).
- **Binding FFlag policy for rung ports:** new flags are translated into
  luaur-common's registry AND added to the keep-OFF exception set that
  `set_all_flags(true)` skips (the `LuauExportValueSyntax` mechanism), so
  Nuxie's effective profile tracks the oracle's flags-OFF profile. A flag
  is flipped ON only as its own change with gate evidence, recorded here.
  When upstream removes a flag and hardwires a path, the port removes the
  Rust flag and hardwires the same path.
- The pinned rive C++ does not compile Luau's `Require/` library (it
  registers its own `lua_require`); `Require/` deltas are out of the fork's
  port scope.

## Port plan: Luau 0.724 → 0.732 (rung ladder)

Upstream C++ HEAD is on Luau `rive_0_732` (final pin via S4-43 `395defdb`).
The fork advances one upstream release sync at a time — each rung lands as
its own PR with the full gate battery green (corpus exactness is the
ratchet floor at every rung):

| rung | range | scoped delta | status |
|---|---|---|---|
| 1 | `8f33df91..91caa731` (0.725) | 549+/209- | landed 2026-08-04: 75 rows ported, 29 no-op (C++-only scoping), 10 Require rows out of scope; new dark flags `LuauAutoStack`, `LuauCloneTableFix`, `LuauCustomYieldablePcalls`, `LuauUdataMetatablePinned` in the keep-OFF set; `DesugaredArrayTypeReferenceIsEmpty` removed (ON path hardwired) |
| 2 | `91caa731..86d2a9dc` (0.726) | 924+/212- | inventoried |
| 3 | `86d2a9dc..f1f121dc` (0.727) | 867+/626- | inventoried |
| 4 | `f1f121dc..ddcea05e` (0.728) | 395+/301- | inventoried |
| 5 | `ddcea05e..6e9b580e` (0.729) | 1596+/96- | inventoried |
| 6 | `6e9b580e..e8ae48c4` (0.730) | 1363+/547- | inventoried |
| 7 | `e8ae48c4..f8ca77ac` (0.731) | 2178+/566- | inventoried |
| 8 | `f8ca77ac..decb2d05` (0.732) | 1162+/627- | inventoried |
| 9 | `decb2d05..86eb0096` (rive_0_732 tip) | 304+/15- | inventoried |

Rung 9 is the rive patch set: vector fast functions on 3 components,
native `math.fround`, `LBC_VERSION_TARGET` held at 7 (which is why the
bytecode-v7 fixtures remain valid), unconditional `udatadirectfields`
init, buffer extension declarations, require-path behaviors, `RIVE_LUAU`.

Method per rung (established rung 1): codex read-only inventory (every
hunk → C symbol → Rust twin → FFlag posture → class), orchestrator
adjudication, then a codex writer lane in its own worktree — structure-
preserving, row-by-row commits, focused gates per row, rung-level
`cargo test -p nuxie --features scripting` + `make scripted-golden-compare`
+ land.sh battery.

Additional plan points:

1. **Known target symptom to retire:** `scope_probe` SIGTRAPs on
   `lua_pushcclosurek`'s stack assertion under newer-engine comparison
   (S4-3 carry-forward; C++ completes the stream while Rust traps —
   C++ release builds compile the assertion out and survive on stack
   slack). The upstream fix class is `LuauAutoStack` (rung 1, dark);
   retiring the SIGTRAP will be a recorded flag-enable change validated
   by scope_probe going exact, not a silent flip. Fork parity is not
   claimed until this reproduces green.
2. Oracles: the real bytecode-v7 fixture and the full forced-scripted
   ratchet remain the floor; the exact-0.732 C++ runner is the target
   comparison. The editor-emitted-bytecode compatibility matrix
   (upstream_ref `4ac7b327` discipline — read `port-manifest.toml`, not
   triage briefs, for the pin) is a valid interim evidence step.
3. Versioning: vendored crate versions stay `0.1.8` while the `=0.1.8`
   pins hold (a `[patch]` must satisfy the dependency requirement); the
   fork state is identified by this document plus per-package patch files,
   not by version bumps. Revisit the version scheme if the fork diverges
   enough that a distinct version communicates better.
4. When fork parity lands, update the `deferred-2026-07-19-luau-engine`
   WATCH row to CLOSED with the fork-parity evidence.

Out of scope for the fork-setup change that introduced this document: no
engine internals were ported; the vendor expansion and workspace switch are
provenance-only.
