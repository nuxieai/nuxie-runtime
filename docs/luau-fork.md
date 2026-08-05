# In-house luaur fork (Luau scripting engine)

Decision (Levi, 2026-08-04): fork `luaur` — the pure-Rust Luau translation
that is Nuxie's scripting engine — and maintain the Luau engine port
in-house rather than waiting on upstream luaur releases. This document
records the fork point, the carried patches, and the port plan. It owns the
exit path for the standing WATCH `deferred-2026-07-19-luau-engine`
(docs/parity-gap-register.md): that row's exit criterion is **fork parity
with the pinned C++ engine**, not "luaur publishes a newer base".
**STATUS 2026-08-05: ladder complete — all nine rungs landed, WATCH CLOSED.**
The vendored engine now carries every upstream delta from its 0.724-era
base through official 0.732 plus the rive patch set at `rive_0_732`
(`86eb0096`), which is what the pinned C++ runtime embeds.

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
2. `LuauCompileNoOptNext` omitted (baked OFF); `LuauIntegerBufferFastcalls`
   conjunct baked ON. (A third item — `LuauCompileDuptableConstantPack2`
   shape equality/hash baked ON — was retired at rung 2 when upstream
   removed the flag and hardwired the ON path.)
3. `math.ldexp` constant folding uses `x * 2^exp` instead of `ldexp`
   (edge-case divergence, e.g. `ldexp(0, 2000)`).
4. Assertion handler's "do not trap" return value is ignored (Rust always
   traps).
5. `luaur-rt`'s effective flag profile: `set_all_flags(true)` with an
   explicit keep-OFF exception for `LuauExportValueSyntax`; also
   `FixMathNoisePrecision` ON (Luau CLI keeps it OFF).
6. `luaur-rt::Compiler` exposes a subset of engine `CompileOptions`.
7. `&str`-based error formatting (rung-5 audit finding, adjudicated
   2026-08-04; NARROWED at rung 9): luaur's error layer takes `&str`, so
   non-UTF-8 bytes in a userdata `__type` name are lossy-replaced (U+FFFD)
   in error messages where C passes raw bytes — this still affects
   `luaL_checkudata`/`luaL_checkudatatagged` type names. Rung 9 gave
   `luaL_where` and `pusherror` byte-exact paths (explicit lengths through
   `lua_pushlstring`), so chunk/source bytes in error prefixes are now
   faithful. Unreachable for Nuxie's ASCII type registrations; the
   scripted side-channel referees any corpus-visible divergence.

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
| 2 | `91caa731..86d2a9dc` (0.726) | 924+/212- | landed 2026-08-04: 68 rows ported, 3 no-op; upstream removed `LuauCompileDuptableConstantPack2` (packed table constants + `LBC_VERSION_TARGET` 7 now unconditional — retires the baked-ON parts of baseline divergence 2); new dark flags `LuauCstAttr`, `LuauVirtualBcBuilder` in the keep-OFF set |
| 3 | `86d2a9dc..f1f121dc` (0.727) | 867+/626- | landed 2026-08-04: 116 rows ported, 5 no-op, 25 deferred (Inliner/ JIT infra unreachable under keep-OFF profile; Require untranslated); layout changes Closure(-usage)/CallInfo(+p)/Proto(+optimized,+deoptimized); seven flag removals hardwired; new dark flags incl. `LuauCIProto`, `LuauPromoteProto`, `LuauGcTableStepFix` |
| 4 | `f1f121dc..ddcea05e` (0.728) | 395+/301- | landed 2026-08-04: 57 rows ported, 9 no-op; three flag removals hardwired ON (`LuauConstJustReportErrorForUnderfill`, `LuauCstExprGroup`, `LuauErrorTolerantPrettyPrinting`); bytecode-graph SSA maturation (reverse def-use, sealed SSA, inliner phi anchoring, cyclic-phi serialization) |
| 5 | `ddcea05e..6e9b580e` (0.729) | 1596+/96- | landed 2026-08-04: bytecode v12 unit (dark emission + unconditional loader), Proto::cost, SCCP + DenseHash2 foundation, resume-ccalls hardwiring, direct-field GC remarking; audit REJECT adjudicated to documented divergence 7 (&str error layer) |
| 6 | `6e9b580e..e8ae48c4` (0.730) | 1363+/547- | landed 2026-08-04: unsigned-class cluster, bytecode target 9, mutation-tracker unification, SCCP evaluator/driver; dark flags `LuauMathRoundNegZero`, `LuauGcMarkUdataAccess`; dormant arithToK MOD/POW divergence recorded in luaur-bytecode NUXIE_PATCH.md for re-audit |
| 7 | `e8ae48c4..f8ca77ac` (0.731) | 2178+/566- | landed 2026-08-04: double-vector representation foundation (VECTORD tag, vectorPrecision, allocator/lvector, caller sweep), class hoisting, dark `LuauCompileIifeInline`/`LuauBytecodeFold`/`LuauXpcallFixMessageYieldPath`/`LuauBackedgeHeapCheck`, memorydump/allocationrate wrappers |
| 8 | `f8ca77ac..decb2d05` (0.732) | 1162+/627- | landed 2026-08-04: class inheritance (LOP_NEWCLASS/super/luaR_inheritclass), custom-pcall retirement (rung-1 unit now unconditional), CstAttr retirement, dark v13 double-vector constants / export-table optimization / managed debug names; audit-driven fix: class-shape decode mirrors C's resize+append doubled layout |
| 9 | `decb2d05..86eb0096` (rive_0_732 tip) | 304+/15- | landed 2026-08-05: ALL 28 rows ported, 0 deferred — Rive builtin ABI (LBF_RIVE_FROUND=243, Vector block 245-255), fastcall table wired, math.fround (native + library), Rive vector fast functions, lua_pushvector2 (stale-z quirk faithful), RIVE_LUAU baked ON (no print/newproxy/writestring; rive luaL_where/pusherror), LBC_VERSION_TARGET 7, unconditional lexeme capture |

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

1. **`scope_probe` SIGTRAP — resolved before the ladder, not by it
   (verified 2026-08-05).** The 2026-08-02 triage recorded Rust trapping
   on `lua_pushcclosurek`'s stack assertion for this fixture. Direct
   probe of the completed fork (`rust-golden-runner-scripted --file
   fixtures/sync/scope_probe.riv --execute-scripts --samples 0.0,0.5,1.0`)
   exits 0 with a complete 33-line stream — but so does the SAME probe at
   the pre-ladder fork baseline (`2cb99385`). The symptom's precondition
   was already gone: the S4-3 port (`76cb108a`, 2026-08-02, "Statically
   link library requires") landed two days before the fork baseline. The
   ladder therefore does NOT claim credit for retiring it; the honest
   record is that the trap does not reproduce at either endpoint, and
   `scope_probe` is corpus-`exact` under the standing gates. The upstream
   fix class `LuauAutoStack` remains ported-but-dark (rung 1); enabling it
   stays a separate recorded change with its own gate evidence.
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

## Ladder completion record (2026-08-05)

All nine rungs landed, each as its own PR with the full gate battery green
(corpus exactness held at every rung) and each preceded by an adversarial
read-only audit bound to the C diff:

| rung | PR | audit |
|---|---|---|
| baseline (vendor + workspace switch) | #247 | — |
| ladder record | #250 | — |
| 1 — 0.725 | #251 | ACCEPT, 0 findings |
| 2 — 0.726 | #259 | ACCEPT, 0 findings |
| 3 — 0.727 | #262 | ACCEPT, 0 findings |
| 4 — 0.728 | #265 | ACCEPT, 0 findings |
| 5 — 0.729 | #266 | 1 CONFIRMED → adjudicated to carried divergence 7 |
| 6 — 0.730 | #269 | ACCEPT; dormant `arithToK` divergence recorded for re-audit |
| 7 — 0.731 | #282 | ACCEPT, 0 findings |
| 8 — 0.732 | #289 | 2 CONFIRMED → stale stack rebased; class-shape decode fixed to mirror C |
| 9 — rive_0_732 patch set | #290 | ACCEPT, 0 findings |

## Editor-emitted bytecode compatibility matrix

Each row is a real Nuxie Editor artifact: a small `.riv` materialized
through the production scene publish path (`editor-publisher-wasm`
`publish()`), with the script's compiled bytecode SHA-256 recorded BEFORE
materialization. Row acceptance: the ScriptAsset bytes re-extracted from
the fixture (behind the `0x00` unsigned `SignedContentHeader` byte)
hash-match the recorded compiler output, and the script's effect is
observable in render. The harness is the scripted golden runner pair
(corpus id below) plus the focused acceptance tests in
`crates/nuxie/tests/editor_bytecode_matrix.rs`.

Corpus disposition: ordinary lane `exact` (script inert, both sides draw
the same background), scripted lane `scripted-status:diverges` — the
pinned C++ oracle only registers Rive-signed script modules
(`ScriptAsset doesn't have a generator function`), while the Rust runner
executes unsigned editor bytecode the way the device SDK does behind
`allowsUnverifiedScripts` / `import_with_unsigned_scripts`. The scripted
render-observability proof therefore lives in the Rust-side acceptance
test, not in a cross-runner match.

| row | emitter | emitter commit | compiler FFlag posture | LBC | bytecode sha256 | fixture | corpus id |
|---|---|---|---|---|---|---|---|
| current Nuxie Editor v7 (2026-08-05) | `scripted-resource-compiler` via `editor-publisher-wasm` `publish_json` (nuxie-dev; compiler activated at `8562bc6687`) | nuxie-dev `4a63abca` | all Luau FFlags forced OFF during compile (`DeviceLuauFlagsGuard`), flags-off floor v7 | 7 | `50d69e465eb4413f342a38b1c6c3dbb71531559c98d0a0514a0d2b782ed477bd` | `fixtures/editor/editor_scripted_vector_v7.riv` (sha256 `9a2affb093890685c39f3172de93dd2dad242d35eda21fe2783b177c191d0b24`) | `editor_scripted_vector_v7` |

The v7 row's source is the e4 `scripted-vector.luau` fixture (draws a
path with `color=0xff7f33cc` at translation `(24,18)` — the render
observability oracle). The recorded bytecode hash is the same one the
compiler crate's own contract test pins, so editor-side compiler drift
breaks both repos' gates at once. Regenerate a row by publishing the e4
scripted-vector snapshot through `publish_json` with the bytecode
live-compiled by `compile_luau_bytecode` (script only, no shader
mutation) and re-recording commit + hashes.

Method (reusable for the next engine bump): per rung, a read-only inventory
maps every C hunk to its Rust twin with FFlag posture and scope class; the
orchestrator adjudicates; a writer lane ports row-by-row with per-row focused
gates in its own worktree; an adversarial auditor re-derives behavior from
the C diff; then land.sh. Ledgers and per-rung disposition reports live in
the lane worktrees' untracked `.luau-fork-work/`.

**Standing follow-ups** (none block parity):

- Fast-call dispatch is still entirely `luauF_missing` outside the rive block
  (carried divergence 1) — a perf-only lane, semantically equivalent by
  Luau's fallback design.
- `arithToK` MOD/POW const-lhs: Rust skips where C crashes; unreachable today
  (no `foldConstants` caller in either tree). Re-audit when a caller lands
  (`vendor/luaur-bytecode-0.1.8/NUXIE_PATCH.md`).
- Dark flags ported but pinned OFF track the oracle's flags-OFF profile.
  Enabling any one is its own change with gate evidence.
