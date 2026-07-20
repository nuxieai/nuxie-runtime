# Parity Closeout Status

Live state for `docs/parity-closeout-map.md`. The next session must be able
to resume from this file alone. Keep it small: archive completed-ticket
logs the way `v2-status.md` / `renderer-status.md` did.

## Scorecard (update per session; `make parity-scorecard` once #B-4 lands)

| tier | state | number | notes |
|---|---|---|---|
| 1 Frame parity | PARTIAL | exact-segments 647/647; e2e-exact: gate not built | floor green; #OR-6 missing |
| 2 Interaction parity | RED | side-channel: gate not built; fuzz-clean-nights: 0 | #OR-1/2/3/7 |
| 3 SDK parity | RED | A-rows closed 0/8 | register A-table |
| 4 Platform parity | PARTIAL | pixel-exact 1468/1468; adapters 1 | Dawn-WebGPU/M5-Max only; WebGL2 undecided |
| 5 Performance & size | PARTIAL | ratio 0.897–0.914 (non-blocking, 6 files); size STALE | #OR-9, #B-3 |

Regression floor (must stay green): `make golden-compare` 317/647 ·
`make scripted-golden-compare` · `make renderer-golden` 1468 ·
`cargo test --workspace` · `make capi-smoke`.

Upstream pins: runtime `d788e8ec` (upstream head `b73bc675`, 3 commits
ahead — see #B-1). Renderer pixel-oracle `7c778d13` (historical, do not
advance casually — see upstream-sync-map registry).

## Ticket checklist

- [ ] #B-1 Phase S sync to b73bc675 (USER-GATE at triage approval)
- [ ] #B-2 port-manifest invariant (447/447 rows, CI check)
- [ ] #B-3 size re-measure (USER-GATE for new budget)
- [ ] #B-4 `make parity-scorecard` in CI
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
- [ ] #HD-2 renderer oracle hardening (V7)
- [ ] #HD-3 WebGL2 decision (USER-GATE)
- [ ] #HD-4 TODO(golden) pair
- [ ] #HD-5 publish the parity claim doc
- [ ] #LT-* long tail (each opens by USER-GATE)

## Next queue (top = next; orchestrator maintains)

1. #B-1 — run /sync-upstream inventory + triage; STOP at approval gate.
2. #B-4 — scorecard plumbing (lane; unblocks honest per-commit tracking).
3. #B-2 — port-manifest tool (lane).
4. #B-3 — size re-measure (lane; ends at USER-GATE).
5. #OR-1 — side-channel spec + C++ emit (spine; start once #B-1 triage is
   submitted — porting approved rows can interleave).

## Pending USER-GATEs

(none submitted yet)

## Decisions log

- 2026-07-20: Project created from `docs/parity-gap-register.md` (six-way
  evidence sweep). Method/threading/routing inherited from
  `.claude/commands/goal.md` culture; session protocol at
  `.claude/commands/parity.md`.

## Log

(newest first; one line per session/merge)
