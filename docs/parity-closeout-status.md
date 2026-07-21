# Parity Closeout Status

Live state for `docs/parity-closeout-map.md`. The next session must be able
to resume from this file alone. Keep it small: archive completed-ticket
logs the way `v2-status.md` / `renderer-status.md` did.

## Scorecard (update per session; `make parity-scorecard` once #B-4 lands)

| tier | state | number | notes |
|---|---|---|---|
| 1 Frame parity | PARTIAL | exact-segments 647/647; scripted 647/647; e2e-exact: gate not built | both runtime floors restored green 2026-07-21 (image-policy split); #OR-6 missing |
| 2 Interaction parity | RED | side-channel: gate not built; fuzz-clean-nights: 0 | #OR-1/2/3/7 |
| 3 SDK parity | RED | A-rows closed 0/8 | register A-table |
| 4 Platform parity | PARTIAL | pixel-exact 1468/1468; adapters 2/2; live same-runner 1468/1468 local | static byte-exact 837; live d788 M5 byte-exact 1370; Paravirtual rerun pending; #HD-2's hypothesis oracle and #HD-3 remain |
| 5 Performance & size | RED | ratio 0.897–0.914 (non-blocking, 6 files); size 7.84 MiB OFF / 8.70 MiB ON vs user-approved 9 MiB budget (both variants block; CI recording re-enabled, first green recording pending) | #OR-9 |

Regression floor (must stay green): **restored 2026-07-21, then re-broken
by concurrent main `c7d48ca0` the same day** (see Next queue item 1: ten
trigger probe assertions + four scripted corpus entries, proven present on
pristine origin/main). The 2026-07-21 closeout work itself is green: both
runtime golden gates 317/317 exact / 647/647 segments (the decoded-image
policy split re-admitted `jellyfish_test`), and the seven prior probe
failures are repaired — five were `974aab66` regressions (eager bind-time
component-list sizing), two were genuine long-standing divergences the
stale probe had masked (opacity-zero willDraw filtering in the oracle
stream, unresolved name-based number bind). NOTE: the `RIVE_RUNTIME_DIR`
checkout governs probe/runner builds — it was found at `ba2b6434` and must
be at pin `d788e8ec` for any floor run; unpinned checkouts have now
poisoned two consecutive concurrent-main changes.

Upstream pins: runtime `d788e8ec` (cycle-3 cut `b73bc675`, 3 commits ahead,
awaiting #B-1 approval). Upstream advanced after that completed inventory to
`ba2b6434`; it is next-cycle drift, not part of the pending authorization.
Renderer pixel-oracle `7c778d13` (historical, do not advance casually — see
upstream-sync-map registry).

## Ticket checklist

- [ ] #RB-1 data-binding foundation rebuild (map Phase RB; user-directed
  2026-07-21, P0) — port C++'s retained-identity view-model/data-bind core
  and delete the compensation family; floors are the harness; exit gate
  includes the deletion list. Mini-queue:
  - [x] (a)+(b) retained cell core — `view_model_cell.rs` landed: shared
    typed cells (`Rc<RefCell>` ≈ `rcp`), weak dirt-sink dependents
    (`DependencyHelper` analog; cascade sets bits only, no callbacks),
    `ValueFlags::valueChanged`/`advanced()` semantics including trigger
    zeroing under `SuppressDelegation`; 7 unit tests mirror the C++
    contracts. Additive — no consumers yet, floors untouched.
  - [ ] (c) parent-linked `DataContext` (`data_context.cpp` port): one
    retained context with a parent pointer replacing candidate vectors;
    `getViewModelProperty(path)` walks instances (path[0]=viewModelId) then
    parent, returning retained cells.
  - [ ] (d) retained `DataBind` lifecycle (`data_bind.cpp`): `source(cell)`
    registers the bind's sink as dependent (skipped for bindsOnce); typed
    ContextValues; C++ favored-direction init ordering
    (`updateSourceBinding`, TargetOrigin, `sourceToTargetRunsFirst`).
  - [ ] (e) migrate consumers (state machine, artboard, facade, listeners,
    converters) — sequenced by the compensation-family call-site inventory
    (scout dispatched 2026-07-21); floors green after every migration step.
  - [ ] (f) deletion gate: mutation clocks, candidate vectors, listener
    rescans, alias mirrors, Scene-wide rebind bit all removed; floors at
    completed values including the four currently-red scripted entries.
- [ ] #B-5 editor-cutover parity audit (user-directed 2026-07-21) — scout
  report complete, 12 findings. VERDICT: broadly parity-aligned with
  isolated slips, not structurally off-course — most bytes are additive
  editor surface (project-data-converter format, event-context host APIs,
  converter build cache) plus tests. Risk concentrates in ONE refactor:
  the owned-view-model "candidate" unification (c7d48ca0), which added a
  strict value-kind match guard in `resolve_value_for_source_path`
  (kind mismatch → `bound=false` → serialized-default fallback) and new
  unresolved-source fallbacks synthesizing bindable defaults from
  serialized `propertyValue` (enum default changed 0 → u32::MAX). That
  guard/fallback pair is the prime suspect for the open scalar shift.
  Remaining (c)-class rows needing fixtures before close: (i) 974aab66's
  two-way custom-property seeding no longer target→source-seeds two-way
  binds (C++ seeds both, favored direction wins) — fixture: two-way
  TrimPath.start bind, serialized target ≠ source initial, t=0; (ii) a
  parent-relative nested number/enum bind with serialized target
  propertyValue ≠ 0 at t=0; (iii) self-referential layout recursion guard
  (mark-on-entry → deferred insert); (iv) owned trigger source paths
  deeper than 2 segments. Each becomes a repair or an explicit D-row.
- [ ] #B-1 Phase S sync to b73bc675 — triage submitted; USER-GATE blocks port/pin movement
- [ ] #B-2 port-manifest invariant — implementation/local gate complete at
  exact b73bc675 (447/447: 378 ported / 21 partial / 43 absent / 5 N/A);
  first main CI green pending
- [ ] #B-3 size re-measure — user-approved budget 9 MiB (9,437,184 B) both
  variants, fully wired (`make size-report` blocks; scorecard validates
  `size-report.json`; CI records); close on the first green main CI
  recording
- [ ] #B-4 `make parity-scorecard` — implementation/local gate complete;
  canonical five-floor evidence and CI publication wired; first main CI green
  blocked by the decoded-image policy gate and seven freshly exposed d788
  C++-probe parity assertions
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

1. #RB-1 data-binding foundation rebuild (map Phase RB; user-directed
   2026-07-21) — port C++'s retained-identity model (retained
   `ViewModelInstanceValue` cells, `DependencyHelper` dependent dirt,
   parent-linked `DataContext`, retained DataBind/listener/converter
   lifecycle) and DELETE the compensation family (mutation clocks,
   candidate vectors, listener rescans, alias mirrors, Scene-wide rebind
   bit). Floors are the harness; the four red scripted entries are
   expected to close as a byproduct (their point-fix chase is STOPPED —
   full evidence trail retained below for cross-checking the rebuild).
   Editor-team changes to this layer are frozen until it lands. A final
   supporting fact from the bind-table diff: the pre-rebase machinery
   never rewrote graph sources from the owned context in the runner flow
   at all (empty bind log at `a159897f`), while the candidate binder
   rewrites every source from instance-0 values ([4,0]→95, [3,0]→40,
   [2,4]→1.0667) — the two designs disagree even about WHEN binding
   happens, which is exactly why point-fixing inside them cannot converge.
2. ARCHIVED EVIDENCE for the four scripted entries (was queue item 1;
   subsumed by #RB-1) — FOUR scripted-golden-compare
   entries broken by concurrent main `c7d48ca0` (`db_health_tracker`,
   `echo_show_demo`, `list_index_script_access`, `superbowl`: wild
   transform/gradient/path divergences under forced scripting; the default
   gate is green). Attribution proven on a pristine `5927654b` worktree.
   `c7d48ca0` touched no nuxie-scripting code, so the cause is runtime code
   behaving differently under forced scripting — start from its artboard.rs
   (+1,350 — NOTE: almost all tests; production changes are the
   owned-candidate bind refactor) / data_bind_graph.rs / instance.rs
   changes. LOCALIZED LEAD (2026-07-21, sharpened): in `db_health_tracker`,
   EVERY row-container transform in the scripted lane is shifted by exactly
   -2945.44531 versus both C++ and the pre-rebase-green Rust baseline
   (271.49→-2673.95, 582.49→-2362.95, -20.51→-2965.95, 1195.49→-1749.95;
   y identical, glyph outlines byte-identical). One scalar diverges — the
   list container's scrolled/bound x offset, most plausibly a data-bound
   number through the DataConverterInterpolator/BlendState1DViewModel
   chain evaluating differently under scripting-enabled import. Reproduce:
   run both scripted runners on the file (`--samples 0`, Rust adds
   `--execute-scripts`) and diff; everything else is LSB float noise. A
   green Rust baseline stream builds from pre-rebase commit `a159897f`
   (reflog) with `cargo build -p rust-golden-runner --features scripting`.
   BISECTION COMPLETE THROUGH FOUR STRIKES (2026-07-21): (1) the
   scripting-FEATURE runner build diverges even with `--execute-scripts`
   OFF while the featureless binary from the same tree is exact; every
   runner-side cfg difference was patched out experimentally (scripted
   preallocation, scripted-drawable init, the rebind call) with no effect —
   EXCEPT `selected_artboard_owned_view_model_context`, where the feature
   build constructs the owned main context with
   `RuntimeOwnedViewModelInstance::from_instance(runtime, vm_index, 0)`
   (serialized instance 0) and the featureless build uses `::new`
   (definition defaults); forcing `::new` in the feature build makes the
   stream exact. (2) The runner is CORRECT: C++'s scripted runner does the
   same (`File::createViewModelInstance(viewModelId, 0)` copies serialized
   instance 0) and C++ still renders 271.49 — so the runtime's handling of
   a from_instance-built owned context is at fault. (3) The #B-5 scout's
   kind-guard suspect is EXONERATED: instrumentation shows zero kind
   mismatches; `property_path_for_source_path` fails identically in both
   builds for source paths [3,0] and [2,4] (common, not the delta). (4)
   Property ordering is exonerated: both constructors share
   `from_view_model`'s definition walk; only seeded VALUES differ. (5) The
   value-table diff is DONE (dump
   `runtime_owned_view_model_binding_value_for_property_path` behind an env
   var in both builds, `sort -u`, diff): the instance-0 context resolves
   real serialized data — notably property_path [4,1] = 4022.0 where the
   default context resolves 0.0, plus [0,0..6] = 67/55/70/80/67/75/64 and
   [3,3] ≈ 410–482 vs ≈ 16 — and those values flow into the artboard,
   shifting the rows. ROOT-CAUSE HYPOTHESIS (one step from a fix): C++
   ALSO copies instance 0 yet renders 271.49, so C++'s init must seed
   target→source FIRST (the artboard's serialized state writes into the VM
   before any source→target read — `DataBind::updateSourceBinding` +
   TargetOrigin/`sourceToTargetRunsFirst` favored-direction semantics),
   while c7d48ca0's owned-candidate path in
   `state_machine/instance.rs::bind_owned_view_model_context_candidates`
   applies source→target from the live VM values. Note the artboard side
   (`bind_owned_view_model_artboard_context_candidates`) still calls
   `bind_owned_view_model_target_to_source_bindings` before value
   application — compare the state-machine candidate path against it and
   against pre-rebase `a159897f`, then port the favored-direction ordering
   from C++ `data_bind.cpp`. This also intersects the #B-5 (c)-row (i):
   974aab66 stopped target→source seeding for two-way binds.
   The commit's
   OTHER regression — ten trigger probe assertions — is FIXED 2026-07-21:
   `reset_advanced_data_context` had swapped `trigger.reset()` for
   value-retaining `advanced()`, but C++
   `ViewModelInstanceTrigger::advanced()` zeroes `propertyValue` itself;
   the revert restores 707/707 with every c7d48ca0-added test still green.
   Context: the commit was validated while `RIVE_RUNTIME_DIR` sat at
   drifted `ba2b6434`, and CI's `cargo test --workspace` skips the probe
   suite when `RIVE_CPP_PROBE` is unset, so main went red silently — the
   same class as the five `974aab66` component-list regressions.
3. #B-1 port — execute the approved S3-1 (TextInput) + S3-3 (static linking)
   port per `docs/upstream-sync-map.md`; advance `LAST_SYNCED_SHA` to
   `b73bc675` on a green ratchet. (Text-input code is outside the #RB-1
   layer; may proceed in a lane while #RB-1 holds the spine.)
4. #B-5 editor-cutover parity audit — classify every runtime-behavior hunk
   of `974aab66`/`c7d48ca0` (see Ticket checklist); most (b)/(c) rows are
   expected to dissolve into #RB-1's deletion gate.
5. #OR-1 — side-channel spec + C++ emit once the floor is restored.
6. #FT-TEXT — unblocked by the #B-1 approval; starts after the port lands.

## Pending USER-GATEs

(none — the reopened #B-3 budget was decided 2026-07-21; see the
Decisions log.)

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
- 2026-07-21: **Decoded-image policy split approved (register D-11).**
  Low-level compatibility/golden paths retain every decoded image, exactly
  like pinned C++; the high-level `nuxie::File` path is bounded at 64 MiB
  aggregate by default via `FileImportLimits::max_retained_decoded_image_bytes`;
  `FileImportLimits::unbounded()` is truly unbounded. Only the bounded host
  mode is a deliberate divergence. No C ABI change.
- 2026-07-21: **#B-1 cycle-3 approval granted.** PORT S3-1 (`1b4df2ad`,
  TextInput) and S3-3 (`b73bc675`, static library linking); S3-2 (`079305d7`,
  profiler) deferred WATCH; both dependency WATCH rows retained at staleness 2.
  Authorization covers exactly the `b73bc675` cut; `ba2b6434` drift stays in
  the next inventory.
- 2026-07-21: **#B-3 budget decided: 8 MiB (8,388,608 B), blocking for BOTH
  scripting OFF and ON.** ON has ~52 KiB headroom today; if the approved
  TextInput port pushes ON past the budget, the gate reopens with fresh
  measurements — the constant is never silently raised.
- 2026-07-21: **#RB-1 opened (user decision): rebuild the view-model,
  data-binding, and value layers to match the C++ architecture exactly.**
  Rationale: the Rust port flattened bound sources into copied values,
  losing C++'s retained `ViewModelInstanceValue` identity and dependent
  registration; the resulting compensation family (mutation clocks,
  candidate vectors, listener rescans, alias mirrors, facade-wide rebinds)
  has produced every one of the 20+ editor-cutover parity regressions found
  2026-07-21, including the four still-red scripted entries. Point-fixing
  inside the compensations is stopped; the map gains Phase RB with a
  deletion exit gate. Editor changes to this layer freeze until it lands.
- 2026-07-21: **#B-3 replacement budget user-approved: 9 MiB (9,437,184 B),
  both variants blocking.** The 8 MiB decision predated `974aab66`; honest
  re-measurement with the 43-root harness (OFF 7.84 MiB / ON 8.70 MiB)
  reopened the gate the same day, and the user approved the recommended
  9 MiB replacement (~3.4% headroom over ON). CI evidence recording is
  re-enabled.

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
  floors were green before the concurrent main change; the Paravirtual rerun
  remains pending.
- 2026-07-20 — #HD-2 was rebased intact over concurrent main `974aab66` as
  local commit `bcb6e165`. The historical 7c oracle remains immutable; the
  separately pinned d788 live replay is 1,468/1,468 on M5 with 1,370 byte-exact,
  and the static floor is 1,468/1,468 with 837 byte-exact. The commit is not
  pushed because the required aggregate floor is red.
- 2026-07-20 — Concurrent main `974aab66` introduced a global 64 MiB retained
  decoded-image ceiling. Both runtime golden gates now fail only
  `jellyfish_test`; localization proves image import and codecs are sound and
  the aggregate reservation alone suppresses images 22–23. This is a candidate
  deliberate-divergence row and is stopped at the USER-GATE above.
- 2026-07-20 — Rebuilding `tools/cpp-probe` at exact d788 fixed the apparent
  unknown-uint overflow mismatch: pinned upstream commit `296742c13` and Rust
  both consume the full uint64 fallback. The fresh oracle then exposed seven
  runtime assertions that the prior probe/test ordering had masked: five
  component-list bind cases, opacity-zero shape filtering, and an unresolved
  name-based number bind. These are floor work, not accepted divergences.
- 2026-07-21 — Three USER-GATEs decided (image-policy split, #B-1 approval,
  #B-3 8 MiB budget); see Decisions log. Register gained D-11; H1/H2 updated.
- 2026-07-21 — Image-policy split implemented: `RuntimeRenderImages` carries
  an optional aggregate budget (Default = unbounded, like C++); all low-level
  preallocation entry points stay unbounded; `nuxie::File` threads
  `FileImportLimits::max_retained_decoded_image_bytes` (default 64 MiB,
  `unbounded()` = none) into cache allocation. Both runtime golden floors are
  back to 317/317 exact / 647/647 segments with `jellyfish_test` green.
- 2026-07-21 — Seven d788 probe assertions repaired to 706/706. A pre-974
  control run (same fresh probe, parent commit e21a0ca0) proved five were
  `974aab66` regressions: bind-time component-list sizing eagerly reported
  the populated item count where C++ defers population to the advance pass;
  reverted to bind-time zero. The opacity failure was the oracle-facing
  `draw_commands()` stream retaining opacity-zero drawables C++ filters via
  live `willDraw`; the prepared-frame path now passes `include_invisible`
  (topology retained, replay filters live) while the oracle path applies
  `Shape/TextInputDrawable/Image` opacity checks at build. The name-based
  failure was Rust fabricating an unresolved-source fallback with the
  serialized default where C++ `bindFromContext` unbinds; the fallback is now
  skipped for name-based binds, mirroring the artboard-flow contract.
- 2026-07-21 — #B-3 wired: `make size-report` enforces 8,388,608 B on both
  variants and prints a parseable summary; the scorecard validates recorded
  `size-report.json` evidence against `size.budget_bytes`
  (parity-scorecard.toml) with drift and over-budget as errors; CI records
  the gate in the runtime evidence job; scorecard tests 19/19.
- 2026-07-21 — #B-3 gate fired on its first full run and REOPENED the budget:
  the fail-closed root inventory caught `974aab66`'s new public
  `Factory::make_gpu_canvas_image` (43rd root added to the audited inventory
  and consumer harness), and honest re-measurement reports 7.84 MiB OFF /
  8.70 MiB ON — ON exceeds the 8 MiB decision by 729,496 B because the
  decision was made against pre-974aab66 evidence. The constant was NOT
  raised; the CI recording step is held; the replacement budget is a pending
  USER-GATE (recommendation 9 MiB both variants).
- 2026-07-21 — Rebase over concurrent main `c7d48ca0`/`5927654b` (editor
  cutover gaps + apple runtime 0.1.2). The remote change had independently
  "fixed" jellyfish by doubling the global decoded-image cap to 128 MiB —
  the rejected arbitrary-constant option; the conflict resolved to the
  approved D-11 policy split and the three admission tests were restored to
  the 64 MiB bounded-host form. Post-rebase floors: both runtime golden
  gates 317/317 exact / 647/647, capi-smoke ok, but `cargo test --workspace`
  is 697/707 — ten NEW trigger-flavored probe regressions traced to
  `c7d48ca0` (top of the Next queue), and the scripted gate's exit status
  needs re-examination.
- 2026-07-21 — The c7d48ca0 trigger regression is repaired: one-line revert
  of `advanced()` back to `reset()` in `reset_advanced_data_context`, cited
  against C++ `ViewModelInstanceTrigger::advanced()` which zeroes
  `propertyValue` under SuppressDelegation. cpp-probe 707/707; nuxie-runtime
  lib 324/324 including every test c7d48ca0 added (its retained-value
  semantics was not load-bearing). The four scripted corpus entries remain
  red and stay at the top of the queue.
- 2026-07-21 — Upstream checkout hygiene: `/Users/levi/dev/oss/rive-runtime`
  was at post-inventory `ba2b6434` with no built libraries, so the first
  probe/runner rebuild of the day silently targeted the wrong revision. The
  checkout was restored to pin `d788e8ec` and `librive` (+text/layout
  companions) rebuilt before any floor evidence was accepted.
