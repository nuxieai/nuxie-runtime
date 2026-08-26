# Waves C15-C17 strict inventory

Status: **READ-ONLY AUTHOR DOSSIER; NOT PARITY ACCEPTANCE**

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Wave C15: semantic artboard and lifecycle

Denominator: 19 cases — `semantic_artboard_test.cpp` (10) and
`semantic_data_lifecycle_test.cpp` (9).

Strict result: **4 adapted passes / 5 real expected reds / 10 unported**.

The semantic-artboard file has one adapted pass, five real owner-seam reds, and
four incomplete cases whose final `nodeById` or ancestry assertions are
unconditional placeholders. A real initial failure does not make an incomplete
case an admissible red. The missing seam is an owner-safe selected semantic
node query with existence and parent/ancestor role.

The lifecycle file has three adapted passes and six unported cases. Current
aggregates change pinned constants/flags/action streams. Two destruction cases
call an otherwise unused explicit `detach` test facade rather than exercising
scope destruction, and the fixture-driven state-machine case exists only as
report/corpus metadata.

Author partitions:

1. complete the ten Artboard cases through a real manager node/ancestry query;
2. exact owner tests for lifecycle cases 1-8, including genuine teardown;
3. one literal fixture/action/diff test for lifecycle case 9;
4. ledger correction after the owners freeze.

## Wave C16: semantic dispatch and focus list

Denominator: 18 cases — `semantic_dispatch_test.cpp` (16) and
`semantic_focus_list_test.cpp` (2).

Strict result: **3 adapted passes / 2 real expected reds / 13 unported**.

Only dispatch #7, #15, and #16 presently preserve admissible owner behavior.
The other dispatch claims merge fresh-listener cases into altered aggregates,
change IDs and action order, inject node identities, omit exact sibling flags,
or rely on a fake state-machine resolver. Separate green tests cannot be
composed to stand in for the upstream lookup-to-owner-to-listener chain.

Both focus-list cases are complete live-owner reds. Preserve their exact
ordering/geometry failures; do not rewrite expectations.

Author partitions:

1. literal standalone dispatch/listener and focused-state owner tests;
2. literal manager lookup/removal/focus tests using real SemanticData-created
   nodes where upstream does;
3. activate and retain the two genuine focus-list reds;
4. ledger correction after author lanes freeze.

## Wave C17: semantic label inference

Denominator: all 36 cases in `semantic_label_inference_test.cpp`.

Strict result before authoring: **36 unported**.

The adjacent 28-test `SemanticManager` suite is green and conceptually broad,
but every pinned case is merged, changes constants/tree shapes, or omits
authoritative arrays or identity assertions. The report and aggregate module
are not one-for-one evidence. No production owner gap is proven until the
literal tests execute.

The highest-risk missing streams are:

- the non-vacuous two-child no-reorder condition;
- complete added/updated/childrenUpdated array ordering;
- exact auto-id, cross-manager, watermark, and collision identity behavior;
- absorbed-child incremental updates;
- partial sibling-removal preorder (`[2, 4]`, not whole-subtree removal).

Author partition:

1. cases 1-15: label inference and interactive roles;
2. cases 16-23: ordering and authoritative diff arrays;
3. cases 24-27: id allocation and lookup identity;
4. cases 28-36: incremental dirt and removal ordering.

Each case must preserve pinned node IDs, roles, labels, bounds, mutation order,
and every assertion. Shared construction helpers may create live node owners;
they may not collapse cases or compute expected answers.

## Relied-on gates

All focused probes used `CARGO_INCREMENTAL=0`. Green altered aggregates were
treated only as evidence that adjacent owners run, never as exact case ports.
No production or test files were changed during inventory.
