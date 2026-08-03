# P3E file/hunk map

Commits could not be created in this worktree. The sandbox allows source-file writes but rejects the linked Git worktree metadata lock at `/Users/levi/dev/nuxie/.git/worktrees/nuxie-mr-c15/index.lock`. The requested `levi/p3e-lua-gpu` branch was also absent locally, so this work remains on the pre-existing `levi/mr2b-c05` checkout for orchestrator extraction. The pre-existing `MR2B-report.md` modification was not touched.

Pinned source used: `rive-runtime` `d788e8ec6e8b598526607d6a1e8818e8b637b60c`, chiefly `src/lua/renderer/lua_gpu.cpp:34-3717`.

## Implementation hunks

- `crates/nuxie-render-api/src/lib.rs`: extend the backend-neutral Lua GPU plan with vertex step mode, index buffers/draws, texture uploads/views, sampler bindings, pipeline state, depth/stencil, and dynamic pass state.
- `crates/nuxie-scripting/src/gpu_canvas.rs`: implement the pinned GPU-prefixed Lua userdata surface and descriptor/member names; preserve buffers/resources into the typed plan.
- `crates/nuxie-renderer/src/gpu_canvas.rs`: validate reflected WGSL resources and execute the plan through wgpu, including texture allocation/upload/view, sampler creation, generic bind groups, indexed draw, blend/primitive/depth state, multisampling, viewport/scissor/stencil/blend state, and pipeline caching.
- `crates/nuxie-renderer/src/lib.rs`, `crates/nuxie/src/lib.rs`: re-export the extended public transport types.
- `crates/nuxie-scripting/tests/fixtures/lua-gpu-full-surface.luau`: authored focused oracle; deliberately not a corpus entry.
- `crates/nuxie-scripting/tests/gpu_canvas_tools.rs`: focused full-surface plan assertions.
- `crates/nuxie-scripting/tests/shader_asset_resolution.rs`, `crates/nux-capi/src/size_report_roots.rs`: update existing `GpuCanvasPlan` literals for the extended transport.

## Ledger hunks

- `file-correspondence-manifest.toml`: record the P3E direct owners and evidence while leaving the row `pending` until the remaining semantic combinations are implemented and the orchestrator lands the approved D-row.
- `docs/runtime-frame-loop-ownership.toml`: add the Lua GPU scripting handoff anchors to the existing `scripting.render_context` update/advance/draw lifecycle; backend-only renderer anchors stay outside this ownership row and status counts remain unchanged.
- `tools/port-manifest/port_manifest.py`, `tools/port-manifest/test_port_manifest.py`, `port-manifest.toml`: move the mixed upstream file from `absent` to `partial`; GPU-prefixed names are implemented, while non-GPU-prefixed `Canvas` and `Image:view` residue remains outside P3-e.

## Closeout files

- `P3E-report.md`: evidence, proposed D-row text, pending rows, and shared-file conflict queue.
