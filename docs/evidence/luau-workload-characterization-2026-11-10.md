# Luau workload characterization (2026-11-10)

Measurement only. No VM was built or changed beyond off-by-default counters.

## Outcome

Most of the time real scripts spend is not in bytecode. It is the boundary
between Luau and the Rust host.

Across the 16 workloads that run script every frame, the script share of the
frame is pooled as follows. All counts are native instructions, which do not
depend on machine load.

| Where the script time goes | Pooled | Median workload |
|---|---:|---:|
| Boundary overhead: entering a callback, trampoline argument and result marshalling, runtime dispatcher chunks | 41% | 43% |
| Bodies of host functions (Rust) | 37% | 38% |
| Authored bytecode | 21% | 14% |
| GC | 0.5% | 1.3% |

Even the "authored bytecode" slice is mostly host-call work. These scripts
retire 150–500 native instructions per bytecode op. The upstream `tests/`
benchmarks retire 41–47 per op on the same build. The difference is the
VM-side half of host calls: NAMECALL lookup, `__index` metamethod calls, and
C-call frame setup. The scripts make one host call every ~10 bytecode ops,
against fewer than 2 per 1000 ops on most benchmarks.

### Verdicts on the four candidates

| # | Candidate | Measured upper bound on this corpus | Verdict |
|---|---|---|---|
| 2 | Host boundary as data | Boundary overhead is 41% of script instructions. VM-side call work inside the bytecode slice adds roughly another 10–15 points (estimated; see "Not verified"). Typical host methods cost 2–5k instructions per call, and about half of that is marshalling. Callback entry costs 3.4–7.3k instructions per invocation. | **Supported. This is the largest lever.** |
| 1 | Skip callbacks whose inputs are unchanged | 70% of callback instructions ran with inputs equal to the previous invocation (loose comparison, which compares userdata by type). 65% hold under the strict comparison (userdata by identity). The median workload is 91%. Outputs were identical to the previous invocation for 70% of instructions pooled, and 99.9% for the median workload. With the pointer sweep these figures are 72% / 68% / 66%. | **Supported, with a condition.** The bound is real. It needs host objects to publish change stamps, because loose equality cannot see whether a path's contents or an artboard's state changed. It is skewed by one workload: `script_path_effects_test` re-runs `update` with identical inputs and outputs, costing 12.5M instructions every frame. |
| 4 | Typed fast tier | Authored bytecode is at most 21% of script instructions (pooled), and much of that is call work, not arithmetic. Static types are absent: 0 of 159 parameters seen at run time carry bytecode type info, 0 of 145 parameters in product sources are annotated, and no file uses `--!strict`. Dynamically the sites are almost perfectly monomorphic: arithmetic is 81% number-only and 19% single vector/number; table and userdata field sites are 100% single-shape. | **Refuted as the primary lever. A speculative (type-feedback) tier would work, but its ceiling is low.** |
| 3 | Frame-scoped arena memory | GC is 0.5% of script instructions. There are 5.6 allocations per 1000 bytecode ops. 98.5% of allocations made inside a host-initiated callback die inside it. | **Refuted as a speed win (the ceiling is about 1%).** The lifetime pattern is real, so an arena is sound for memory hygiene. The allocations it would capture are mostly host-created: per-call scoped userdata (7.6 per frame) and their closures (3.5 per frame). These are an API-shape cost, not a GC cost. |

### Product scripts barely run per frame

None of the 87 product fixtures from nuxie-dev runs a script callback in the
steady state, either idle or under a synthetic pointer sweep. Their script
share is a median of 1.5% of the frame and at most 3.2%. All of it comes from
module setup. 25 of the 87 load scripts at all:

- `script_response_set`
- generated converters (`map`, `empty-text`, `enum-*`, `input-presence`)
- listener actions

These are event-driven work (`performAction`, `convert`) that a frame-loop
benchmark never exercises. All of the steady-state data below therefore comes
from Rive upstream scripted content (15 fixtures) plus one editor fixture.
That content is the best proxy available for what scripted drawing, path
effects, layout and artboard composition will look like in Nuxie. It is not
Nuxie product code.

### Also large, and covered by none of the four

- **Method lookup goes through Rust.** `Path` installs a Rust `__index`
  closure (`crates/nuxie-scripting/src/vm/lua_path.rs`) so that numeric
  indexing works. As a result every `path:lineTo(...)` makes two host
  crossings: one to find the method and one to call it. In
  `script_layout_test` that lookup is 759k of 1.36M host calls. Types with
  fields route through the Lua dispatcher chunk instead, which accounts for
  6% of pooled script instructions as "helper bytecode".
- **`Vector.xy` / `Mat2D.withTranslation` constructors are host calls.** Each
  costs about 4k instructions, although Luau has a native vector type.
  `Vector.xy` accounts for 600k host calls in `script_layout_test` and 1.7M
  in `script_path_effects_test`.
- **Scoped userdata rebuild their metatables per scope.**
  `create_scoped_userdata` creates a method table, a metatable and a closure
  per method every time a scoped value is made, for example the renderer
  passed to `draw`. This is the source of the 3.5 closures per frame.

## Method

### Instrumentation

The counters are behind cargo features that are off by default:

- `luaur-vm/workload-stats`, plus `-detail`
- forwarded by `luaur-rt`, `nuxie-scripting` and `rust-golden-runner`

The code is in `vendor/luaur-vm-0.1.8/src/stats.rs`, with hooks at:

- interpreter entry
- C-call sites (CALL, CALLFB, precall, metamethod calls)
- allocation, free and realloc
- `luaC_step`
- the luaur-rt `Function::call` and trampoline
- the golden runner's phase marks

Two modes, two binaries:

- **time** counts retired instructions per thread with
  `thread_selfcounts`. Each hook reads the counter first. The read cost
  (calibrated as the minimum of 200 back-to-back reads, about 1686
  instructions) is subtracted for each segment. Attribution is exclusive,
  using a category stack. There is no per-instruction hook in this binary.
  On `script_paths_test` the attributed total is 397M against 420M for the
  uninstrumented runner.
- **detail** adds a per-instruction hook. It records:
  - opcode histograms and pairs;
  - per-site receiver shapes and type tags (up to 8 distinct keys per site);
  - which callee each call reaches;
  - userdata gets, sets and calls;
  - table shapes at death;
  - input and output hashes per invocation:
    - strict: userdata by identity;
    - loose: userdata by metatable;
    - outputs are host-call arguments plus writes to state that existed before the invocation;
  - deadness, measured with a forced full GC after each top-level callback.

  Only counts come from detail mode. Its instruction totals are biased by the
  hook and are never used.

### Categories

| Category | What it covers |
|---|---|
| bridge | `nuxie-scripting` `protected_call` |
| rt_call | luaur-rt `Function::call` minus the VM |
| bytecode | `luau_execute` in authored chunks |
| helper_bytecode | runtime-owned chunks whose source starts with `=`, such as the userdata dispatcher and `rive_promise` |
| host_rt_marshal | trampoline and argument collection |
| host_rt_body | the Rust callback body |
| host_lib | Luau builtins |
| host_other | everything else in the host |
| gc | GC work |

### Runs

- Each workload runs 3000 frames at 60 Hz through `rust-golden-runner --execute-scripts --benchmark`.
- Variants:
  - idle: the corpus's own input script, if any;
  - pointer: a synthetic sweep with a move every frame and a tap every 0.5 s. Rive fixtures use a 500×500 artboard; nuxie-dev fixtures use a phone-sized one.
- Three kinds of run per variant: base (uninstrumented), time, and detail. That is 882 runs in total.
- Build: release with `lto=off` and `codegen-units=16` (see "Not verified").
- Host load average was about 19–22 during the runs. Instruction counts are insensitive to it, and no wall-clock numbers are used.

### Tooling

- `spikes/luau-workload-characterization/run.py` drives the runs.
- `analyze.py` produces every table below.
- `static_types.py` produces the source annotation census.

## Corpus and its limits

| Source | Fixtures | Scripts every frame | Scripts only at setup or on events |
|---|---:|---:|---:|
| Rive upstream scripted content (`corpus.toml` entries with `Script*` features, pin checkout) | 58 | 15 | 6 |
| Editor fixtures in this repo | 2 | 1 | 0 |
| nuxie-dev product fixtures (iOS SDK runtime and journeys, editor crates; read-only) | 87 | 0 | 2 |

**Product source census.** This covers 40 unique `.luau` files in nuxie-dev,
1668 code lines:

- 0 of 145 parameters typed;
- 5 of 151 functions with a return annotation;
- 0 of 357 locals typed;
- 2 type declarations;
- no `--!strict`.

**What could not be obtained:**

- Product scripts under realistic events. The pointer sweep never hit a
  product listener, and no event-level harness drives `performAction` or
  `convert` with recorded inputs.
- The editor builtins `responsive-border.luau` and `presence-cursor.luau`
  in any runnable `.riv`. They never appeared among the run-time protos.
  Only `nuxie.generated.converter.map` did, at setup.
- Sources for the Rive fixtures. The `.riv` files embed bytecode only, so
  their typing comes from bytecode type info, which is empty.
- A `perf-scripts/` lane. None exists in this repo.

**Excluded from the steady-state means:**

- `viewmodel_from_context`. Its frame cost of 86M instructions is outside
  script. It is listed in the table but is an anomaly worth a separate look.
- The upstream `tests/` benchmarks, which appear only as a contrast column.

### Corpus by source (idle variant)

| Source | Fixtures run | With per-frame callbacks | With callbacks only at setup/events |
|---|---:|---:|---:|
| nuxie-dev | 87 | 0 | 2 |
| nuxie-editor-fixture | 2 | 1 | 0 |
| rive-upstream | 58 | 15 | 6 |

Pointer variant adds callbacks to 16 fixtures' runs; fixtures whose script runs only under pointer input: none.

### Per-workload, idle variant (fixtures with at least one steady callback every other frame)

| Workload | Frame instr (base) | Script instr/frame | Script share | Callbacks/frame | Bytecode instrs dispatched/frame | Host calls/frame | Bytecode % | Boundary % | Host body % | GC % | Allocs/frame | Bytes/frame | Dead in callback | Inputs unchanged (loose) | Outputs unchanged (loose) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| script_path_effects_test | 11807k | 12572.8k | 97% | 4.0 | 28014 | 2082.8 | 25% | 44% | 31% | 0.1% | 18.5 | 2305 | 100% of 6033 | 100% | 100% |
| script_layout_test | 2913k | 3078.8k | 95% | 2.0 | 2102 | 709.1 | 17% | 47% | 37% | 0.0% | 1.5 | 190 | 0% of 0 | 0% | 0% |
| script_artboard_opacity_test | 1554k | 1513.0k | 92% | 4.0 | 1280 | 184.1 | 14% | 26% | 59% | 0.6% | 32.5 | 2752 | 100% of 90102 | 0% | 0% |
| script_artboard_origin_test | 1093k | 1002.6k | 88% | 2.0 | 845 | 122.1 | 14% | 26% | 59% | 0.6% | 21.5 | 1790 | 100% of 60102 | 0% | 0% |
| script_artboard_test | 286k | 257.9k | 89% | 1.0 | 309 | 47.1 | 19% | 38% | 42% | 1.2% | 10.5 | 912 | 100% of 27112 | 100% | 100% |
| data_bind_artboard_input | 290k | 245.7k | 91% | 1.0 | 28 | 3.1 | 2% | 6% | 79% | 14.0% | 2.5 | 347 | 100% of 3000 | 100% | 100% |
| viewmodel_from_context | 86302k | 233.4k | 0% | 2.0 | 56 | 27.1 | 3% | 13% | 75% | 9.7% | 58.5 | 3620 | 100% of 171019 | 68% | 100% |
| viewmodel_access | 250k | 204.3k | 77% | 2.0 | 144 | 30.1 | 4% | 31% | 60% | 4.0% | 23.5 | 1397 | 91% of 69202 | 1% | 1% |
| script_paths_test | 139k | 94.0k | 75% | 1.0 | 62 | 18.2 | 15% | 46% | 37% | 1.1% | 3.5 | 353 | 99% of 6102 | 100% | 100% |
| script_paths_opacity_test | 130k | 83.1k | 73% | 1.0 | 79 | 13.2 | 23% | 42% | 34% | 0.5% | 1.5 | 191 | 0% of 0 | 100% | 100% |
| scripting_linear_animation | 125k | 69.5k | 65% | 3.0 | 58 | 5.1 | 12% | 39% | 48% | 1.4% | 2.5 | 349 | 0% of 0 | 82% | 82% |
| script_affects_has_changed | 73k | 29.5k | 55% | 1.0 | 23 | 4.1 | 14% | 44% | 40% | 2.8% | 2.5 | 269 | 98% of 3102 | 99% | 99% |
| editor_scripted_vector_v7 | 64k | 27.6k | 62% | 1.0 | 15 | 5.1 | 15% | 50% | 32% | 2.9% | 2.5 | 269 | 98% of 3102 | 100% | 100% |
| script_inputs_test_1 | 58k | 11.9k | 33% | 1.0 | 5 | 1.1 | 12% | 61% | 23% | 3.7% | 1.5 | 188 | 0% of 0 | 100% | 100% |
| script_create_viewmodel_instance | 196k | 10.3k | 6% | 2.0 | 3 | 0.2 | 12% | 77% | 5% | 5.3% | 1.7 | 204 | 0% of 0 | 84% | 100% |
| group_effect | 54k | 5.3k | 16% | 1.0 | 2 | 0.2 | 14% | 71% | 14% | 1.2% | 0.6 | 39 | 81% of 403 | 68% | 85% |

### Script cost by category, idle variant (16 workloads)

| Category | Pooled share (sum of per-frame instr) | Median across workloads |
|---|---:|---:|
| bridge | 0.2% | 1.4% |
| rt_call | 0.5% | 3.5% |
| bytecode | 21.3% | 14.1% |
| helper_bytecode | 6.0% | 1.4% |
| host_rt_body | 37.1% | 37.8% |
| host_rt_marshal | 34.2% | 21.6% |
| host_lib | 0.0% | 0.0% |
| host_other | 0.2% | 0.4% |
| gc | 0.5% | 1.3% |

### Script cost by category, pointer variant (16 workloads)

| Category | Pooled share (sum of per-frame instr) | Median across workloads |
|---|---:|---:|
| bridge | 0.2% | 1.4% |
| rt_call | 0.5% | 3.5% |
| bytecode | 21.5% | 13.7% |
| helper_bytecode | 6.2% | 1.4% |
| host_rt_body | 37.1% | 37.8% |
| host_rt_marshal | 33.8% | 21.0% |
| host_lib | 0.0% | 0.0% |
| host_other | 0.2% | 0.4% |
| gc | 0.5% | 1.2% |

### Candidate upper bounds, idle variant

| Measure | Pooled | Median workload |
|---|---:|---:|
| Callback invocations whose inputs were unchanged, strict (userdata by identity) | 32.9% of invocations | |
| Callback invocations whose inputs were unchanged, loose (userdata by type) | 64.5% of invocations | |
| Script instructions in invocations with unchanged inputs, strict | 64.5% | |
| Script instructions in invocations with unchanged inputs, loose | 69.7% | 91.1% |
| Script instructions in invocations whose outputs equal the previous invocation's, loose | 70.1% | 99.9% |
| Boundary overhead share (bridge + rt_call + trampoline marshalling + dispatcher bytecode) | 40.9% | |
| Host function bodies share | 37.3% | |
| Authored bytecode share | 21.3% | |
| GC share | 0.52% | |
| Allocations made in host-initiated callbacks that died inside them | 98.5% of 439279 | 99.7% |

### Candidate upper bounds, pointer variant

| Measure | Pooled | Median workload |
|---|---:|---:|
| Callback invocations whose inputs were unchanged, strict (userdata by identity) | 31.7% of invocations | |
| Callback invocations whose inputs were unchanged, loose (userdata by type) | 56.6% of invocations | |
| Script instructions in invocations with unchanged inputs, strict | 67.9% | |
| Script instructions in invocations with unchanged inputs, loose | 71.5% | 65.8% |
| Script instructions in invocations whose outputs equal the previous invocation's, loose | 72.4% | 92.2% |
| Boundary overhead share (bridge + rt_call + trampoline marshalling + dispatcher bytecode) | 40.7% | |
| Host function bodies share | 37.3% | |
| Authored bytecode share | 21.5% | |
| GC share | 0.49% | |
| Allocations made in host-initiated callbacks that died inside them | 98.4% of 491438 | 99.3% |

### Authored instruction sites (idle variant, executions)

| Measure | Value |
|---|---:|
| Field accesses (GETTABLE*/SETTABLE*/NAMECALL) | 12313411 |
| ... on tables | 25.7% (monomorphic shape: 100.0% of those; raw hit 98.0%, via __index 0.0%) |
| ... on userdata (host objects) | 46.3% (single type per site: 100.0%) |
| ... on vectors / strings / mixed receivers | 28.0% / 0.0% / 0.0% |
| ... on tables created by the same invocation | 0.0% |
| Constant-key sites (GETTABLEKS/SETTABLEKS/NAMECALL) with one receiver shape or type | 100.0% of 12247373 |
| Arithmetic executions | 17988435 |
| ... at sites that only ever saw numbers | 80.9% |
| ... at sites that only ever saw one vector/number combination | 19.1% |
| ... at sites with another single type combination | 0.0% |
| ... at sites with several combinations | 0.0% |
| Compare executions | 3283426 |
| ... at sites that only ever saw numbers | 96.8% |
| ... at sites that only ever saw one vector/number combination | 0.0% |
| ... at sites with another single type combination | 3.2% |
| ... at sites with several combinations | 0.0% |
| Calls from bytecode | 8366762; to C functions 98.1%, to Lua 1.9% |
| Call sites with one observed callee | 100.0% of call executions; Lua-only sites monomorphic: 100.0% |
| Arguments per bytecode call | 2.69 |
| NEWCLOSURE / DUPCLOSURE executions | 8 / 100 |

### Tables by allocation site (idle variant; shape = key set + metatable at death or at dump)

| Allocated from | Sites | Tables | Single-shape sites | Tables at single-shape sites | Empty | Array-like | Record-like | Mixed | With metatable | Born in callback | Died in callback |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| authored | 157 | 6416 | 81 | 1% | 2% | 0% | 98% | 0% | 47% | 6222 | 6061 |
| helper | 216 | 3986 | 148 | 79% | 5% | 0% | 95% | 0% | 3% | 3157 | 2999 |
| host/vm | 16 | 448 | 0 | 0% | 11% | 0% | 86% | 4% | 4% | 0 | 0 |

Largest authored table allocation sites:

| Workload | Function | Line | Tables | Shapes | Kind | Sample keys |
|---|---|---:|---:|---:|---|---|
| viewmodel_from_context | advance | 21 | 5998 | 4 | record | [0]{__index} |
| script_create_viewmodel_instance | (main) | 75 | 18 | 9 | record | [0]{} |
| script_create_viewmodel_instance | init | 22 | 14 | 8 | record | [0]{__index} |
| group_effect | init | 12 | 8 | 7 | record | [0]{} |
| viewmodel_access | advance | 15 | 6 | 3 | record | [0]{} |
| viewmodel_from_context | init | 7 | 6 | 3 | record | [0]{} |
| script_create_viewmodel_instance | init | 22 | 5 | 4 | record | [0]{} |
| data_bind_artboard_input | update | 18 | 4 | 3 | record | [0]{} |
| data_bind_artboard_input | (main) | 1 | 4 | 3 | record | [0]{} |
| data_bind_artboard_input | (main) | 28 | 4 | 4 | record | [0]{addToPath,advance,animation,bounds,draw,drawModulated,ga |
| editor_scripted_vector_v7 | (main) | 1 | 4 | 3 | record | [0]{} |
| editor_scripted_vector_v7 | init | 3 | 4 | 3 | record | [0]{} |
| editor_scripted_vector_v7 | init | 3 | 4 | 3 | record | [0]{__index,__newindex} |
| editor_scripted_vector_v7 | draw | 12 | 4 | 3 | record | [8]{tx,ty,xx,xy,yx,yy} |
| editor_scripted_vector_v7 | init | 3 | 4 | 3 | record | [0]{} |

### Opcode mix, authored chunks (idle variant; 76573189 dispatched; runtime helper chunks dispatched another 22505954, 22.7% of all)

| Opcode | Share | | Opcode pair | Share of authored |
|---|---:|---|---|---:|
| MOVE | 19.6% | | MOVE → MOVE | 6.6% |
| CALL | 10.9% | | MOVE → LOADB | 4.1% |
| GETTABLEKS | 8.5% | | JUMPBACK → JUMPIFNOTLT | 4.1% |
| ADD | 7.9% | | JUMPIFNOTLT → JUMPIFNOT | 4.1% |
| NAMECALL | 7.4% | | MOVE → NAMECALL | 3.7% |
| LOADB | 6.3% | | CALL → MOVE | 3.2% |
| MUL | 6.1% | | CALL → GETIMPORT | 3.1% |
| JUMPIFNOT | 5.0% | | MUL → ADD | 2.7% |
| JUMPIFNOTLT | 4.1% | | NAMECALL → CALL | 2.6% |
| JUMPBACK | 4.1% | | GETIMPORT → GETTABLEKS | 2.3% |
| MINUS | 3.8% | | GETTABLEKS → CALL | 2.3% |
| GETIMPORT | 3.3% | | MOVE → ADD | 2.3% |
| MULK | 2.3% | | ADD → MOVE | 2.2% |
| JUMP | 2.1% | | GETTABLEKS → MINUS | 2.2% |
| FASTCALL2 | 2.1% | | MINUS → GETTABLEKS | 2.2% |
| SUB | 1.5% | | MOVE → GETTABLEKS | 2.1% |
| DIVK | 1.5% | | LOADB → JUMP | 2.1% |
| FORNLOOP | 0.9% | | GETTABLEKS → LOADB | 2.1% |
| FORGLOOP | 0.5% | | JUMP → JUMPBACK | 2.1% |
| FASTCALL1 | 0.4% | | JUMPIFNOT → MOVE | 2.1% |

Opcode groups (authored chunks) versus the upstream `tests/` microbenchmarks (each bench weighted equally):

| Group | Nuxie/Rive corpus | Upstream benches |
|---|---:|---:|
| field access | 16.1% | 24.0% |
| calls/returns | 11.2% | 5.9% |
| loads/moves | 26.1% | 11.9% |
| globals/upvalues/imports | 3.6% | 6.3% |
| arithmetic | 23.5% | 28.8% |
| compare/branch | 15.6% | 10.0% |
| loops | 1.4% | 4.4% |
| builtin fastcalls | 2.5% | 8.5% |
| allocation (table/closure/concat) | 0.0% | 0.3% |

Upstream benches, per run (contrast column):

| Bench | Instructions dispatched | Host C calls per 1k instructions | Allocations per 1k instructions | Authored field accesses on tables, monomorphic |
|---|---:|---:|---:|---:|
| base64 | 96642986 | 1.76 | 0.53 | 80% |
| chess | 343229526 | 0.75 | 25.75 | 100% |
| life | 682843434 | 0.00 | 0.00 | 100% |
| matrixmult | 170924467 | 1.43 | 0.03 | 99% |
| mesh-normal-scalar | 185346090 | 0.00 | 6.38 | 98% |
| mesh-normal-vector | 98430354 | 0.00 | 2.45 | 97% |
| pcmmix | 70200820 | 5.70 | 0.01 | 100% |
| qsort | 629917611 | 6.60 | 0.38 | 100% |
| sha256 | 131606238 | 1.50 | 0.08 | 96% |
| sieve | 385388130 | 198.74 | 3.16 | 100% |
| tictactoe | 890398674 | 1.40 | 1.40 | 88% |
| vector-math | 84001078 | 0.00 | 0.01 | 100% |
| voxelgen | 460295905 | 22.37 | 0.78 | 43% |

Corpus density for the same columns: host C calls per 1k dispatched instructions 98.5, allocations per 1k 5.6.

### Host functions by instructions (idle variant, all steady workloads, time mode)

| Function | Calls | Workloads | Instr per call (marshal + body) | Share of host instr | Args/call | Results/call |
|---|---:|---:|---:|---:|---:|---:|
| `luaur-rt-callback` | 3925487 | 16 | 3754 | 35.3% | 2.0 | 1.0 |
| `ScriptedContourMeasure:positionAndTangent` | 1719573 | 1 | 4928 | 20.3% | 2.0 | 2.0 |
| `ScriptedContourMeasure:extract` | 1575525 | 1 | 4361 | 16.5% | 5.0 | 0.0 |
| `ScriptedArtboard:advance` | 156030 | 6 | 21033 | 7.9% | 2.0 | 1.0 |
| `ScriptedPath:cubicTo` | 573226 | 3 | 3220 | 4.4% | 4.0 | 0.0 |
| `ScriptedArtboard:draw` | 189000 | 6 | 7761 | 3.5% | 2.0 | 0.0 |
| `ScriptedPath:lineTo` | 450069 | 4 | 2562 | 2.8% | 2.0 | 0.0 |
| `ScriptedRenderer:transform` | 189000 | 6 | 3544 | 1.6% | 2.0 | 0.0 |
| `ScriptedRenderer:drawPath` | 81000 | 5 | 6661 | 1.3% | 3.0 | 0.0 |
| `ScriptedRenderer:restore` | 186000 | 5 | 2431 | 1.1% | 1.0 | 0.0 |
| `ScriptedRenderer:save` | 186000 | 5 | 2428 | 1.1% | 1.0 | 0.0 |
| `ScriptedPath:moveTo` | 153026 | 5 | 2801 | 1.0% | 2.0 | 0.0 |
| `ScriptedPath:close` | 153024 | 5 | 2309 | 0.8% | 1.0 | 0.0 |
| `ScriptedArtboard:instance` | 3071 | 6 | 114888 | 0.8% | 1.0 | 1.0 |
| `ScriptedPropertyEnum:values` | 2999 | 1 | 20671 | 0.1% | 1.0 | 1.0 |
| `ScriptedPath:reset` | 21004 | 2 | 2841 | 0.1% | 1.0 | 0.0 |
| `ScriptedPath:contours` | 6002 | 1 | 8941 | 0.1% | 1.0 | 1.0 |
| `ScriptedPropertyList:push` | 2999 | 1 | 17720 | 0.1% | 2.0 | 0.0 |
| `ScriptedContext:viewModel` | 3001 | 2 | 16779 | 0.1% | 1.0 | 1.0 |
| `luaur-rt-settable` | 54816 | 16 | 773 | 0.1% | 3.0 | 0.0 |
| `ScriptedAnimation:advance` | 5998 | 1 | 6110 | 0.1% | 2.0 | 1.0 |
| `ScriptedPropertyNumber.value=` | 5999 | 2 | 5768 | 0.1% | 2.0 | 0.0 |
| `luaur-rt-exec-raw` | 39030 | 16 | 829 | 0.1% | 0.4 | 1.0 |
| `ScriptedPropertyEnum.value=` | 2999 | 1 | 8989 | 0.1% | 2.0 | 0.0 |
| `ScriptedPath:add` | 6042 | 2 | 4425 | 0.1% | 2.0 | 0.0 |
| `ScriptedPropertyViewModel.value` | 2999 | 1 | 8002 | 0.1% | 1.0 | 1.0 |
| `ScriptedContourMeasure.next` | 6002 | 1 | 3223 | 0.0% | 1.0 | 1.0 |
| `ScriptedContourMeasure.length` | 6002 | 1 | 3129 | 0.0% | 1.0 | 1.0 |
| `luaur-rt-gettable` | 17995 | 2 | 815 | 0.0% | 2.0 | 1.0 |
| `ipairs` | 20999 | 4 | 631 | 0.0% | 1.0 | 3.0 |

Userdata operations from authored bytecode: 5698603 total; gets 0.5%, sets 0.2%, method calls 99.4%. Top 20:

| Operation | Kind | Count |
|---|---|---:|
| `ScriptedContourMeasure.positionAndTangent` | call | 1719573 |
| `ScriptedContourMeasure.extract` | call | 1575525 |
| `ScriptedPath.cubicTo` | call | 573226 |
| `ScriptedPath.lineTo` | call | 450069 |
| `ScriptedArtboard.draw` | call | 189000 |
| `ScriptedRenderer.transform` | call | 189000 |
| `ScriptedRenderer.save` | call | 186000 |
| `ScriptedRenderer.restore` | call | 186000 |
| `ScriptedArtboard.advance` | call | 156030 |
| `ScriptedPath.moveTo` | call | 153026 |
| `ScriptedPath.close` | call | 153024 |
| `ScriptedRenderer.drawPath` | call | 81000 |
| `ScriptedPath.reset` | call | 21004 |
| `ScriptedPath.add` | call | 6042 |
| `ScriptedContourMeasure.next` | get | 6002 |
| `ScriptedPath.contours` | call | 6002 |
| `ScriptedContourMeasure.length` | get | 6002 |
| `ScriptedPropertyNumber.value` | set | 5999 |
| `ScriptedAnimation.advance` | call | 5998 |
| `ScriptedArtboard.instance` | call | 3071 |

### Callbacks by name (idle variant)

| Callback | Workloads | Invocations | Phase (advance/input/draw) | Share of callback instr | Share of dispatched bytecode | Instr per invocation |
|---|---:|---:|---|---:|---:|---:|
| `update` | 11 | 12021 | 12021/0/0 | 65.1% | 84.8% | 3128920 |
| `draw` | 14 | 48000 | 0/0/48000 | 26.6% | 12.8% | 320848 |
| `advance` | 10 | 24058 | 24058/0/0 | 8.3% | 2.3% | 199212 |
| `resize` | 1 | 3000 | 3000/0/0 | 0.0% | 0.0% | 1221 |

### Calls, closures, coroutines, strings (idle variant, totals over all steady workloads)

- Frames: 48000. Closures allocated: 169886.0 (3.54/frame). Threads allocated: 18.
- Host calls named pcall 0, coroutine resume 0, yield 0, wrap 0, create 0.
- String work: CONCAT executions 0, string allocations 7543.0 (0.16/frame), calls to format 0, tostring 0, sub 0.
- Builtin fastcall ids by executions: 19:1575525, 24:150044, 9:150044, 40:27002, 12:9001, 18:9001, 53:2999, 52:92, 61:48, 60:12.

### Bytecode type info (authored protos seen at run time, idle variant)

107 protos, 159 parameters; parameters with bytecode type info 0, of which typed (not `any`) 0.


### Native instructions per dispatched bytecode op

| Workload set | Authored bytecode instructions per op | Dispatcher chunk instructions per op |
|---|---:|---|
| Upstream `tests/life`, `tests/matrixmult` (pure interpreter) | 41–47 | n/a |
| Scripted corpus (16 workloads) | 147–504, median about 260 | 91–919 |

| Workload set | Marshalling per host call | Rust body per host call | Callback entry (rt_call + bridge) per invocation |
|---|---|---|---|
| Scripted corpus | 1.3–2.4k | 1.2–6.5k (a median of about 2.3k; `ScriptedArtboard:advance` is 21k) | 3.4–7.3k |

## Not verified

- **Not a production build.** The build was not fat-LTO, which is the
  production profile. Per-op and per-call instruction counts will shrink
  under fat LTO. The ratios between categories are expected to hold, but
  this was not checked.
- **The VM-side call share is inferred.** The claim that the per-op excess
  over the benchmarks is VM-side call work comes from the 41–47 vs 150–500
  contrast and the host-call density. It was not measured by splitting the
  bytecode category per opcode.
- **No C++ comparison.** The C++ runtime running the same `.riv` files was
  not measured, so nothing here says how far from C++ these workloads are.
- **Skip-unchanged is only an upper bound.** Loose equality ignores the
  contents of userdata. The output-equality column is the empirical check:
  it bounds what memoization could save if host objects carried change
  stamps. Nothing tested whether such stamps are cheap.
- **Product workloads are unmeasured beyond setup.** The event-driven product
  scripts were never exercised.
- **One workload dominates the pooled numbers.** `script_path_effects_test`
  is 12.5M of about 19.5M pooled script instructions per frame. Medians are
  given beside the pooled figures for that reason.
- **The arena cost of scoped userdata creation was not separated.** It sits
  inside host_rt_body and rt_call.
- **One benchmark detail run (`tests/trig`) was stopped unfinished**.
  The contrast uses 13 of the 14 `tests/` benchmarks.

## A product-workload speed metric

This metric would replace "match C++ on upstream microbenchmarks". It is a
proposal only; the plan is not edited.

**Scripted frame cost.**

1. **Corpus.** Fix a corpus of product and product-like workloads:
   - the 16 steady scripted fixtures above;
   - recorded event traces for nuxie-dev screens, replaying the
     `performAction` and `convert` calls a real session makes.
2. **Measurements.** In a fat-LTO build, report per workload:
   - script instructions per frame, summing every category above;
   - script instructions per event invocation, for event-driven scripts.
3. **Aggregate.** Use the geometric mean across workloads.
4. **Target.** At or below the C++ runtime on the same `.riv` and trace,
   measured the same way, wherever C++ can run the content.
5. **Diagnostics, carried alongside the main number** so a regression points
   at its cause:
   - instructions per host call;
   - instructions per callback entry;
   - instructions per bytecode op;
   - allocations per frame.

Upstream microbenchmarks stay as a guard against interpreter regression, not
as the target. On this corpus they weight field access and loops above
host calls, and they make fewer than 2 host calls per 1000 ops against about
98.
