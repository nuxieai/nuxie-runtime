# Wave B1 Transition Self correction candidate

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Prior independent review: `e030d2090`

Status: **CORRECTED CANDIDATE — PENDING FRESH ONE-ROW REVIEW**

This correction changes only `data_binding_test.cpp#14`, **Transition self
conditions**. It does not self-accept the row or Wave B1.

## Corrected owner seam

The executable test still imports
`transition_self_comparator_test.riv`, binds the live artboard, state machine,
and view-model owners, performs the initial draw, and reproduces the complete
pinned number, trigger, color, boolean, and string mutation/draw prefix. It
then proves that the retained `lis` owner starts empty.

The prior unconditional panic is removed. The test-only probe now operates on
that actual retained list owner:

1. it requests the first nullable slot without constructing a typed child
   view model;
2. it proves the owner's logical item count becomes exactly one; and
3. it performs an identity-preserving same-index swap, which can succeed only
   if index zero is backed by a retained `ViewModelInstanceListItem` wrapper.

Rust currently records the logical count but retains no wrapper, so the real
owner returns false for the swap. The expected-red fails on that return/state
mismatch. It contains no unconditional panic and will turn green at this seam
when the owner retains addressable nullable wrappers.

The row's reason is now:

> expected-red: the actual list owner records the first nullable slot count but
> cannot retain an addressable ViewModelInstanceListItem wrapper

The other 69 rows and their evidence bodies are unchanged.

## Exact census and gates

- Wave B1: 70 direct rows, 49 pass and 21 executable expected-red;
- all 49 passing rows executed successfully;
- all 21 expected-red rows were forced individually, each selecting one test
  and failing inside its named body;
- Transition Self fails only after the full scalar/draw prefix, the empty-owner
  assertion, and the successful `Some(1)` logical-count assertion;
- all 70 upstream identities, evidence locators, symbols, outcomes, ignore
  attributes, and reasons validate exactly;
- repository correspondence checker: 157 files and 1,404 pinned `TEST_CASE`s,
  green;
- correspondence checker unit suite: 24/24 green;
- non-test LLVM IR excludes the B1 private image-owner test and the Transition
  Self test/probe symbols;
- JSON parsing and scoped `git diff --check`: green.

No production runtime behavior changed. A fresh independent review should
adjudicate only the corrected Transition Self owner seam; the prior review
already accepted the other 69 rows.
