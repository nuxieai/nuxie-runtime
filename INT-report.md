# MR-2 C10 nuxie-facade integrator report

## Scope

- Branch: `levi/mr2-c10`
- Sole-owned root: `crates/nuxie/src/lib.rs`
- Base: `origin/main` at `ac6f0d49`, after the three landed MR-2 move trains
- Plan cluster: C10 / `nuxie-facade-integrator`

During final verification, `origin/main` advanced to `4886b6cf` through two
unrelated S4 authorization commits that only modify
`docs/upstream-sync-map.md`. A safe fast-forward was attempted, but the sandbox
could not lock the worktree's `ORIG_HEAD`; this report therefore remains based
on the requested three-train snapshot at `ac6f0d49`.

The C10 hotspot contains 17 rows, and the move plan classifies all 17 as
justified exceptions. Every planned post-move retained-module set continues to
name `crates/nuxie/src/lib.rs`. The root-owned portions are public `File` and
`ArtboardInstance` facade/integration behavior rather than dedicated runtime,
binary, or VM implementations, so moving them would cross a crate boundary or
change the public API.

## Moved rows

None. C10 has no row with a dedicated-module move target for its root-owned
fragment. Consequently there was no four-place residue update to make: no
manifest owner was removed, no frame-loop row/source-set owner was removed, no
attribution comment belonged to a row that fully left, and no orphaned one-line
re-export shim was found.

## Justified exceptions

The exception sentences required by `.parity-decomp/mr-move-plan.md` were
already present in `file-correspondence-manifest.toml` from the C17 exception
seed. No `audit_record`, `b6_row_id`, or `b6_verdict` changed.

| Rows | Exception class | Root-owned entanglement that stays |
|---|---|---|
| `B6-0068`, `B6-0077`, `B6-0319`, `B6-0320` | crate-bound facade/runtime adapter | Public state-machine construction, input hydration, and scripted-listener host integration bridge the runtime instance to the facade-owned script runtime. |
| `B6-0094`, `B6-0113` | crate-bound facade/runtime adapter | Public artboard construction/advance/draw and audio-event host routing remain facade methods over runtime-owned implementations. |
| `B6-0106` | host/runtime/VM adapter split | `FileScriptRuntime` owns authenticated file-level script policy, VM lifetime, and public host registration while runtime and Luau implementations remain in their crates. |
| `B6-0208`, `B6-0213` | schema/crate boundary | The public `File` import/catalog surface must call the binary loader and construct runtime state; it cannot move into either foreign crate without changing the facade API or dependency direction. |
| `B6-0260`, `B6-0261`, `B6-0262` | host/VM adapter split | Public logging, artboard, and audio host routes remain on the file/artboard facade while Luau userdata and VM bindings remain in `nuxie-scripting`. |
| `B6-0321`, `B6-0322`, `B6-0323`, `B6-0324`, `B6-0325` | facade/runtime/VM integration split | Facade-owned occurrence preparation and callback hosting bridge runtime scripted owners to the authenticated per-file VM; extracting them would require a cross-crate ownership or signature change. |

## Cross-root queue

Five rows already match the plan's final retained-module set and need no
foreign reconciliation: `B6-0068`, `B6-0260`, `B6-0261`, `B6-0319`, and
`B6-0320`.

The following rows retain non-C10 modules beyond, or instead of, the plan's
post-move set. C10 left those foreign roots and their metadata untouched; their
same-boundary consolidation must be completed by the named root owners and
landed atomically before the manifest/frame-loop ledgers can be narrowed.

| Row | Foreign reconciliation still required |
|---|---|
| `B6-0077` | C11 must reconcile the remaining state-machine leaf fragments into `state_machine/state_machine_instance.rs`. |
| `B6-0094` | C04/C05/C11 must reconcile retained `draw.rs` and listener-action-owner fragments with `artboard.rs`. |
| `B6-0106` | C11/C16 must reconcile scripted-listener/data-converter fragments with `script_asset.rs`; C08's VM adapter remains an exception. |
| `B6-0113` | C04/C11 must reconcile artboard/state-machine fragments with `audio_event.rs`. |
| `B6-0208` | C07/C09/C14 must reconcile binary asset fragments and establish the planned binary `file.rs` owner. |
| `B6-0213` | C06/C07/C11 must remove the remaining importer-root residue while retaining the planned importer, binary-root, and runtime-template owners. |
| `B6-0262` | C08 must reconcile the extra `vm/view_model.rs` fragment with `vm/lua_audio.rs`. |
| `B6-0321` | C11/C16 must reconcile data-bind/artboard/state-machine fragments with `scripted_data_converter.rs`. |
| `B6-0322` | C04/C05/C16 must create/finalize the planned dedicated scripted-drawable owner; it does not exist in the current tree. |
| `B6-0323` | C01/C08/C16/tool owners must reconcile the remaining animation/scripting fragments; this preserves C01's prior cross-root queue. |
| `B6-0324` | C04/C05/C16 must reconcile artboard/draw/scripting fragments with `scripted_layout.rs`. |
| `B6-0325` | C04/C16 must reconcile artboard/scripting fragments with `scripted_object.rs`. |

## Verification

- Confirmed all 17 C10 manifest rows retain `crates/nuxie/src/lib.rs` and the
  plan-seeded exception sentence.
- Confirmed all upstream-file mentions remain valid because no C10 row fully
  left the facade root.
- Confirmed `crates/nuxie/src/lib.rs`, `file-correspondence-manifest.toml`, and
  `docs/runtime-frame-loop-ownership.toml` were not modified.
- `cargo check --workspace --exclude nux-capi --tests`: passed after the
  repository's checksum-pinned `tools/fetch-test-assets.sh` copied the ignored
  test fixtures from the local pinned Rive checkout (no network access).
- `make runtime-frame-loop-port-check`: passed (108 unit tests plus
  correspondence and ownership checks).
- `make rust-attribution-check`: passed (10 unit tests plus complete in-scope
  Rust-source attribution coverage).
