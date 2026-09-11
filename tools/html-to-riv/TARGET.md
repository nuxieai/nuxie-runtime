# Immutable Rive target

This compiler targets the runtime tree at `6c7ac16617835b5f581784ff08a9e779bb52faf3` (tree `25ccbb131d88dbd0fdde8f4c660919143b252edf`). PR #629 restored that exact tree on origin/main as `9738049372ffd45639de39c2217c1f548963340a`.

The runtime, renderer, public render interfaces, schema/wire format, shared dependencies and their build configuration are fixed inputs. A compiler feature cannot change them to become supported. Even an independently useful bug fix is a separate runtime proposal; do not retain it in this module's changes.

The output is a self-contained ordinary `.riv` scene plus optional diagnostic source metadata. Importing and drawing the bytes through the baseline public interfaces must suffice. No runtime-requirements sidecar, post-import CSS setters, replacement font shaper, new shaders, private recording commands, scripts, bindings or custom host paint adapter may supply missing semantics. Source maps identify objects for read-only inspection, never control rendering.

A file-level shim is a composition of already supported Rive objects and properties stored in the emitted file. It must preserve the intended geometry, painting, clipping and responsive resize behavior. A wrapper that runs CSS logic on the host is not a file-level shim. Neither recompilation on resize nor browser-baked rectangles or raster screenshots are the current output contract. Discuss any such output-profile change explicitly before adopting it.

## Capability decisions

Every feature and combination receives one of these evidence states:

1. **Native candidate:** a matching ordinary property/object exists. This is a source finding, not support.
2. **Composition candidate:** a proposed arrangement of ordinary objects might express it. Record the arrangement, resource bounds and responsive behavior to test.
3. **Qualified:** the native or composed file passes public Rust/CLI/WASM parity, baseline import, read-only geometry, real renderer pixels, same-file original/clone resize and visual review in the documented profile.
4. **Unsupported:** a demonstrated baseline limitation rules out the attempted contract. Preserve a minimal failure and explain the missing capability. Runtime changes may be proposed separately, never implemented to make this row pass.
5. **Unresolved:** evidence does not yet distinguish a viable composition from a limitation. Do not call it impossible or supported.

Recognition of CSS syntax is separate from admission. Unsupported computed contexts must fail with precise diagnostics before output is published. A whitelist of property names cannot capture context-dependent layout, clip, paint-order and opacity limitations.

## Migration order

1. Revert the coupled PR completely. **Done:** PR #629 and full-tree equality.
2. Inventory every mutation and policy, preserving historical evidence as historical. **Source audit done:** validation/reverted-mutations.json and the three immutable audit documents. Replacement experiments remain open.
3. Establish an executable source/dependency identity check and a bytes-only baseline importer/renderer test path. Use baseline-owned build resolution, not a compiler workspace patch that changes runtime dependencies.
4. Restore compiler-owned parsing/cascade/assets/wire code selectively in an isolated module. Delete policy transport and host mutation routes. Do not cherry-pick the broad feature commit.
5. Work through all 99 backlog items in their original priority order. Test ordinary Rive capabilities and composition candidates; qualify or diagnose from evidence. Preserve original/clone resizing and visual inspection. No new radial renderer work.

Historical native receipts from PR #628 do not certify this target. Chrome references may be reused only after exact source/reset/font/image/viewport identity checks. Regenerate native artifacts with the immutable build.
