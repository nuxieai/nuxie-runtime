# F6FID commit map

## Completed commit

`5b84f06b Port retained semantic focus runtime`

- `crates/nuxie-runtime/src/artboard.rs`: retained focus traversal across
  mounted occurrences and pinned two-corner artboard-boundary projection.
- `crates/nuxie-runtime/src/constraints.rs`: pinned
  `scrollConstraintToShowBounds`, effective offsets, snap selection, and
  elastic/direct scroll routing.
- `crates/nuxie-runtime/src/constraints/scrolling/scroll_physics.rs`: retained
  effective target and elastic `scrollToPosition` path.
- `crates/nuxie-runtime/src/focus_data.rs`: owner-qualified focus target
  selection for repeated mounted local ids.
- `crates/nuxie-runtime/src/semantic_data.rs`: live authored/provider refresh,
  retained tree membership reconciliation, focus no-op fidelity, and lifecycle
  evidence.
- `crates/nuxie-runtime/src/semantic_manager.rs`: exact upstream direct manager,
  ordering, incremental-diff, dispatch, and focus-rejection cases.
- `crates/nuxie-runtime/src/semantic_provider.rs`: four-corner mounted root
  transform bounds.
- `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`: retained
  occurrence tree, routes, listener registration/action drain, focus handoff,
  and pending scroll consumption.
- `crates/nuxie-runtime/tests/semantic_focus_runtime.rs`: four exact RB-2 focus
  cases.
- `tools/fetch-test-assets.sh`: pinned fixture registration.
- `fixtures/semantic/semantic_list_scroll_focus_fixed.riv`: ignored fixture,
  successfully staged with `git add -f` in the completed commit.

## Pending coherent commit

Suggested message: `Record F6 semantic fidelity evidence`

- `file-correspondence-manifest.toml`: keep all four canonical semantic source
  rows pending while recording F6 evidence and named #LT-1 residue.
- `port-manifest.toml`: promote only `semantic_data`, `semantic_manager`, and
  `semantic_provider` to partial; keep `semantic_inference_registry` absent.
- `test-correspondence-manifest.toml`: promote direct dispatch/label suites,
  record the four state-machine focus cases, and tighten pending 81 to 79.
- `tools/port-manifest/port_manifest.py`: make the three partial rows and the
  unpromoted inference row reproducible by the generator.
- `F6FID-report.md`: lane status, evidence, LT-1 residue, and shared queued
  items. This file is ignored by `*-report.md` and requires `git add -f`.
- `F6FID-map.md`: this fallback commit map.

Staging commands:

```sh
git add file-correspondence-manifest.toml port-manifest.toml \
  test-correspondence-manifest.toml tools/port-manifest/port_manifest.py \
  F6FID-map.md
git add -f F6FID-report.md
git commit -m "Record F6 semantic fidelity evidence"
```

The attempted commit failed before staging with:

```text
fatal: Unable to create '/Users/levi/dev/nuxie-runtime/.git/worktrees/nuxie-mr-c14/index.lock': Operation not permitted
```
