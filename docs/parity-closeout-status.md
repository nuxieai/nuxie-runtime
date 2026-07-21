# Parity Closeout Status

Live state for `docs/parity-closeout-map.md`. The next session must be able
to resume from this file alone. Keep it small: archive completed-ticket
logs the way `v2-status.md` / `renderer-status.md` did.

## Scorecard (update per session; `make parity-scorecard` once #B-4 lands)

| tier | state | number | notes |
|---|---|---|---|
| 1 Frame parity | PARTIAL | exact-segments 647/647; scripted 647/647; e2e-exact: gate not built | floor green; #OR-6 missing |
| 2 Interaction parity | RED | side-channel: gate not built; fuzz-clean-nights: 0 | #OR-1/2/3/7 |
| 3 SDK parity | RED | A-rows closed 0/8 | register A-table |
| 4 Platform parity | PARTIAL | pixel-exact 1468/1468; adapters 2/2; live same-runner 1468/1468 local | static byte-exact 837; live d788 M5 byte-exact 1370; Paravirtual rerun pending; #HD-2's hypothesis oracle and #HD-3 remain |
| 5 Performance & size | RED | ratio 0.897–0.914 (non-blocking, 6 files); size 7.19 MiB OFF / 7.95 MiB ON, budget pending | #OR-9, #B-3 USER-GATE |

Regression floor (must stay green): `make golden-compare` 317/647 ·
`make scripted-golden-compare` · `make renderer-golden` 1468 ·
`cargo test --workspace` · `make capi-smoke`.

Upstream pins: runtime `d788e8ec` (cycle-3 cut `b73bc675`, 3 commits ahead,
awaiting #B-1 approval). Upstream advanced after that completed inventory to
`ba2b6434`; it is next-cycle drift, not part of the pending authorization.
Renderer pixel-oracle `7c778d13` (historical, do not advance casually — see
upstream-sync-map registry).

## Ticket checklist

- [ ] #B-1 Phase S sync to b73bc675 — triage submitted; USER-GATE blocks port/pin movement
- [ ] #B-2 port-manifest invariant — implementation/local gate complete at
  exact b73bc675 (447/447: 378 ported / 21 partial / 43 absent / 5 N/A);
  first main CI green pending
- [ ] #B-3 size re-measure — measurement/audit complete; USER-GATE blocks the new budget
- [ ] #B-4 `make parity-scorecard` — implementation/local gate complete;
  canonical five-floor evidence and CI publication wired; first main CI green
  pending
- [ ] #OR-1 side-channel spec + C++ emit
- [ ] #OR-2 Rust emit + corpus-wide side-channel exact
- [ ] #OR-3 script verbs (setInput/VM-mutation/resize; key/textInput reserved)
- [ ] #OR-4 sampling densification (237 t=0-only entries)
- [ ] #OR-5 input-script coverage (all listener-bearing files)
- [ ] #OR-6 `make e2e-golden` blocking (≥50 files)
- [ ] #OR-7 differential fuzzer nightly (seeded-divergence proof, then 3 clean nights)
- [ ] #OR-8 tolerance root-cause (0.5 / 0.75 entries)
- [ ] #OR-9 blocking perf gate (≥20 files)
- [ ] #FT-ASSET asset loader seam (A1)
- [ ] #FT-TEXT text-input interaction (F2/F5/A3; blocked by #B-1)
- [ ] #FT-SCROLL scroll physics (F4)
- [ ] #FT-AUDIO audio (F1/A2; USER-GATE engine choice)
- [ ] #FT-CAPI portable surface (A4/A5/A7; 4 groups)
- [ ] #FT-PROD production-flow corpus lane (C3; USER-GATE access)
- [ ] #FT-FIXSWEEP typeKey fixtures (F10/C1/F9)
- [ ] #FT diagnostic spot-check: unported Lua binding → named diagnostic (from #LT-2)
- [ ] #HD-1 threading-model decision (USER-GATE)
- [ ] #HD-2 renderer oracle hardening (V7; adapter matrix 2/2 complete,
  current-runtime same-runner 1,468/1,468 local, Paravirtual CI rerun and
  clockwise-atomic hypothesis oracle pending)
- [ ] #HD-3 WebGL2 decision (USER-GATE)
- [ ] #HD-4 TODO(golden) pair
- [ ] #HD-5 publish the parity claim doc
- [ ] #LT-* long tail (each opens by USER-GATE)

## Next queue (top = next; orchestrator maintains)

1. #B-1 USER-GATE — approve named S3 rows or defer; no port or pin movement
   before the response.
2. #B-3 USER-GATE — choose the renderer-on size budget and whether it blocks
   on scripting OFF only or both variants.
3. #OR-1 — side-channel spec + C++ emit (spine; next executable work after
   the two submitted gates are answered).
4. #FT-TEXT — remains blocked on the approved/completed #B-1 TextInput port.

## Pending USER-GATEs

- **#B-1 Phase S cycle 3:** report
  `docs/sync/triage-2026-07-20-b73bc675.md` recommends approving S3-1
  (`1b4df2ad`, TextInput) and S3-3 (`b73bc675`, static library linking),
  deferring S3-2 (`079305d7`, profiler) as WATCH, and retaining both existing
  dependency WATCH rows at staleness 2. No active pin has moved.
- **#B-3 size budget:** exact renderer-on link closures are 7,534,056 B
  (7.19 MiB) scripting OFF and 8,335,288 B (7.95 MiB) scripting ON. Choose
  the blocking budget and whether it covers OFF only or both variants.

## Decisions log

- 2026-07-20: Project created from `docs/parity-gap-register.md` (six-way
  evidence sweep). Method/threading/routing inherited from
  `.claude/commands/goal.md` culture; session protocol at
  `.claude/commands/parity.md`.
- 2026-07-20: Cycle-3 approval cut remains fixed at `b73bc675`; post-inventory
  upstream `ba2b6434` is explicitly deferred to the next inventory rather than
  silently widening the pending authorization.
- 2026-07-20: The historical 2.75 MiB size budget is not reused: it measured a
  pre-renderer artifact. #B-3 remains open until the user chooses a new metric
  and budget.

## Log

- 2026-07-20 — #B-1 triage submitted for all 3 commits at `b73bc675`; pinned
  and candidate runtime ratchets localized exactly, and the session stopped
  before porting or pin movement.
- 2026-07-20 — Floor repair restored 317/647 default and scripted runtime
  exactness, migrated three stale atomic/tape renderer oracles without changing
  their contracts, and bound the adapter-dependent clippedcubic2 strict oracle
  to exact M5 Max / Apple Paravirtual references from the same historical `7c`
  revision, with provenance and a fail-closed revision-consistency check. The
  tape row's d788 static reference has a distinct path from the immutable 7c
  strict-atlas reference, so either capture workflow cannot overwrite the
  other's evidence.
- 2026-07-20 — Closeout hardening made the renderer negative control fail
  every active row: exact 0 / diverges 1,468. Ten 6×5 enum rows whose former
  32-pixel budget accepted every possible image tightened from 2/32 to 0/0;
  the real Rust/reference pairs remain byte-exact.
- 2026-07-20 — #B-2 landed the 447-row fail-closed port manifest; exact-b73
  verification reports 378 ported, 21 partial, 43 absent, and 5 N/A.
- 2026-07-20 — #B-3 remeasured the complete 42-root Darwin renderer surface
  twice with byte-identical artifacts: 7.19 MiB OFF, 7.95 MiB ON; independent
  source/root/symbol/hash audit clean; budget USER-GATE pending.
- 2026-07-20 — #B-4 landed the five-tier scorecard and bound each evidence file
  to its canonical command; unbuilt gates remain explicit rather than green.
- 2026-07-20 — #B-1 pin/candidate probing exposed stale golden-runner objects
  across `RIVE_RUNTIME_DIR` changes. The runner now rebuilds both translation
  units for every invocation, preventing upstream-header/library ABI mixing.
- 2026-07-20 — Adapter-selected static renderer references now participate in
  the same physical-alias and stream/frame/mode identity checks as primary
  references; adapter-bound stub oracles must be members of the approved set.
- 2026-07-20 — #HD-2's adapter ratchet advanced from 1/2 to 2/2 without a
  status or tolerance change. Eighty-two clockwise-atomic rows now select
  strict C++ Dawn references for Apple M5 Max and Apple Paravirtual device;
  the 28 legacy native-Metal rows were recaptured on M5 at renderer pin
  `7c778d13` / Dawn `211333b2`. Both static floors report 1,468/1,468 exact;
  the first main CI rerun is pending.
- 2026-07-20 — Main CI run `29806487036` kept the non-renderer required gates
  green but exposed nine same-runner rows: `tape` was a historical-7c versus
  product-d788 oracle skew, while the other eight localized to unconditional
  Metal `preserveInvariance` changing one-of-four edge samples on Apple
  Paravirtual. The gate now builds a separately pinned d788 C++ Dawn replay,
  including immutable dependency revisions and an exact-input cache, without
  relabeling the historical 7c oracle. Vendored wgpu now requests preserved
  invariance only for Naga-emitted invariant positions. Local M5 same-runner is
  1,468/1,468 contract-exact (1,370 byte-exact); the static floor is also
  1,468/1,468 with byte-exactness improved from 831 to 837. All five regression
  floors are green; the main Paravirtual rerun is pending.
