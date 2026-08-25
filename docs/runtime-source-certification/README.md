# Runtime source certification

This campaign certifies the Rust runtime against pinned Rive runtime commit
`4ac7b32798da0482e441ef09304dc3b480ed3ee5` at the source-symbol level. It is
deliberately narrower and stricter than the completed source-correspondence
campaign: a file path, ownership note, structural verdict, or passing final
render is not by itself evidence that every upstream behavior was translated.

## Certification units

The v2 denominator freezes every pinned authority owner and assigns atomic rows
to:

- out-of-line function bodies, inline methods on source-local classes,
  namespace-scope data/defaulted definitions, source-local macro definitions,
  and external body-macro statements in the 456 manifest-bijected handwritten
  `.cpp` files and the three independently censused Objective-C++ `.mm` files;
- executable bodies in every handwritten `include/rive` or `src` `.h`/`.hpp`
  file, including inline methods, templates, constructors, destructors, and
  operators;
- every handwritten macro definition, plus every invocation of a local macro
  whose replacement list can generate executable bodies; and
- byte-exact `dev/defs`, generated C++ output, checked-in Rust schema, and
  nuxie-codegen authority, with a separate byte-for-byte schema replay.

The existing `file-correspondence-manifest.toml` remains the proven 456 `.cpp`
bijection. It is only that subset, not a claim of whole-source parity. The v2
denominator independently discovers `.mm` and handwritten header authority so
those files cannot disappear merely because the older manifest did not list
them.

Each receipt must:

1. enumerate every authority unit assigned to its owners by the generated
   denominator;
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

The machine ledger intentionally requires more than a disposition word.
`exact` and `adapted` rows require concrete Rust owner symbols, a receipt path,
an accepted independent review with reviewer identity, and behavioral evidence
or a specific evidence-exemption reason. `adapted` also names its approved
adaptation. `not-applicable` requires a governing decision, and `missing`
requires tracking. Receipt paths must be repository-relative, must resolve to
an existing file under `docs/runtime-source-certification`, and are checked for
both owner and symbol rows. These structural checks make omissions and dangling
proof references mechanically detectable. They do not prove that a named Rust
symbol, evidence claim, reviewer identity, or prose receipt is truthful; the
required independent adversarial read remains the semantic proof.

The ledger also has a bijective owner section: every one of the 1,105 authority
paths appears exactly once with a receipt and accepted independent review.
Owners with zero extracted units still require an explicit
`no_executable_units_decision`; a byte fingerprint alone cannot adjudicate
parity for wrapper or declaration-only files.

Non-behavioral include guards and constant macros use `not-applicable` with the
governing non-behavioral decision; they are not mislabeled as unsupported.

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
- all 1,105 authority owners (456 `.cpp`, 3 `.mm`, and 646 handwritten headers)
  have receipts and all 7,818 extracted authority units have one non-`missing`
  disposition;
- all 369 `dev/defs` inputs, 640 generated C++ outputs, the checked-in Rust
  schema, and nuxie-codegen inputs match their frozen byte fingerprints, and a
  fresh nuxie-codegen replay exactly reproduces `schema.rs`;
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

## Parser policy and limits

The checker is a deterministic lexical C++ authority census, not a compiler.
It counts every authored conditional-compilation branch rather than selecting
one platform. Comments, literals, and preprocessor directives cannot create
fake braces. Inline class/nested-class bodies and multiline declarators are
parsed directly; lambda bodies and braced member initializers are excluded as
function false positives. Every remaining unclassified handwritten-header
brace region is nevertheless emitted as a `lexical-brace-authority` row. An
auditor must either map executable behavior in that region or give a governing
`not-applicable` decision for a non-executable aggregate initializer.
Constructor declarators are selected before their initializer lists, including
inline constructors and `alignas` class declarations. Because the pinned
corpus has no legitimate function whose final qualified segment begins `m_`,
the gate rejects such a row as a probable member-initializer misparse; the
frozen snapshot has a direct regression assertion for that invariant.

Arbitrary macros and includes are not expanded. That limitation is explicit:
every local `#define` is its own denominator authority unit, body-generating
macro invocations are separately listed, and every owner file is also frozen by
path, byte count, and SHA-256. Declarations that exist only after macro/include
expansion are governed by those macro authority rows rather than given invented
post-expansion symbol names. Generated directories are never treated as
handwritten; their complete bytes and the Rust schema replay are checked by
`make runtime-generated-authority-gate`.
