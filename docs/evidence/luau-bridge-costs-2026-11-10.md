# Luau bridge costs (2026-11-10)

Three costs in the Luau host bridge that the workload characterization
(`claude/luau-workload-characterization-5391cc`, commit `ab20732f95`) named,
fixed without changing what scripts observe.

## Outcome

Pooled over the 16 steady workloads, script instructions per frame fall from
19.59M to 16.42M (−16%). Almost all of it is in the two workloads that call
host methods in loops: `script_layout_test` −45%, `script_path_effects_test`
−14%. The other 14 workloads move within ±2%, which is run-to-run noise in
this build, except the view-model workloads (−4%, −6%).

| Fix | Change | Effect |
|---|---|---|
| Fix 1 | Method lookup inside the VM | Path, the vector value metatable and PropertyList answered `__index` in a Rust closure, so `path:lineTo(...)` crossed into the host twice. An embedded Luau dispatcher now answers method names from the method table and passes every other key to the unchanged Rust closure. Lookup costs about 800 instructions instead of about 4,400. |
| Fix 2 | Vector constructors in the VM | `Vector.xy`, `Vector.xyz`, `Vector.origin` and `Vector(...)` call the native `vector.create` when every argument is a number, and the unchanged Rust function otherwise. About 1,000 instructions instead of about 2,600. |
| Fix 3 | Metatables patched once per VM | Path, PathCommand, PropertyList and enum value lists fetched their shared metatable through `exec_raw` (one C closure per call) for every value created. Enum value lists also installed a new `__index` closure each time. This now happens on the first value per VM. |

The `workload-stats` counters and the `run.py` driver that produced these
numbers are not in this change. They live on the local branch
`claude/luau-bridge-costs-0bad08` (measurement commit `3264428bde`, taken
from `ab20732f95` on `claude/luau-workload-characterization-5391cc`).

## Method

- Build: release with `CARGO_PROFILE_RELEASE_LTO=off` and
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`, the same non-LTO build the
  characterization used. Cargo was used only for these measurement builds.
- Runs: `spikes/luau-workload-characterization/run.py --kinds base,time
  --variants idle --frames 3000` over the 16 steady workloads, one run per
  stage. Counts are retired instructions from the `time` mode.
- Metrics:
  - script instructions per frame: the sum of every script category;
  - instructions per host call: host body plus marshalling, divided by host
    calls;
  - instructions per callback entry: `rt_call + bridge`, divided by
    callback invocations.
- The baseline reproduces the characterization: 19.59M pooled against 19.5M.
- Noise: workloads the fixes cannot reach move up to ±2% between runs.

### Script instructions per frame

| Workload | Before | Fix 1 | Fix 2 | Fix 3 | Change |
|---|---:|---:|---:|---:|---:|
| script_path_effects_test | 12660.9k | 11856.0k | 10974.7k | 10897.4k | -14% |
| script_layout_test | 3113.3k | 1987.0k | 1708.1k | 1706.8k | -45% |
| script_artboard_opacity_test | 1523.8k | 1522.6k | 1538.3k | 1536.2k | +1% |
| script_artboard_origin_test | 1013.9k | 1011.6k | 1021.4k | 1019.7k | +1% |
| script_artboard_test | 259.8k | 260.4k | 263.8k | 261.9k | +1% |
| data_bind_artboard_input | 245.6k | 247.4k | 245.9k | 242.6k | -1% |
| viewmodel_from_context | 234.7k | 224.0k | 226.2k | 225.2k | -4% |
| viewmodel_access | 205.5k | 204.7k | 208.6k | 192.7k | -6% |
| script_paths_test | 93.3k | 94.5k | 95.8k | 94.9k | +2% |
| script_paths_opacity_test | 83.4k | 82.1k | 84.0k | 82.6k | -1% |
| scripting_linear_animation | 70.0k | 70.7k | 70.6k | 70.7k | +1% |
| script_affects_has_changed | 29.9k | 30.3k | 29.2k | 29.0k | -3% |
| editor_scripted_vector_v7 | 28.1k | 28.2k | 28.9k | 28.5k | +2% |
| script_inputs_test_1 | 11.7k | 11.9k | 12.2k | 11.9k | +1% |
| script_create_viewmodel_instance | 10.4k | 10.6k | 10.9k | 10.6k | +2% |
| group_effect | 5.4k | 5.3k | 5.8k | 5.3k | -1% |

### Host calls per frame

| Workload | Before | Fix 1 | Fix 2 | Fix 3 | Change |
|---|---:|---:|---:|---:|---:|
| script_path_effects_test | 2082.8 | 1881.7 | 1308.5 | 1304.5 | -37% |
| script_layout_test | 709.1 | 456.1 | 256.1 | 256.1 | -64% |
| script_artboard_opacity_test | 184.1 | 184.1 | 184.1 | 184.1 | +0% |
| script_artboard_origin_test | 122.1 | 122.1 | 122.1 | 122.1 | +0% |
| script_artboard_test | 47.1 | 47.1 | 47.1 | 47.1 | +0% |
| data_bind_artboard_input | 3.1 | 3.1 | 3.1 | 3.1 | +0% |
| viewmodel_from_context | 27.1 | 25.1 | 25.1 | 25.1 | -7% |
| viewmodel_access | 30.1 | 30.1 | 30.1 | 28.1 | -7% |
| script_paths_test | 18.2 | 18.2 | 18.1 | 18.1 | -0% |
| script_paths_opacity_test | 13.2 | 13.2 | 13.1 | 13.1 | -0% |
| scripting_linear_animation | 5.1 | 5.1 | 5.1 | 5.1 | +0% |
| script_affects_has_changed | 4.1 | 4.1 | 3.1 | 3.1 | -24% |
| editor_scripted_vector_v7 | 5.1 | 5.1 | 5.1 | 5.1 | +0% |
| script_inputs_test_1 | 1.1 | 1.1 | 1.1 | 1.1 | +0% |
| script_create_viewmodel_instance | 0.2 | 0.2 | 0.2 | 0.2 | +3% |
| group_effect | 0.2 | 0.2 | 0.2 | 0.2 | -6% |

### Instructions per host call (marshalling + body)

| Workload | Before | Fix 1 | Fix 2 | Fix 3 | Change |
|---|---:|---:|---:|---:|---:|
| script_path_effects_test | 4021 | 3923 | 4455 | 4432 | +10% |
| script_layout_test | 3630 | 2723 | 2825 | 2821 | -22% |
| script_artboard_opacity_test | 6636 | 6630 | 6690 | 6679 | +1% |
| script_artboard_origin_test | 6672 | 6662 | 6723 | 6708 | +1% |
| script_artboard_test | 4066 | 4072 | 4118 | 4091 | +1% |
| data_bind_artboard_input | 65049 | 65399 | 65857 | 65422 | +1% |
| viewmodel_from_context | 6927 | 7184 | 7266 | 7230 | +4% |
| viewmodel_access | 5423 | 5403 | 5508 | 5387 | -1% |
| script_paths_test | 4025 | 4081 | 4135 | 4099 | +2% |
| script_paths_opacity_test | 4497 | 4412 | 4529 | 4443 | -1% |
| scripting_linear_animation | 8553 | 8648 | 8597 | 8657 | +1% |
| script_affects_has_changed | 4693 | 4749 | 5494 | 5427 | +16% |
| editor_scripted_vector_v7 | 3574 | 3581 | 3663 | 3623 | +1% |
| script_inputs_test_1 | 4885 | 4922 | 4939 | 4889 | +0% |
| script_create_viewmodel_instance | 3487 | 3521 | 3442 | 3523 | +1% |
| group_effect | 4808 | 4804 | 4835 | 4578 | -5% |

### Instructions per callback entry (rt_call + bridge)

| Workload | Before | Fix 1 | Fix 2 | Fix 3 | Change |
|---|---:|---:|---:|---:|---:|
| script_path_effects_test | 7096 | 7452 | 7465 | 7288 | +3% |
| script_layout_test | 4073 | 3974 | 4068 | 4037 | -1% |
| script_artboard_opacity_test | 4007 | 4057 | 4124 | 4142 | +3% |
| script_artboard_origin_test | 4041 | 4078 | 4182 | 4162 | +3% |
| script_artboard_test | 4509 | 4638 | 4812 | 4667 | +3% |
| data_bind_artboard_input | 4578 | 4590 | 4782 | 4621 | +1% |
| viewmodel_from_context | 6752 | 4477 | 4426 | 4419 | -35% |
| viewmodel_access | 3997 | 4007 | 4106 | 4056 | +1% |
| script_paths_test | 4557 | 4625 | 4716 | 4656 | +2% |
| script_paths_opacity_test | 4612 | 4572 | 4750 | 4616 | +0% |
| scripting_linear_animation | 4177 | 4209 | 4242 | 4186 | +0% |
| script_affects_has_changed | 4565 | 4657 | 4719 | 4646 | +2% |
| editor_scripted_vector_v7 | 4652 | 4692 | 4872 | 4656 | +0% |
| script_inputs_test_1 | 4521 | 4612 | 4773 | 4563 | +1% |
| script_create_viewmodel_instance | 3954 | 4038 | 4159 | 4020 | +2% |
| group_effect | 3499 | 3454 | 3818 | 3507 | +0% |

### Closures allocated per frame

| Workload | Before | Fix 1 | Fix 2 | Fix 3 | Change |
|---|---:|---:|---:|---:|---:|
| script_path_effects_test | 4.22 | 4.22 | 4.23 | 0.23 | -95% |
| script_layout_test | 0.21 | 0.22 | 0.22 | 0.22 | +3% |
| script_artboard_opacity_test | 0.23 | 0.23 | 0.23 | 0.23 | +3% |
| script_artboard_origin_test | 0.22 | 0.23 | 0.23 | 0.23 | +3% |
| script_artboard_test | 0.23 | 0.23 | 0.24 | 0.24 | +3% |
| data_bind_artboard_input | 0.20 | 0.20 | 0.21 | 0.21 | +4% |
| viewmodel_from_context | 36.20 | 35.21 | 35.21 | 35.21 | -3% |
| viewmodel_access | 13.22 | 13.22 | 13.22 | 10.23 | -23% |
| script_paths_test | 0.23 | 0.23 | 0.24 | 0.24 | +2% |
| script_paths_opacity_test | 0.21 | 0.21 | 0.22 | 0.21 | +2% |
| scripting_linear_animation | 0.21 | 0.22 | 0.22 | 0.22 | +4% |
| script_affects_has_changed | 0.22 | 0.23 | 0.23 | 0.23 | +3% |
| editor_scripted_vector_v7 | 0.22 | 0.23 | 0.23 | 0.23 | +4% |
| script_inputs_test_1 | 0.21 | 0.21 | 0.22 | 0.22 | +4% |
| script_create_viewmodel_instance | 0.36 | 0.37 | 0.37 | 0.37 | +2% |
| group_effect | 0.23 | 0.23 | 0.24 | 0.23 | +2% |

Pooled script instructions per frame: 19590k → 17647k → 16524k → 16416k (-16.2%)
Median per-workload change: -0.2%

Instructions per host call rise in some workloads after fix 2. This is a mix
shift: the cheap `Vector.xy` calls left the host, so the average of the calls
that remain went up. Per-function costs did not change.

Callback entry in `viewmodel_from_context` drops 35% after fix 1, because
PropertyList lookups no longer re-enter Lua through a Rust closure.

## What changed against the task

- **Fix 3's stated target has no production caller.**
  `create_scoped_userdata` is reached only through `Lua::scope` →
  `Scope::create_userdata`, which nothing in nuxie-scripting calls. The
  renderer passed to `draw` already uses luaur-rt's cached per-type metatable
  (`lua.create_userdata`). Caching scoped metatables per type would also need
  a type key for non-`'static` types, which needs `unsafe`. The closures the
  characterization measured come from two other places:
  - per-value `exec_raw` metatable lookups (fixed: 4.2 → 0.2 closures per
    frame in `script_path_effects_test`);
  - view-model tables (not fixed, see below).
- **Mat2D constructors are still host calls.** A Mat2D is userdata, so a host
  call must remain. Most of its marshalling cost is luaur-rt taking a
  registry reference for the new userdata and dropping it again after
  pushing. Removing that needs a new luaur-rt return path, which means new
  `unsafe`. In the corpus this is one call per artboard draw, about 8% of
  `script_artboard_opacity_test`.
- **Not changed: `Vector` methods.** `Vector.length`, `dot`, `lerp` and the
  rest stay Rust calls, and so does instance syntax after fix 1's lookup.
  Luaur has Rive vector builtins, but the corpus never calls these methods
  in a loop. Matching the Rust `mul_add` arithmetic bit for bit would need
  its own check.
- **Not changed: field-bearing userdata** (for example ContourMeasure,
  `positionAndTangent` and `extract`). These already resolve methods through
  luaur-rt's Lua dispatcher chunk, with no host crossing. The ~800
  instructions per lookup that remain are the cost of the VM calling a Lua
  `__index` function. Going lower needs `__namecall`, which needs the namecall
  atom from Rust and therefore `unsafe`.
- **Not changed: other Rust `__index` sites** (RiveFile, Font,
  TransitionChild, ScrollEvent, view-model tables). They serve field data
  computed in Rust, and none is hot in the corpus.

## Not fixed: view-model closures

`viewmodel_from_context` still allocates 35 closures per frame and
`viewmodel_access` 10. Every view-model table gets 15 new Rust closures
(`getNumber`, `getColor`, and so on, each capturing its model). The property
patcher also creates a wrapper closure on every lookup of a mutating method.
Sharing these would make `vmA.getNumber == vmB.getNumber` true where it is
false today, and calling one with another table as `self` would read the
other model. Both are visible to scripts, so this needs Levi's decision.

## Verification

The gates first ran on 2026-11-11 with the fixes on base `6cc1a07141`. They
ran again after the rebase onto `origin/main` `974f2276ec`. Neither tree
had the measurement commit, so the ratchets count only what lands.

| Gate | On `6cc1a07141` | On `974f2276ec` |
| --- | --- | --- |
| `make ratchet-unsafe` | pass. nuxie-scripting 336 sites (base 338: the four embedded-chunk loads now share one loader); every vendored luaur crate +0 | pass. Main now counts only the six vendored luaur crates; all are +0 |
| `make ratchet-panic-lints` | pass | pass |
| `make luau-conformance` | pass. 28 of 55 scripts OK; all 28 required scripts pass; 0 expected-fail scripts pass | pass, same counts |
| `runtime.py test -p nuxie-scripting` (40 Bazel targets) | 40 pass | 40 pass |
| `make scripted-golden-compare` (C++ runner at pin `cd5351c3`, release) | 364 entries: 348 exact, 16 diverges with verified signatures, 1 failure (see below) | not rerun: the other gates stayed green, and the rebase touched no scripting files |

`test_scripted_layout` needs `RIVE_RUNTIME_DIR` set to the pinned
rive-runtime checkout for its fixture. Without it, the test fails on the
missing variable.

The one golden failure is `semantic_text_inference_lt1`, which reports
"semantic comparison is now exact; remove the reviewed signature
explicitly". Base `6cc1a07141` fails the same way, with the same counts
(348 exact, 16 diverges, 1 failure). This entry is a known red on main and
does not use scripting.

With the `compiler` feature off (`--no-default-features --features luau`),
the unit target builds and passes. That configuration still fails to build
the integration test `tests/upstream_scripting_vector.rs`, which is
unchanged from main.

Tests added for the fixes (in the `nuxie-scripting` unit target):

- Fix 1, `vm::host_dispatch::method_dispatch_tests`:
  - Method names are answered in the VM and never reach the Rust fallback.
    Numeric keys, unknown keys and keys that raise errors all reach the
    fallback.
  - On a real Path, `__index` is the VM dispatcher, and `#p`, `p[1].type`,
    `p[2][1]`, `p[3.0]`, `p[0]` and `p[99]` read as before. `p[true]` still
    raises "Path index must be a string or number".
  - On Vector, `v.length` is `Vector.length`, `v:length()` works, `v[1]`
    and `v["2"]` work, and unknown fields still raise.
- Fix 3:
  - `vm::host_dispatch::patch_once_tests`: across three VMs, the patch runs
    once per VM, not once per call. A failed patch is retried. Every Path in
    a VM shares one `__index`. Each new VM patches again, and each new VM's
    Path `__index` is the VM dispatcher. luaur-rt clears a VM's metatable
    cache only in its drop, which clears that VM's app data in the same
    step. A new VM, possibly at the same state address, is therefore the
    only way to reach a cleared cache.
  - `vm::view_model::enum_values_metatable_tests`: enum-value lists keep one
    shared `__index`, and they still index correctly.
- Fix 2, `vm_constructors_match_the_rust_constructors`:
  - It compares 23 constructor calls between the new constructors and the
    Rust-only ones, including `-0`, NaN, overflow, strings, booleans, and
    missing and extra arguments.
  - It caught a `-0` divergence in the first version of fix 2: the Rust path
    reads integral numbers through `i64`, which turns `-0` into `0`. That
    behavior is now matched.

All 16 measurement workloads ran to exit 0 at every stage.

## Not verified

- No fat-LTO numbers.
- No pointer variant.
- No detail-mode (per-opcode) runs.
- The scripted golden lane shares the base's `semantic_text_inference_lt1`
  red. The remaining 363 entries match the base exactly; the failing entry
  itself is not a scripted fixture.
