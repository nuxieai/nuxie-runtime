# S4B ordered commit reconstruction map

The sandbox cannot create the shared-worktree Git `index.lock`, so the rows
below are the intended commit boundaries. Force-add `S4B-report.md`,
`S4B-map.md`, and every listed `fixtures/sync/*.riv` because repository ignore
rules cover them.

## 1. S4-3 — `b73bc675`

Commit message:

```text
[sync] Port rive-runtime b73bc675: Statically link library requires
```

Owned files/hunks:

- `crates/nuxie-schema/src/generated/schema.rs`: complete generated-schema diff
  (exact regeneration from `b73bc675:dev/defs`).
- `crates/nuxie-schema/src/lib.rs`: remove runtime `UintStorage::Uint64`.
- `crates/nuxie-schema/tests/generated_schema.rs`: remove live FileAsset scope
  field/uint64 expectations.
- `tools/nuxie-codegen/tests/generated_schema.rs`: S4-3 schema census changes
  (341 definitions, 592 properties, no runtime uint64, 442 descriptions,
  3 journal fields, 548 initial values).
- `crates/nuxie-binary/src/assets/file_asset.rs`: remove `LibraryAsset` handling.
- `crates/nuxie-binary/src/core/field_types/mod.rs`: remove active uint64
  dispatch.
- `crates/nuxie-binary/src/core/field_types/core_uint64_type.rs`: compatibility
  comment/allow only; file intentionally remains until the integrator advances
  the correspondence pin.
- `crates/nuxie-binary/src/lib.rs`,
  `crates/nuxie-binary/tests/{authoring_records,cpp_import}.rs`: remove runtime
  LibraryAsset/known-uint64 behaviors and update the upstream source audit.
- `crates/nuxie-scripting/src/vm.rs`: remove `ScopeKey`, serialized pin tables,
  and scoped registration APIs; make `require` and module/protocol registration
  use prelinked names verbatim; add full/short-name blob and shader
  registration. Reserve checked parent-frame headroom before registry/table
  conversion and use raw cache access, preventing a failed candidate module
  from leaving the next host operation at luaur's `ci->top` boundary.
- `crates/nuxie-scripting/src/vm/lua_blob.rs`: calling-chunk-aware scoped asset
  reference ranking for host, own-library, and `lib:` blob lookups; preserve
  authored order for equal-rank matches and expose the authored short name.
- `crates/nuxie-scripting/src/gpu_canvas.rs`: apply the same scoped-reference
  ranking to imported ShaderAssets, retaining full name, short name, owner, and
  authored order rather than a global bare-name alias.
- `crates/nuxie-scripting/tests/library_scope.rs`: replace runtime pin-table
  tests with flat prelinked-name tests; add the exact candidate module-graph
  failure regression proving the parent VM remains usable.
- `crates/nuxie-scripting/tests/shader_asset_resolution.rs`: prove bare shader
  lookup prefers the caller's mangled library scope and `lib:` lookup does not
  fall back to a same-named host shader.
- `crates/nuxie-scripting/tests/host_logging.rs`: use flat source-module API.
- `crates/nuxie-scripting/src/vm/view_model.rs`: adapt the blob lookup surface
  test to register the S4 full and short prelinked names; prove caller-scoped
  selection returns the right payload and authored short `.name` (the initial
  surface adaptation was found during S4-17 verification).
- `crates/nuxie/src/lib.rs`: remove imported scope/library records and register
  script assets by exported names; register blob/shader full and short names;
  keep successfully registered modules and protocols when unrelated bare
  dependencies remain unresolved, matching C++ `performRegistration`; make the
  scope-probe contract test hermetic against `fixtures/sync/scope_probe.riv`.
- `tools/rust-golden-runner/src/main.rs`: same flat prelinked registration
  model; mirror C++ file registration by retaining a partially registered VM
  when unrelated utility/protocol dependencies remain unresolved.
- `tools/cpp-probe/main.cpp`: compile against both the frozen pre-S4 pin and
  the S4 runtime after removal of uint64 scope fields.
- `file-correspondence-manifest.toml`: compatibility note on the pinned
  `core_uint64_type.cpp` row; row removal is deferred to pin advance.
- `tools/fetch-test-assets.sh` and force-added
  `fixtures/sync/scope_probe.riv`: exact S4-3 fixture/checksum. Fetch vendored
  sync assets from each row's recorded Git object rather than copying whatever
  version happens to be checked out in `RIVE_RUNTIME_DIR`.
- `S4B-report.md`: S4-3 status row and common notes.

Verification: exact `make schema` at `b73bc675`; schema/codegen tests green;
candidate `scope_probe` produces the same 745-byte stream under Rust and exact
candidate C++; its production module contract yields `(lib, hasDecode, cached)
= (1, 1, 1)` with no root bare leak; failed-module parent-stack and scoped
blob/shader regressions green; required runtime, scripting, frame-loop, and
Rust-attribution gates green.

## 2. S4-4 — `ba2b6434`

Commit message:

```text
[sync] Port rive-runtime ba2b6434: Allow unsetting global view models
```

Owned files/hunks:

- `crates/nuxie-runtime/src/viewmodel/viewmodel.rs`: add
  `unset_global_named`, preserving successful empty-slot clears.
- `crates/nuxie-runtime/src/data_bind/data_context.rs`: clear a named global
  from both the view-model context and its unusual-slot relay, then resync.
- `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`:
  accept optional global instances; validate names before allocating a primary
  context; make a valid clear on an unbound machine succeed; ensure `bind()`
  creates the empty context needed to complete authored main/global defaults;
  extend the focused lifecycle regression.
- `crates/nuxie-runtime/src/artboard.rs`: add the matching optional global
  setter/getter seam with the same validation and no-allocation clear behavior;
  cover set/get/clear in the focused regression.
- `S4B-report.md`: S4-4 status row.

Verification: focused global set/clear and empty-context bind regression green;
required runtime, scripting, frame-loop, and Rust-attribution gates green.

## 3. S4-10 — `e85a1160`

Commit message:

```text
[sync] Port rive-runtime e85a1160: Bidirectional binding for stateful component properties
```

Owned files/hunks:

- `crates/nuxie-runtime/src/data_bind/context/context_value.rs`: retain each
  bidirectional occurrence's last concrete target value; refresh that cache
  after source-to-target application; filter unchanged target notifications;
  add the return-to-the-pre-write-value regression.
- `tools/fetch-test-assets.sh` and force-added
  `fixtures/sync/bidirectional_stateful_property.riv`: exact upstream fixture
  and checksum.
- `.s4-deferred-corpus.toml`: append only the generated corpus metadata for
  the new fixture; do not enroll it in the active corpus before pin advance.
- `S4B-report.md`: S4-10 status and the unrelated flaky-test retry note.

Verification: focused target-cache regression green. The first runtime-suite
pass hit the existing `root_artboard_advance_polls_global_async_work_before_advancing`
thread-affinity flake; its focused retry and the complete rerun passed. The
scripting, frame-loop, and Rust-attribution gates also passed.

## 4. S4-14 — `e62b2bf3`

Commit message:

```text
[sync] Port rive-runtime e62b2bf3: Guard scripted data converter against unhydrated instance
```

Owned files/hunks:

- `crates/nuxie-runtime/src/scripted_data_converter.rs`: attribute the
  already-present `script_lifetime_valid`/`m_self == 0` pass-through guard by
  naming its regression for the unhydrated-self case and tightening the
  assertion text. No duplicate behavior was added.
- `S4B-report.md`: S4-14 status row.

Verification: focused unhydrated-self conversion/advance regression and all
four required gates green.

## 5. S4-17 — `8af27351`

Commit message:

```text
[sync] Port rive-runtime 8af27351: Nested view model properties mint the referenced type from scripts
```

Owned files/hunks:

- `crates/nuxie-scripting/src/vm/view_model.rs`: add a Luau regression proving
  a nested property wrapper's `instance()` uses the dynamically referenced
  child schema rather than the owning schema. The production implementation
  already derives `view_model_index` from the linked child on every property
  read, which also covers post-relink swaps; no parallel cached type was added.
- `S4B-report.md`: S4-17 status row.

Verification: focused nuxie-scripting referenced-type minting regression and
all four required gates green.

## 6. S4-18 — `0a2e478a`

Commit message:

```text
[sync] Port rive-runtime 0a2e478a: Propagate opacity and layout to paused nested artboards
```

Owned files/hunks:

- `crates/nuxie-runtime/src/artboard.rs`: during nested-host update, recurse
  into a paused mounted child when the child's own Components dirt is pending,
  even when the host only carries RenderOpacity dirt; add a synthetic paused
  opacity propagation/cleanliness regression.
- `tools/fetch-test-assets.sh` and force-added
  `fixtures/sync/paused_nested_artboard_opacity.riv`: exact upstream fixture
  and checksum.
- `.s4-deferred-corpus.toml`: append only the generated metadata for the new
  fixture, leaving active enrollment to pin advance.
- `S4B-report.md`: S4-18 status row.

Verification: focused paused-child opacity regression and all four required
gates green.

## 7. S4-22 — `b1d97fe7`

Commit message:

```text
[sync] Port rive-runtime b1d97fe7: Host-bound view models bind through riveLuaPushArtboard
```

Owned files/hunks:

- `crates/nuxie/src/lib.rs`: add a regression proving a cloned
  `FileScriptArtboard` keeps the exact supplied host-bound view-model identity
  and retains its `Arc<File>` after both the source userdata and external host
  owner are dropped. Production already stores the non-cyclic host file on
  every `FileScriptArtboard` and clones it through `instance()`.
- `S4B-report.md`: S4-22 status row.

Verification: focused host-file/bound-model lifetime regression and all four
required gates green.

## 8. S4-24 — `fc995413`

Commit message:

```text
[sync] Port rive-runtime fc995413: Font data binding from Luau scripts
```

Owned files/hunks:

- `crates/nuxie-runtime/src/scripting.rs` and `crates/nuxie-runtime/src/lib.rs`:
  add the backend-neutral retained `ScriptFont` identity, font property kind,
  file/live font lookup, and exact-Arc assignment seam.
- Force-add `crates/nuxie-scripting/src/vm/lua_font.rs`: add the opaque Lua
  Font userdata and file-owner registry; userdata retains the resolved Arc.
- `crates/nuxie-scripting/src/vm.rs` and
  `crates/nuxie-scripting/src/vm/view_model.rs`: install font owners, expose
  `getFont` and direct `Property<Font>` wrappers, and cover exact owner
  lifetime across registry replacement and assignment.
- `crates/nuxie/src/lib.rs`: attach each prepared file's font owners to the
  scripting VM, mirroring the existing image-owner handoff.
- `crates/nuxie-runtime/src/data_bind/context/context_value_asset_font.rs` and
  `crates/nuxie-runtime/src/data_bind/data_bind_context.rs`: guard TextStyle
  replacement when the source asset has no resolved font; update the existing
  live/file font regression for authored/effective font preservation.
- `file-correspondence-manifest.toml`: classify the new direct Lua font owner
  under upstream `src/lua/lua_properties.cpp`.
- `S4B-report.md`: S4-24 status and known flaky-test retry note.

Verification: focused Lua getFont/direct-property ownership regression and
font-binding hydration guard regression green; required runtime, scripting,
frame-loop, and Rust-attribution gates green. The first runtime-suite pass hit
the same pre-existing thread-affinity flake as S4-10; the complete retry passed.

## 9. S4-29 — `38c92412`

Commit message:

```text
[sync] Port rive-runtime 38c92412: Skip non-solo-set children during solo data binding
```

Owned files/hunks:

- `crates/nuxie-runtime/src/solo.rs`: centralize Solo membership and exclude
  Constraint, ClippingShape, FocusData, and SemanticData from index/name
  selection and active collapse participation.
- `crates/nuxie-runtime/src/artboard.rs`: add a synthetic interleaved-child
  regression for index, name, out-of-range, and collapse behavior; add its
  string-property fixture helper.
- `tools/fetch-test-assets.sh` and force-added
  `fixtures/sync/solo_index_test.riv`: exact upstream fixture and checksum.
- `.s4-deferred-corpus.toml`: append only the generated type-key metadata for
  `solo_index_test`; active enrollment remains deferred to pin advance.
- `S4B-report.md`: S4-29 status row.

Verification: focused Solo membership regression and all four required gates
green. The definitive gate run was made with the following S4-30 delta absent,
preserving the requested upstream row boundary.

## 10. S4-30 — `e06c3583`

Commit message:

```text
[sync] Port rive-runtime e06c3583: Remove inner feather when converting fill to stroke
```

Owned files/hunks:

- `crates/nuxie-runtime/src/shapes/paint/feather.rs`: add the upstream
  effective-inner predicate (serialized `inner` plus Fill parent), use it for
  Feather update dirt, and add the focused fill/stroke/null-parent regression.
- `crates/nuxie-runtime/src/draw.rs`: normalize retained Feather state through
  the effective-inner predicate for both live owners and prepared commands, so
  every existing draw/hit branch observes outward feathering on Stroke.
- `S4B-report.md`: S4-30 status row.

Verification: focused effective-inner regression and all four required gates
green. No golden was moved. The end-of-set comparison confirmed this is the
sole corpus delta: Rust emits the new outward-stroke `drawPath` while the frozen
pre-change C++ oracle still emits `clipPath` at line 1340.

## 11. S4-35 — `353ef4fc`

Commit message:

```text
[sync] Port rive-runtime 353ef4fc: Propagate live image/font into nested exposed asset properties
```

Owned files/hunks:

- `crates/nuxie-runtime/src/data_bind/data_bind_context.rs`: retain live image
  and font companions on exposed-property bindings, schedule identity-only
  changes even when the serialized sentinel is unchanged, and apply the exact
  owner into stateful nested view-model contexts. Admit asset graph values only
  for exposed `ViewModelInstanceAssetImage/Font` uint targets so direct
  Image/TextStyle adapters remain single-owned.
- `tools/fetch-test-assets.sh` and force-added
  `fixtures/sync/stateful_component_image_test.riv`: exact upstream fixture and
  checksum.
- `.s4-deferred-corpus.toml`: append only the fixture's generated type-key
  metadata; active enrollment remains deferred to pin advance.
- `S4B-report.md`: S4-35 status row.

Verification: exact fixture regression proves the nested context retains the
source `Rc<dyn RenderImage>` by pointer identity; existing direct live-font
binding regression remains green; all four required gates green.

## 12. S4-46 — `f7c84546`

Commit message:

```text
[sync] Port rive-runtime f7c84546: stable identity for asset Property.value reads
```

Owned files/hunks:

- `crates/nuxie-runtime/src/scripting.rs`: add a synchronous retained-cell
  change observer for scripted property owners, mirroring upstream
  `ScriptedProperty::valueChanged()` cache release at the mutation boundary.
- `crates/nuxie-scripting/src/vm/view_model.rs`: cache image/font value
  userdata per property wrapper, return the same registry identity across
  unchanged reads, and clear it on the exact backing-cell notification.
  Extend the existing image/font ownership tests with stable-read,
  host-replacement, and same-owner no-op assertions.
- `file-correspondence-manifest.toml`: include runtime `scripting.rs` in the
  `lua_properties.cpp` regrouping and record that the frozen pinned schema has
  no Blob view-model property type; the umbrella row remains pending rather
  than claiming unavailable coverage.
- `S4B-report.md`: S4-46 status row.

Verification: focused image/font identity and lifetime regressions and all four
required gates green. No schema, product/runtime pin, or Luau engine pin moved.

## End-of-set verification

- Final `cargo test -p nuxie-runtime`: green after one unrelated
  `root_artboard_advance_polls_global_async_work_before_advancing` parallel-test
  flake; its focused retry and the complete rerun passed.
- Final `cargo test -p nuxie --features scripting`: green.
- Final `make runtime-frame-loop-port-check`: green.
- Final `make rust-attribution-check`: green.
- Full `make scripted-golden-compare`: completed all 321 exact entries and 657
  segments; the only failure is the triage-attributed S4-30
  `echo_show_demo` delta against the frozen C++ pin. No golden was moved.
