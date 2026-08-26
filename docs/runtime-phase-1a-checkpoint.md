# Runtime exact-parity Phase 1A checkpoint

Status: **accepted accounting checkpoint; not a source-parity claim**

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Accounting result

The independently accepted ledger snapshots account for **1,095 of 1,404**
upstream cases:

| disposition | cases |
|---|---:|
| executable pass | 768 |
| executable expected-red | 155 |
| approved not-applicable | 3 |
| pending missing-owner blockers | 169 |
| **independently accepted accounting subtotal** | **1,095** |

Thus the independently accepted subtotal contains **923 executable cases**,
not 926: 768 pass plus 155 genuine executable expected-red. Its executable
mechanisms are 752 direct, four live differential, and 167 approved adapted.
The three remaining non-pending rows are B3 `not-applicable` adaptations and
must not be called executable.

Wave B1 contributes a separate **70 provisional executable cases** (49 pass,
21 expected-red, all direct). Its final document is explicitly titled
`transition-self-acceptance` and git history supplies no durable independent
reviewer identity, so it is not included in the independently accepted
subtotal. It is also not relabeled pending.

The complete denominator therefore reconciles as:

`1,404 = 1,095 independently accepted + 70 provisional B1 + 239 untouched/unaccounted`.

Equivalently, the 1,095 accepted subtotal is `923 executable + 3
not-applicable + 169 pending`. The earlier workflow document's claim that all
non-pending rows were executable conflated B3's three `not-applicable` rows
with executable evidence.

The newly closed checkpoint waves contribute:

| wave | receipt | accounted | pass | expected-red | pending |
|---|---|---:|---:|---:|---:|
| C7 | `1ab4dd63f` | 58 | 6 | 0 | 52 |
| C8 | `70185181b` | 62 | 22 | 13 | 27 |
| C9 | `b1a86c2b5` | 46 | 28 | 2 | 16 |
| **total** |  | **166** | **56** | **15** | **95** |

## Evidence methodology

`tools/runtime-frame-loop-port/aggregate_phase_1a_checkpoint.py` is the
deterministic source for these numbers. It does not read mutable working-tree
ledgers. For each included wave it reads both the ledger and named acceptance
receipt from the exact acceptance commit with `git show`, requires the pinned
upstream SHA and an acceptance verdict, verifies the ledger row count, rejects
duplicate case identities, and aggregates status/outcome fields. Its embedded
provenance table names all 25 accepted receipt/ledger pairs. C9 is read from
its final acceptance commit, not its earlier candidate or rejection.

The script deliberately puts B1 in a separate provisional input. No candidate
presence, self-acceptance, current-file state, or prose assertion of prior
review is inferred to be independent acceptance.

Root closeout independently executed the aggregator, resolved all 25 ledger
and receipt paths from their named commits, confirmed the full arithmetic and
33-file blocker sum, and verified that every source path named by the first
correspondence queue exists at the pinned checkout.

## Pending blockers by upstream test file

These 169 rows are audited inventory, not completed parity:

| upstream test file | pending |
|---|---:|
| `text_input_test.cpp` | 20 |
| `text_test.cpp` | 16 |
| `raw_text_input_test.cpp` | 13 |
| `simple_array_test.cpp` | 13 |
| `serialized_rendering_test.cpp` | 12 |
| `math_test.cpp` | 11 |
| `signed_content_header_test.cpp` | 10 |
| `path_test.cpp` | 9 |
| `semantic_state_machine_test.cpp` | 9 |
| `simd_test.cpp` | 8 |
| `wangs_formula_test.cpp` | 7 |
| `state_machine_test.cpp` | 6 |
| `layout_participant_test.cpp` | 4 |
| `object_stream_test.cpp` | 4 |
| `raw_path_test.cpp` | 4 |
| `line_break_test.cpp` | 3 |
| `node_test.cpp` | 2 |
| `rounded_rect_path_test.cpp` | 2 |
| `text_modifier_test.cpp` | 2 |
| `layout_grid_test.cpp` | 1 |
| `layout_scroll_test.cpp` | 1 |
| `layout_test.cpp` | 1 |
| `nested_text_run_test.cpp` | 1 |
| `reader_test.cpp` | 1 |
| `render_test.cpp` | 1 |
| `rotation_constraint_test.cpp` | 1 |
| `scale_constraint_test.cpp` | 1 |
| `scripting/scripting_properties_test.cpp` | 1 |
| `span_test.cpp` | 1 |
| `state_machine_event_test.cpp` | 1 |
| `stroke_test.cpp` | 1 |
| `translation_constraint_test.cpp` | 1 |
| `trim_test.cpp` | 1 |
| **total** | **169** |

## First source-correspondence queue

This is a source-pair queue, not another test wave. Each numbered source owner
is a separate complete-file audit and review unit. Pending tests are consumers
that identify risk; they do not define the implementation.

1. Audit `src/text/text_input.cpp` against its Rust owners first. It directly
   blocks 20 cases and is currently spread across six Rust files, including
   Artboard, constraints, component, and text coordination paths. That is the
   highest blocker count and highest packed/shared-owner risk.
2. Audit `src/text/text.cpp`, then `src/text/raw_text_input.cpp`, as two
   separate pairs. They block 16 and 13 cases respectively and share shaping,
   layout, cursor, selection, and draw state with the first pair. Together the
   text family accounts for 56 pending rows once line-break, modifier, nested
   run, and span consumers are included.
3. Audit `src/simple_array.cpp` together with the behavior-owning
   `include/rive/simple_array.hpp` boundary. All 13 cases are pending; the
   current correspondence routes header-owned container behavior through a
   broad Rust adaptation while the `.cpp` owns only testing counters. This is
   a concentrated ownership/adaptation risk.
4. Audit the math/path primitive owners one file at a time, starting with the
   exact owners of comparison/bit-mask helpers, SIMD, Wang's formula, and
   `src/math/raw_path.cpp`, before higher-level Shape paths. This family backs
   41 pending rows and is reused widely enough that an error can create many
   downstream render divergences.
5. Audit `src/semantic/semantic_manager.cpp` and its two Rust manager/tree
   owners. Nine semantic rows require live `nodeById`, focus, current state,
   and bounds rather than diff reconstruction; the same owner is shared by
   pointer and state-machine paths.
6. Audit the binary owner boundaries in this order:
   `include/rive/signed_content_header.hpp`,
   `include/rive/object_stream.hpp`, then `src/core/binary_data_reader.cpp`.
   They account for 15 pending cases and expose intermediate position,
   overflow, flags, and stream ownership that downstream parsing cannot prove.
7. Audit the live state-machine owner pair beginning at
   `src/animation/state_machine.cpp` and the corresponding Rust instance
   owner. Seven pending state/event rows depend on intermediate transition,
   reset-pool, listener-rebinding, nested ViewModel, and nested-event state.
8. Audit layout/constraint owners next, one pair at a time, beginning with
   `src/layout/layout_participant.cpp`; the layout and transform-related files
   account for ten pending rows and feed text, semantics, and rendering.
9. Reattribute the 12 pending `serialized_rendering_test.cpp` consumers to
   their actual production owners—randomization, data binding/ViewModel,
   transition advance, virtualization, or rendering—after those source pairs
   are audited. The integration test file itself is not a production owner and
   must not become a monolithic correspondence unit.

After each source-pair audit creates or identifies the exact callable owner,
only its blocked tests return for literal porting. Expected-red rows remain
executable discrepancy evidence and are activated only when source comparison
recovers the pinned behavior.
