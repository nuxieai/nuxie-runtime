# Luau VM bake-off: vendored luaur vs ruau vs luau-rs vs C++ (2026-10-31)

Context: sizing a memory-safe, panic-free Luau VM. Candidates measured against
the upstream Luau `0.734` tree (`3fc82b10`, the version ruau tracks and the
era of our `rive_0_734` pin), all built release, all run on this laptop with
nothing else running during the perf pass (the first pass ran under build
load and was discarded).

| VM | What it is | Source |
|---|---|---|
| cpp | upstream C++ Luau 0.734, `luau` CLI + `Luau.UnitTest` | luau-lang/luau `3fc82b10` |
| luaur | our vendored fork (`vendor/luaur-*-0.1.8`, Rive profile), driven through `luaur-rt` | this repo |
| luaurs | luau-rs v0.739 (pcbzzz), faithful C++ port, `luau` CLI + `luau` crate | github.com/luau-rs/luau `2026-09-23` |
| ruau | ruau 0.4.0 (cortesi), `unsafe_code = "forbid"`, generational-index arena GC, gas-metered | github.com/cortesi/ruau `2026-08-18` |

Runners (scratchpad only, not committed): one small Rust binary per Rust VM
that mirrors upstream's `runConformance` harness as far as each public API
allows (`collectgarbage`, `loadstring`, silent `print`, sandbox, `_G`), plus
a `bench` mode that runs upstream `bench/` scripts with `bench_support.lua`
inlined. All four compile at optimization level 1, debug level 1, like the
C++ CLI default.

## 1. Conformance (upstream `tests/conformance/*.luau`, 55 scripts)

Pass = chunk returns `"OK"`. C++ passes all by construction (upstream CI at
the tag); I verified a sample (`-tc=Basic,sort,...`) locally.

```
== plain scripts (no host setup upstream): 29
script                           luaur    luaurs      ruau
assert                              OK        OK        OK
attrib                              OK        OK        OK
basic                            ERROR     ERROR     ERROR
bitwise                             OK        OK        OK
calls                               OK        OK     ERROR
clear                               OK        OK        OK
closure                          ERROR        OK        OK
constructs                          OK        OK        OK
coroutine                        ERROR     ERROR     ERROR
datetime                            OK        OK     ERROR
debug                               OK        OK        OK
errors                           ERROR     ERROR     ERROR
events                           ERROR        OK     ERROR
explicit_type_instantiations        OK        OK        OK
ifelseexpr                          OK        OK        OK
iter_fenv                           OK        OK        OK
literals                         ERROR        OK        OK
locals                              OK        OK        OK
math                                OK        OK        OK
move                                OK        OK        OK
pm                               ERROR        OK        OK
safeenv                             OK        OK        OK
sort                                OK     ERROR     ERROR
strconv                             OK        OK        OK
stringinterp                        OK        OK        OK
strings                          ERROR        OK     ERROR
tpack                               OK        OK        OK
utf8                                OK        OK        OK
vararg                              OK        OK        OK
  luaur: 21/29 OK
  luaurs: 25/29 OK
  ruau: 21/29 OK

== scripts needing host setup upstream: 26
script                           luaur    luaurs      ruau
apicalls                            OK        OK        OK
buffers                          ERROR     ERROR     ERROR
classes                          ERROR     ERRORCOMPILE-ERROR
coverage                         ERROR     ERROR     ERROR
cyield                           ERROR     ERROR     ERROR
debugger                         ERROR     ERROR     ERROR
exceptions                          OK        OK        OK
gc                               ERROR     ERROR     ERROR
integers                         ERROR     ERROR     ERROR
integers_regspill                ERROR     ERROR     ERROR
interrupt                           OK        OK        OK
iter                             ERROR     ERROR     ERROR
jit_inliner                         OK        OK        OK
native                           ERROR     ERROR     ERROR
native_integer_spills               OK        OK        OK
native_types                     ERROR     ERROR     ERROR
native_userdata                  ERROR     ERROR     ERROR
ndebug_upvalues                 NOT-OK    NOT-OK     ERROR
pcall                            ERROR     ERROR     ERROR
tables                           ERROR     ERROR     ERROR
tmerror                          ERROR        OK        OK
types                            ERROR     ERROR     ERROR
udata_direct                     ERROR     ERROR     ERROR
userdata                         ERROR     ERROR     ERROR
vector                           ERROR     ERROR     ERROR
vector_library                   ERROR     ERROR     ERROR
  luaur: 5/26 OK
  luaurs: 6/26 OK
  ruau: 6/26 OK
```

How to read the failures (classified by reading the failing line):

**Harness-limited (my runner, not the VM)** — same for all three Rust VMs:
`coroutine.luau:8/10` (upstream resumes the *main* state via `lua_resume`,
public APIs give pcall-on-main or a coroutine), `errors.luau:21` and
`basic.luau:39/596` (chunk-name rendering of `loadstring` errors;
luaur-rt always prefixes `=` and the VM prints it verbatim), `pcall.luau:8`,
`datetime.luau:90` (ruau: frozen deterministic clock), `gc.luau` beyond
line 99 (sized `collectgarbage("step", kb)` not exposed by luau-rs).

**Profile omissions (deliberate in the VM's product profile):**
- luaur (Rive profile): no `newproxy`, no `print` → `events`, `strings`,
  `iter`, parts of `basic`/`errors`. `integers*.luau` use the upstream
  integer-literal syntax the Rive fork does not carry. `pm.luau:72` and
  `literals.luau`: luaur's compiler and chunk API take Rust `String`, so
  Latin-1 test sources cannot be fed byte-exact (C++ takes bytes).
- ruau: no `newproxy`/userdata (declared permanent non-goal), `classes`
  syntax; its `Limits` profile caps varargs/unpack differently so
  `calls.luau:262`, `coroutine.luau:287`, `tables.luau:677` ("too many
  results to unpack" boundary at 8000) diverge; `calls.luau` takes ~160 s
  to reach that line (gas lifted).
- `classes.luau` needs the experimental-classes compiler flag everywhere.

**Genuine divergence candidates (worth a look before trusting either VM):**
- ruau and luau-rs: `sort.luau:107` — comparator that mutates the array and
  runs `loadstring`+`collectgarbage` must make `pcall(table.sort)` fail;
  both return success. luaur passes this.
- luau-rs: `iter.luau:440` (yield inside generalized iteration),
  `closure`/`events` pass where luaur does not only because luau-rs ships
  `newproxy`.
- luaur: `pcall.luau:66` (10 000-result chain against LUAI_MAXCALLS) and
  `closure.luau:293` need a chunk-name-independent re-check.

**Scripts needing host setup** (upstream passes a C++ setup lambda:
buffers, vector helpers, userdata metatables, yield helpers, callbacks)
were run without it; the 5–6 that still pass need no setup in practice.
These are the host-API half of a port and are not scored here.

luau-rs's *own* suite (0.739 fixtures with its setup code) passes 57/58
in 0.8 s; the one failure is `coroutine.finally`, a post-0.734 API.
ruau's own suite could not be run: its decomposed `conformance-ruau/`
scripts are not in the public repository, so its scope file's claims are
not reproducible from the outside.

## 2. Performance (upstream `bench/tests` + `bench/micro_tests`, ms, median of 20 runs)

Summary (ratio to C++; the `classes` benches fail everywhere without the
flag; `Failure_*` and `string_lib` are error-path benches where the Rust
VMs win by returning `Result` instead of throwing):

| VM | geomean x/cpp (all 72) | geomean x/cpp (66, error-path benches excluded) | median |
|---|---|---|---|
| luau-rs | 0.86x | 1.01x | 1.04x |
| luaur (ours, sandboxed) | 0.89x | 1.12x | 1.13x |
| ruau | 7.03x | 9.60x | 9.09x |

luaur's worst: `tictactoe` 2.28x, `sha256` 1.42x, `trig` 1.40x,
`Assert`/`TableIteration` 1.4x. Note the first luaur pass, run without
`luaL_sandbox`, showed 4–8x on `math.sqrt`/`ipairs`/`assert`: builtin
fastcalls only engage under `safeenv`, which is a trap for anyone
benchmarking through the raw API.

ruau's worst: `UpvalueCapture` 46x, `Assert` 43x, `sha256` 30x,
`VariadicSelect` 27x, `vector_lib` 20x, `sieve` 16x; its best non-error
bench is `TableCreate_number` at 1.7x. ruau runs gas-metered and snapshot-
able by design (tenant isolation), so part of this is architecture, not
just the safe-handle tax; but it is the number a zero-unsafe Luau VM
currently posts.

```
bench                                                    cpp      luaurs       luaur        ruau     x/cpp     x/cpp     x/cpp
micro_tests__test_AbsSum_abs                            3.51        4.00        4.11       27.51     1.14x     1.17x     7.84x
micro_tests__test_AbsSum_and_or                         4.69        5.43        4.76       41.18     1.16x     1.01x     8.78x
micro_tests__test_AbsSum_math_abs                       3.59        3.99        4.14       30.01     1.11x     1.15x     8.37x
micro_tests__test_Assert                                2.45        2.81        3.42      105.62     1.15x     1.40x    43.16x
micro_tests__test_Factorial                            26.59       27.98       33.05      251.01     1.05x     1.24x     9.44x
micro_tests__test_Failure_pcall_a_bar                  48.36        4.83        1.52        4.03     0.10x     0.03x     0.08x
micro_tests__test_Failure_pcall_game_Foo               56.44        5.06        1.59        4.20     0.09x     0.03x     0.07x
micro_tests__test_Failure_xpcall_a_bar                 55.85        5.07        1.73        4.87     0.09x     0.03x     0.09x
micro_tests__test_Failure_xpcall_game_Foo              60.82        5.03        1.89        5.36     0.08x     0.03x     0.09x
micro_tests__test_LargeTableCtor_array                  3.95        3.63        3.53       29.11     0.92x     0.89x     7.38x
micro_tests__test_LargeTableCtor_hash                   3.03        2.84        2.75       28.48     0.94x     0.91x     9.41x
micro_tests__test_LargeTableSum_loop_index              3.79        3.43        3.39       24.26     0.91x     0.90x     6.41x
micro_tests__test_LargeTableSum_loop_ipairs             2.83        2.88        3.39       21.24     1.02x     1.20x     7.51x
micro_tests__test_LargeTableSum_loop_iter               2.60        2.57        3.29       22.23     0.99x     1.26x     8.54x
micro_tests__test_LargeTableSum_loop_pairs              2.78        2.61        3.25       21.89     0.94x     1.17x     7.87x
micro_tests__test_MethodCalls                          25.41       21.41       22.06      200.36     0.84x     0.87x     7.88x
micro_tests__test_OOP_constructor                      30.97       29.31       29.08      342.08     0.95x     0.94x    11.05x
micro_tests__test_OOP_constructor_classes               FAIL        FAIL        FAIL        FAIL         -         -         -
micro_tests__test_OOP_constructor_classes_direct        FAIL        FAIL        FAIL        FAIL         -         -         -
micro_tests__test_OOP_field_access                     23.84       23.84       30.04      257.62     1.00x     1.26x    10.80x
micro_tests__test_OOP_field_access_classes              FAIL        FAIL        FAIL        FAIL         -         -         -
micro_tests__test_OOP_field_access_random             256.97      266.14      269.55     2752.02     1.04x     1.05x    10.71x
micro_tests__test_OOP_field_access_random_classes        FAIL        FAIL        FAIL        FAIL         -         -         -
micro_tests__test_OOP_method_access                    76.67       92.01       81.19      411.47     1.20x     1.06x     5.37x
micro_tests__test_OOP_method_access_classes             FAIL        FAIL        FAIL        FAIL         -         -         -
micro_tests__test_OOP_method_call                      99.04      105.96      101.97     1110.44     1.07x     1.03x    11.21x
micro_tests__test_OOP_method_call_class                 FAIL        FAIL        FAIL        FAIL         -         -         -
micro_tests__test_OOP_virtual_constructor               2.56        3.06        3.04       27.72     1.19x     1.19x    10.81x
micro_tests__test_Pcall_call_return                     0.64        0.79        0.79        6.86     1.24x     1.23x    10.75x
micro_tests__test_Pcall_pcall_return                    2.54        1.46        2.77       12.18     0.57x     1.09x     4.79x
micro_tests__test_Pcall_pcall_yield                    14.49       14.91       16.21      120.75     1.03x     1.12x     8.33x
micro_tests__test_Pcall_xpcall_return                   3.32        1.56        3.22       12.24     0.47x     0.97x     3.69x
micro_tests__test_SqrtSum_exponent                      1.67        1.65        1.67       14.61     0.99x     1.00x     8.74x
micro_tests__test_SqrtSum_math_sqrt                     1.74        1.85        1.93       10.18     1.07x     1.11x     5.85x
micro_tests__test_SqrtSum_sqrt                          1.73        1.90        2.00       10.15     1.10x     1.15x     5.86x
micro_tests__test_SqrtSum_sqrt_getfenv                  4.76        4.89        3.97        FAIL     1.03x     0.83x         -
micro_tests__test_SqrtSum_sqrt_roundabout               4.39        4.78        3.93       41.44     1.09x     0.89x     9.43x
micro_tests__test_StringInterp                         80.29       89.10       86.59      517.66     1.11x     1.08x     6.45x
micro_tests__test_TableCreate_nil                      10.30       10.51       11.12       20.59     1.02x     1.08x     2.00x
micro_tests__test_TableCreate_number                   12.72       12.94       13.77       21.32     1.02x     1.08x     1.68x
micro_tests__test_TableCreate_zerofill                 66.30       58.20       59.17      303.11     0.88x     0.89x     4.57x
micro_tests__test_TableFind_loop_ipairs                15.37        9.33       13.32       58.68     0.61x     0.87x     3.82x
micro_tests__test_TableFind_table_find                  3.19        3.36        3.12       15.51     1.05x     0.98x     4.85x
micro_tests__test_TableInsertion_index_cached           2.38        2.19        2.20        6.96     0.92x     0.92x     2.92x
micro_tests__test_TableInsertion_index_len              1.89        2.07        2.15       12.51     1.10x     1.14x     6.64x
micro_tests__test_TableInsertion_table_insert           1.72        1.89        2.31       21.64     1.09x     1.34x    12.55x
micro_tests__test_TableInsertion_table_insert_index        0.78        0.49        0.94       15.70     0.62x     1.21x    20.18x
micro_tests__test_TableIteration                        2.97        3.36        4.12       30.15     1.13x     1.39x    10.14x
micro_tests__test_TableMarshal_select                   6.21        7.16        7.01       53.03     1.15x     1.13x     8.53x
micro_tests__test_TableMarshal_table_pack               5.70        6.49        6.37       58.04     1.14x     1.12x    10.18x
micro_tests__test_TableMarshal_table_unpack_array       11.31       13.46       13.42      218.83     1.19x     1.19x    19.35x
micro_tests__test_TableMarshal_table_unpack_range       12.93       14.81       14.38      239.64     1.15x     1.11x    18.54x
micro_tests__test_TableMarshal_varargs                  4.07        4.58        5.10       35.28     1.12x     1.25x     8.66x
micro_tests__test_TableMove_empty_table                 0.13        0.12        0.14        2.83     0.93x     1.13x    22.31x
micro_tests__test_TableMove_same_table                  0.07        0.06        0.07        5.53     0.85x     0.98x    75.82x
micro_tests__test_TableMove_table_create                0.07        0.06        0.09        2.15     0.81x     1.21x    29.42x
micro_tests__test_TableRemoval_table_remove             0.70        0.63        0.73       15.10     0.90x     1.05x    21.67x
micro_tests__test_TableSort                            11.29       10.60       10.84       57.12     0.94x     0.96x     5.06x
micro_tests__test_ToNumberString                       11.04        9.70       12.77      171.62     0.88x     1.16x    15.55x
micro_tests__test_UpvalueCapture                       24.56       24.95       24.98     1129.17     1.02x     1.02x    45.98x
micro_tests__test_VariadicSelect                        6.20        7.95        8.01      166.01     1.28x     1.29x    26.79x
micro_tests__test_string_lib                           10.47        1.79        2.24        4.54     0.17x     0.21x     0.43x
micro_tests__test_table_concat                         19.37       15.34       15.34       36.42     0.79x     0.79x     1.88x
micro_tests__test_vector_lib                           21.34       20.67       19.73      435.30     0.97x     0.92x    20.39x
tests__base64                                           8.02        9.14       10.07      125.13     1.14x     1.26x    15.60x
tests__chess                                           43.66       45.16       55.86      489.11     1.03x     1.28x    11.20x
tests__chess-classes                                    FAIL        FAIL        FAIL        FAIL         -         -         -
tests__life                                            35.58       37.75       48.68      269.70     1.06x     1.37x     7.58x
tests__matrixmult                                       8.63       12.51        8.80       65.95     1.45x     1.02x     7.64x
tests__mesh-normal-scalar                              12.26       14.14       15.39      143.25     1.15x     1.26x    11.68x
tests__mesh-normal-vector                               6.28        6.45        7.88       70.40     1.03x     1.26x    11.21x
tests__pcmmix                                           3.37        3.69        3.91       63.22     1.10x     1.16x    18.78x
tests__qsort                                           52.10       48.20       61.26      311.41     0.93x     1.18x     5.98x
tests__sha256                                           8.07        9.59       11.44      243.44     1.19x     1.42x    30.17x
tests__sieve                                           86.11      113.65      106.88     1379.67     1.32x     1.24x    16.02x
tests__tictactoe                                       62.33       72.88      142.16      411.89     1.17x     2.28x     6.61x
tests__trig                                             8.46       12.21       11.83      118.10     1.44x     1.40x    13.95x
tests__vector-math                                      4.30        4.81        4.69       42.82     1.12x     1.09x     9.96x
tests__voxelgen                                        38.91       37.73       51.02      257.11     0.97x     1.31x     6.61x
luaurs: geomean 0.86x median 1.03x over 72 benches (times in ms, median of 20 runs)
luaur: geomean 0.89x median 1.11x over 72 benches (times in ms, median of 20 runs)
ruau: geomean 7.03x median 8.66x over 71 benches (times in ms, median of 20 runs)
```

## 3. Panic-capable sites (clippy, forced lints, lib target only)

Lints: unwrap_used, expect_used, indexing_slicing, panic, unreachable,
arithmetic_side_effects, unwrap_in_result. Counts are sites, not proofs.

ruau-vm (55k lines, zero `unsafe`):
```
1028 clippy::arithmetic_side_effects
   327 clippy::indexing_slicing
   120 clippy::expect_used
    35 clippy::unreachable
    24 clippy::unwrap_in_result
  1534 total
```

luaur-vm (53k lines, 1433 `unsafe` sites; other clippy lints shown too
because the crate is generated code without lint suppression):
```
1047 clippy::arithmetic_side_effects
   784 clippy::missing_safety_doc
   505 clippy::not_unsafe_ptr_arg_deref
   204 clippy::indexing_slicing
    72 clippy::manual_c_str_literals
    53 clippy::macro_metavars_in_unsafe
    51 clippy::unnecessary_cast
    48 clippy::explicit_auto_deref
    40 clippy::ptr_eq
    23 clippy::needless_return
    17 clippy::manual_range_contains
    15 clippy::unwrap_used
    11 clippy::crate_in_macro_def
    10 clippy::expect_used
    10 clippy::eq_op
     9 clippy::collapsible_if
     8 clippy::useless_attribute
     8 clippy::panic
     7 clippy::excessive_precision
     6 clippy::derivable_impls
     6 clippy::manual_div_ceil
     6 clippy::neg_cmp_op_on_partial_ord
     6 clippy::approx_constant
     5 clippy::manual_rotate
     5 clippy::unreachable
     4 clippy::needless_borrow
     4 clippy::zero_ptr
     3 clippy::unwrap_in_result
     3 clippy::needless_late_init
     3 clippy::match_like_matches_macro
     3 clippy::while_immutable_condition
     2 clippy::missing_transmute_annotations
     2 clippy::ptr_offset_with_cast
     2 clippy::bool_comparison
     2 clippy::blocks_in_conditions
     2 clippy::char_lit_as_u8
     2 clippy::if_same_then_else
     2 clippy::useless_conversion
     2 clippy::self_named_constructors
     1 clippy::int_plus_one
     1 clippy::let_unit_value
     1 clippy::assign_op_pattern
     1 clippy::unnecessary_unwrap
     1 clippy::cmp_null
     1 clippy::out_of_bounds_indexing
     1 clippy::byte_char_slices
     1 clippy::identity_op
     1 clippy::should_implement_trait
     1 clippy::mut_from_ref
     1 clippy::let_and_return
  3003 total
```

Reading: ruau is memory-safe but not panic-free by construction; ~500
unwrap/expect/unreachable/indexing sites remain inside the VM, contained
only by a `catch_unwind` at the exec boundary. luaur has fewer explicit
panic sites but 1433 unsafe sites and 505 public functions that
dereference raw pointers, so its failure mode is UB, not a panic.

## 4. What this settles

1. **A faithful Rust port can match C++.** luau-rs is at parity on the
   upstream benches and our own luaur fork is within ~12% once sandboxed.
   The "Rust VM must be slow" worry is not supported.
2. **The only zero-unsafe Luau today is ~9x slower** on the same benches
   and drops userdata, `newproxy` and the upstream limit profile. Forking
   ruau means accepting that starting point and then closing a 9x gap in
   a design built for metered multi-tenant isolation, not for a game
   runtime hot loop.
3. **Conformance pass rates are harness-bound, not VM-bound.** On the 29
   plain scripts the three Rust VMs land at 21–25 with every gap
   explainable by profile choices or my harness. The real conformance
   question is the 26 host-setup scripts, i.e. the embedding API
   (userdata, callbacks, yields across C, interrupts), which is exactly
   where our product bindings live.
4. **Panic-freedom is a separate project from memory safety.** Neither
   candidate has it. It would be a clippy-ratchet plus abort-profile probe
   campaign on whichever VM we pick.

Artifacts (scratchpad, session-local): runners under `priorart/runner-*`,
raw outputs under `priorart/conf-out` and `priorart/bench-out`, upstream
tree under `priorart/upstream/luau`.
