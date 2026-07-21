# #B-6 Triage (planner pass, 2026-07-21)

Input: SUMMARY.md (447 rows: 19 ISOMORPHIC / 182 ADAPTED / 162 DIVERGENT /
36 UNKNOWN / 48 N/A; 258 mutation-gated mechanisms). Spot-checks performed:
B6-0123 (top-ranked, verified — mechanisms real, disposition nuanced),
focus-input family (verified credible), mechanism census across all
results files. The 162 DIVERGENT rows collapse into four families:

## Family A — RB-1 scope (~65 rows) — already in flight, no new tickets

All 50 `default_value_is_resolved` rows (data-bind-view-model cluster)
plus `owned_view_model_candidate_generation_rebind` (5),
`listener_observed_copy_rescan` (4), `view_model_trigger_observed_reset`,
`scene_rebind_view_model_input_rehydration`, detached/stateful view-model
mirror rows, AND the keyframe family
(`key_frame_prototype_revision_poll` x6,
`copied_key_frame_value_holder_refresh` x6) — the same graph machinery
cloned per animation. DISPOSITION: covered by the #RB-1 rebuild;
keyframe data-bind graphs are hereby recorded as explicit RB-1 slice
(e)/(f) scope. These rows re-audit automatically at the RB-1 deletion
gate.

## Family B — Retained-renderer invalidation epochs (~60-70 rows) — PROPOSED D-ROW (user decision)

`cache/prepared/command/path/layout/text/draw_order/tree-paint` epochs and
their fanout rows (dominating layout-shapes-paint 44/54, component.cpp
B6-0123, artboard, parts of constraints/text). VERIFIED REAL but the
disposition is not "rebuild": C++ redraws through live objects each frame;
Phase R deliberately built a pure-Rust RETAINED renderer, and the epochs
are the instance-to-cache invalidation bridge that architecture requires.
They are pixel-gated (1,468/1,468) and golden-gated. PROPOSAL: one
explicit accepted-architecture D-row ("retained renderer invalidation
epochs are the deliberate Phase R design; C++ has no counterpart because
it has no retained replay caches"), with the standing rule that any epoch
later found compensating for a missed PORT (not bridging to the renderer)
is carved out and fixed individually. USER APPROVED 2026-07-21 — recorded as register D-12; the ~60-70 Family B rows are closed as documented-and-intentional.

## Family C — New rebuild candidates (planner-verified or pending)

- **RB-2 (open): focus system** (focus-input cluster, 3/3 rows, high
  confidence, spot-verified): C++ FocusNode retains live Focusable
  pointers; Rust stores copied booleans/geometry + occurrence-key lookup
  maps refreshed each advance/pointer cycle (AF-1 + AF-8). Also a flagged
  possible behavior gap: no non-test consumer of take_events, no mapped
  key/text/gamepad focus dispatch — intersects #FT-TEXT's keyboard work.
  Small subsystem; rebuild after RB-1's pattern.
- **Pending planner verification** (one pass each before ticketing):
  scripting `deferred_script_advance_queue` (3), mesh/slice vertex
  snapshot family (3-5), `script input kind/name/default` snapshots (4),
  `solid-color revision handoff` (1), `target_lookup_rebuild` (1).

## Family D — UNKNOWN (36) and N/A (48)

N/A rows are the manifest's absent/not-applicable files — correct. The 36
UNKNOWNs (9 animation, 9 text, 4 assets-importers, 4 constraints, 3
lua-scripting, 3 scripted, rest scattered) get a focused second pass with
the blockers named in each record; schedule after RB-1 lands so the
data-adjacent ones audit against stable code.

## Judge feedback (for the spec)

The five-axis brief + amendments held up: no false positives found in
spot-checks; the one systematic gap is DISPOSITIONAL, not detective — the
spec needs a third outcome lane ("deliberate architectural divergence,
maps to recorded decision X") so retained-renderer-class findings don't
read as rebuild demands. Added to the spec's next revision.
