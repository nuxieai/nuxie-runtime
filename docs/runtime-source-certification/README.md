# Runtime source certification

This campaign certifies the Rust runtime against pinned Rive runtime commit
`4ac7b32798da0482e441ef09304dc3b480ed3ee5` at the source-symbol level. It is
deliberately narrower and stricter than the completed source-correspondence
campaign: a file path, ownership note, structural verdict, or passing final
render is not by itself evidence that every upstream behavior was translated.

## Certification unit

The atomic unit is one pinned C++ implementation file. Its receipt must:

1. enumerate every out-of-line function, method, override, constructor,
   destructor, operator, and file-local behavioral helper in the generated
   symbol denominator;
2. map each symbol to exact Rust source symbols, or record a named adaptation
   or not-applicable decision;
3. describe side effects that are easy to lose during translation, including
   dirt propagation, callbacks, parent/child notification, update ordering,
   defaults, cache invalidation, clone/reset behavior, and error paths;
4. bind behaviorally meaningful symbols to a direct unit test or a C++/Rust
   differential that observes their result or state transition;
5. record every discrepancy found and correct it only by recovering pinned
   behavior within the approved adaptation ceiling.

Receipts may group tightly coupled files for readability, but no symbol may be
certified only transitively through a subsystem-level summary.

## Allowed dispositions

- `exact`: the Rust symbol preserves the pinned algorithm, ordering, and
  observable side effects.
- `adapted`: implementation differs under a pre-approved Rust, Taffy, audio,
  or scripting adaptation while the supported observable contract is bound to
  evidence.
- `not-applicable`: the symbol has no runtime meaning in this product; the
  receipt must cite the governing decision.
- `missing`: no faithful Rust behavior exists yet. This is a work item, never a
  certification result.

`reviewed`, `mapped`, `faithful`, `DIVERGENT`, and `TRACKED-GAP` from earlier
ledgers are inputs to this campaign, not symbol dispositions.

## Review discipline

The implementing auditor reads the complete pinned C++ file and all mapped
Rust owners side by side. A separate adversarial reviewer then tries to falsify
the receipt, paying particular attention to syntactically similar constructs
with different language semantics and to behavior split across Rust modules.
The reviewer does not infer correctness from the original auditor's confidence
or from a prior B6 verdict.

For files that require corrections, the reviewer checks both the patch and the
new evidence. A workaround that merely satisfies the test is rejected when it
cannot be traced to the pinned source.

## Campaign gates

The campaign closes only when:

- the generated source-symbol denominator matches the pinned checkout;
- all 456 source-owner files have receipts and every extracted symbol has one
  non-`missing` disposition;
- every receipt has independent adversarial review;
- the 87 pending-verification rows, 27 historical tracked gaps, open gap
  register entries, shared owners, and explicit exceptions are re-adjudicated;
- all 1,404 translated upstream tests execute with no ignored tests, and their
  fixture/action/assertion correspondence is certified;
- ordinary and scripted C++/Rust differential gates contain no unexplained
  result;
- focused state-trace or generative differentials cover lifecycle behavior that
  final rendering does not reliably expose.

Broad platform CI, editor integration, packaging, and renderer qualification
remain separate campaigns. They do not replace the local source and behavioral
evidence required here.
