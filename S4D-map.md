# S4D ordered commit reconstruction map

The sandbox cannot create the shared-worktree Git index lock. Reconstruct the
following commits in order. `S4D-report.md` is ignored by the repository's
broad `*-report.md` pattern and must be force-added; this map is untracked but
not ignored.

## 1. S4-7 — `beb246e5`

Commit message:

```text
[sync] Port rive-runtime beb246e5: Focus dirty flag avoids per-frame tree walks
```

Files/hunks:

- `crates/nuxie-runtime/src/focus.rs`
  - import `Cell` alongside `RefCell`;
  - add `FocusManager::focusable_content_cache`;
  - preserve the cache for unchanged `insert_child` projections and invalidate
    it for actual topology changes;
  - invalidate unrestricted `node_mut`, add exact manager-owned can-focus and
    focusable-backing mutation entry points, and add predicate-aware
    `update_node` for retained projection;
  - cache `has_focusable_content` and dirty it from `unlink`;
  - route `RuntimeFocusTree::sync` through `update_node`;
  - add the four upstream invalidation scenarios plus an unchanged-projection
    performance regression.
- `S4D-report.md`
  - initial report/table and S4-7 implementation/gate status.
- `S4D-map.md`
  - this reconstruction entry only.

Required gates: all green. `cargo test -p nuxie-runtime` reported 877 passed,
1 ignored; `cargo test -p nuxie --features scripting` completed all suites;
both structural make gates passed.

## 2. S4-8 — `18411981`

Commit message:

```text
[sync] Port rive-runtime 18411981: Tree-shake unused scripts from runtime export
```

Files/hunks:

- `crates/nuxie/src/scene.rs`
  - add the versioned require-edge verification contract and fail-safe stamp;
  - retain direct dependency ids and explicit include-in-export state per
    authored script;
  - expose transactional edge/include setters and whole-graph verification;
  - invalidate verification on script-set and edge edits;
  - collect runtime script roots from scripted drawables, layout
    interpolators, data converters, and explicit roots, then retain their
    transitive dependencies in dependency-first order;
  - preserve the previous keep-all behavior for absent or stale verification.
- `crates/nuxie/tests/scene_authoring.rs`
  - add verified tree-shaking, dependency ordering, file-id remapping, explicit
    root, reachable module-cycle, and post-verification edit invalidation
    coverage.
- `tools/nuxie-codegen/tests/generated_schema.rs`
  - prove editor-only `scriptProtocolValue` (1035) and
    `scriptEdgesVerifiedVersion` (1042) do not enter the runtime schema.
- `S4D-report.md`
  - record S4-8 implementation and gate status.
- `S4D-map.md`
  - this reconstruction entry only.

Required gates: all green, including the full `nuxie-codegen` suite and the
four cycle-required cumulative gates.

## 3. S4-16 — `482b24a1`

Commit message:

```text
[sync] Port rive-runtime 482b24a1: Virtualized scroll sizing/overscroll and listener-event fixes
```

Files/hunks:

- `crates/nuxie-runtime/src/constraints/scrolling/scroll_virtualizer.rs`
  - include leading/trailing padding in finite virtualized content size while
    preserving padding-free infinite-cycle sizing.
- `crates/nuxie-runtime/src/constraints.rs`
  - pass authored content padding into both virtualized axes;
  - add finite/infinite padding coverage, the no-physics overscroll regression,
    and real import coverage for the clipped component-list fixture.
- `crates/nuxie-runtime/src/constraints/scrolling/scroll_constraint_proxy.rs`
  - preserve raw drag overscroll for physics owners and immediately clamp
    finite axes when no physics owner exists.
- `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`
  - retain events first reported during an apply-events loop for one host read;
  - preserve the 100-batch ceiling with correct retained-event visibility;
  - report nonzero trigger cells adopted after an earlier trigger fire;
  - bind and scan imported data-context listener trigger cells;
  - add direct trigger-relink coverage and execute the upstream listener fixture
    through its real imported view-model/state-machine path.
- `fixtures/sync/component_list_clipped_viewport.riv`
  - exact upstream fixture; SHA-256
    `a20c9fd4936c2b7f435011e7afddd276797e95d68b574ec2c914331afd092bac`;
    ignored by the broad fixture rule and must be force-added.
- `fixtures/sync/vm_listener_fire_event.riv`
  - exact upstream fixture; SHA-256
    `683a8ed1ad102fa9dd1020d61df301594a9a9fd20b97655c1f0da62e7b994838`;
    ignored by the broad fixture rule and must be force-added.
- `tools/fetch-test-assets.sh`
  - pin both fixture checksums and `482b24a188bb9e367e983bf05235761707a89718` source refs.
- `.s4-deferred-corpus.toml`
  - append both fixtures as `not-yet`; do not enroll them in `corpus.toml`.
- `docs/runtime-frame-loop-gaps.toml`
  - preserve the exact-100-batch ratchet while accepting the now-semantic
    `iteration` loop binding used to retain batches after the first.
- `S4D-report.md`
  - record S4-16 implementation, fixture provenance, and gate status.
- `S4D-map.md`
  - this reconstruction entry only.

Required gates: all green. `cargo test -p nuxie-runtime` reported 882 passed,
1 ignored; `cargo test -p nuxie --features scripting` completed all suites;
both structural make gates passed. Fixture checksums and fetch-script syntax
were verified locally.

## 4. S4-19 — `15da0652`

Commit message:

```text
[sync] Port rive-runtime 15da0652: Return computed width/height for images
```

Files/hunks:

- `crates/nuxie-runtime/src/shapes/image.rs`
  - add the occurrence-owned computed-size projection from intrinsic image
    dimensions and effective modern/legacy render scale.
- `crates/nuxie-runtime/src/draw.rs`
  - return Image-specific computed width/height instead of Node's zero stub;
  - add direct composed-scale coverage and the complete upstream fixture
    checkpoints before and after its layout animation.
- `fixtures/sync/image_computed_transform_bind.riv`
  - exact upstream fixture; SHA-256
    `17a6d1e6e9f9713cf78d522b96957c21c12d60aa40d54285759bee151c9f4730`;
    ignored by the broad fixture rule and must be force-added.
- `tools/fetch-test-assets.sh`
  - pin the fixture checksum and
    `15da0652fc10b55ef1fbd32e3e19582c9dc271f2` source ref.
- `.s4-deferred-corpus.toml`
  - append the fixture as `not-yet`; do not enroll it in `corpus.toml`.
- `S4D-report.md`
  - record S4-19 implementation, fixture provenance, and gate status.
- `S4D-map.md`
  - this reconstruction entry only.

Required gates: all green. `cargo test -p nuxie-runtime` reported 884 passed,
1 ignored; `cargo test -p nuxie --features scripting` completed all suites;
both structural make gates passed. The fixture checksum, generated type-key
features, fetch-script syntax, rustfmt, and `git diff --check` were verified.

## 5. S4-27 — `6fcceeb4`

Commit message:

```text
[sync] Port rive-runtime 6fcceeb4: Compile-time Luau atom lookup and Unreal/Linux build refactor
```

Files/hunks:

- `crates/nuxie-scripting/src/vm.rs`
  - port all 268 upstream atom name/discriminant pairs;
  - build the 1,024-slot open-addressed FNV-1a table at compile time;
  - reject names above the upstream maximum before probing and compare the
    exact callback-provided byte length;
  - install the resolver on each new Luau VM before Rive initialization;
  - exhaustively test every pair, misses, embedded NUL bytes, maximum length,
    and one live `lua_tostringatom` callback result.
- `S4D-report.md`
  - record S4-27 implementation, exclusions, and gate status.
- `S4D-map.md`
  - this reconstruction entry only.

Explicitly excluded: every non-`src/lua/rive_lua_libs.cpp` hunk in the mixed
upstream commit, including Unreal/Linux packaging and generic C++ build fixes.
The Luau engine pin and all vendor directories remain frozen.

Required gates: all green. The four cycle-required cumulative gates passed;
focused atom tests passed; rustfmt and `git diff --check` passed; and a
mechanical diff proved the Rust table exactly matches the upstream 268-tuple
name/ID list.

## 6. S4-43 — `395defdb`

Commit message:

```text
[sync] Port rive-runtime 395defdb: Suppress dithering at alpha 0; rev Luau to `rive_0_732`
```

Files/hunks:

- `tools/rive-runtime-patches/395defdb-alpha-zero-dither.patch`
  - exact byte-for-byte shader-only diff from upstream `395defdb`; SHA-256
    `9a6e72367d2e30f0c7572c28f3babf8600a80f3a7ecef80f376efac72a409b89`.
- `tools/generate-renderer-shaders.sh`
  - preserve the protected `d788e8ec` runtime revision and apply the authorized
    overlay to an isolated shader-tree copy;
  - load pinned PLY 3.11 from the frozen runtime checkout for offline builds;
  - preserve Naga failures without the sandbox-blocked `/dev/fd` process
    substitution.
- `tools/renderer-shaders/clockwise_atomic_path_webgpu.main`
  - adapt Nuxie's sampled-clip fork to the regenerated minifier aliases.
- `crates/nuxie-renderer/src/generated/*.wgsl`
  - regenerate all 66 modules from the pinned base plus exact overlay; the
    complete digest is
    `3057f6a6ed0e1ed0419cb2962300f776ae32964aa5e63e258136709a771da78f`.
- `tools/check-renderer-shaders.sh`
  - ratchet the regenerated Rust-module and 56-header compiler-input digests.
- `crates/nuxie-renderer/src/lib.rs`
  - add the transparent-overdraw renderer golden for MSAA and
    clockwise-atomic paths.
- `S4D-report.md`
  - record S4-43 implementation, provenance, exclusions, and gates.
- `S4D-map.md`
  - this reconstruction entry only.

Explicitly excluded: `.rive_head` and `scripting/premake5.lua`; the Luau
`rive_0_732` vendor move remains WATCH and every Luau/luaur pin stays frozen.

Required gates: all green. The four cycle-required cumulative gates passed;
the shader provenance gate reproduced 66 WGSL modules and 56 compiler-input
headers exactly; focused Naga validation passed. The renderer-golden test
compiled and returned green, but the local machine exposed no GPU adapter, so
the pixel assertion remains for the renderer GPU lane.
