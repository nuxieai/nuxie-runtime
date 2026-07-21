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
| 5 Performance & size | RED | ratio 0.897–0.914 (non-blocking, 6 files); size 7.84 MiB OFF / 8.70 MiB ON vs 8 MiB budget — ON breaches; budget USER-GATE reopened, CI recording held | #OR-9, #B-3 reopened USER-GATE |

Regression floor (must stay green): **restored 2026-07-21.** Both runtime
golden gates are 317/317 exact, 647/647 segments (the decoded-image policy
split re-admitted `jellyfish_test`); the C++-probe suite is 706/706 at the
exact `d788e8ec` probe. Five of the seven probe failures were regressions
introduced by concurrent main `974aab66` (eager bind-time component-list
sizing), not stale-probe artifacts; the other two (opacity-zero willDraw
filtering in the oracle stream, unresolved name-based number bind) were
genuine long-standing divergences the stale probe had masked. NOTE: the
`RIVE_RUNTIME_DIR` checkout governs probe/runner builds — it was found at
`ba2b6434` and must be at pin `d788e8ec` for any floor run.

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
- [ ] #B-3 size re-measure — budget decided (8 MiB both variants) and wired
  (`make size-report` blocks; scorecard validates `size-report.json`), but
  the gate REOPENED same-day: post-974aab66 measurement is 7.84/8.70 MiB and
  ON breaches. CI recording held pending the replacement budget USER-GATE
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

1. Regression repair — restore the seven d788 C++-probe assertions (five
   component-list binds, one opacity-zero draw filter, one name-based number
   bind) with the probe rebuilt at the exact pin.
2. #B-1 port — execute the approved S3-1 (TextInput) + S3-3 (static linking)
   port per `docs/upstream-sync-map.md`; advance `LAST_SYNCED_SHA` to
   `b73bc675` on a green ratchet.
3. #B-3 — wire the decided 8 MiB dual-variant budget into `make size-report`
   / the scorecard and close the ticket.
4. #OR-1 — side-channel spec + C++ emit once the floor is restored.
5. #FT-TEXT — unblocked by the #B-1 approval; starts after the port lands.

## Pending USER-GATEs

- **#B-3 size budget REOPENED (2026-07-21).** The decided 8 MiB dual-variant
  budget was breached the same day by honest re-measurement: concurrent main
  `974aab66` (editor cutover) grew the runtime and added
  `Factory::make_gpu_canvas_image` to the public renderer surface, which the
  fail-closed root inventory caught; with the 43rd root retained, the
  closures measure OFF 8,216,984 B (7.84 MiB) and ON 9,118,104 B (8.70 MiB,
  729,496 B over). Per the decision's terms the constant was not raised: the
  gate is red locally and the CI evidence-recording step is held. Decide the
  replacement budget. Recommendation: 9 MiB (9,437,184 B) for both variants
  (~3.4% headroom over ON); alternatives: 8 MiB scripting-OFF-only blocking
  (ON unbounded until #FT scope lands), or demand a size reduction from the
  974aab66 surface before re-budgeting.

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
- 2026-07-21 — Upstream checkout hygiene: `/Users/levi/dev/oss/rive-runtime`
  was at post-inventory `ba2b6434` with no built libraries, so the first
  probe/runner rebuild of the day silently targeted the wrong revision. The
  checkout was restored to pin `d788e8ec` and `librive` (+text/layout
  companions) rebuilt before any floor evidence was accepted.
