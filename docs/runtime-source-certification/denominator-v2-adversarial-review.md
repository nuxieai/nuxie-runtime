# V2 source denominator adversarial review

## Verdict

Commits `979a6bc44` and `51ded1406` are **rejected as originally written** and
the corrected denominator in this review is **accepted with the residual limits
below**. The original 7,540-unit snapshot was owner-complete and byte-frozen,
but it was not definition-complete within its stated handwritten source scope.
The corrected snapshot has 7,818 authority units across the same 1,105 owners.

This acceptance certifies the denominator mechanism and frozen authority. It
does not certify that any Rust runtime owner is behaviorally equivalent to its
C++ authority.

## Complete census

The corrected snapshot pins upstream
`4ac7b32798da0482e441ef09304dc3b480ed3ee5` and discovers:

| authority | owners | units |
| --- | ---: | ---: |
| manifest-bijected `.cpp` | 456 | included below |
| independently discovered `.mm` | 3 | included below |
| independently discovered handwritten `.h`/`.hpp` | 646 | included below |
| **total owners** | **1,105** | **7,818** |

The unit-kind census is 6,896 functions, 777 macro definitions, 80
namespace-scope source statements, 45 body-macro invocations, and 20 header
lexical-brace authorities. Six owners have zero extracted units; they are two
empty C++ wrappers, two four-line Apple include wrappers, and two
declaration-only headers. They still require explicit owner receipts and
no-executable-unit decisions.

All 4,252 pre-existing `.cpp` IDs and all 10 pre-existing `.mm` IDs remain
stable. The correction adds 262 `.cpp` IDs and 16 `.mm` IDs. In headers, 3,266
IDs remain stable while 12 over-broad fallback IDs are replaced one-for-one by
the 12 actual function IDs they had swallowed. Generated authority is bytewise
unchanged from `51ded1406`.

## Falsifications and corrections

### Source-local classes were invisible

The old source pass used `include_inline_class=False`. It therefore skipped 180
inline methods in 17 `.cpp`/`.mm` owners, despite those methods being authored
only in the implementation files. Representative omissions included 55 methods
from `src/animation/state_machine_instance.cpp`, 25 comparison methods from
`src/animation/transition_viewmodel_condition.cpp`, 14 methods from
`src/command_server.cpp`, and 14 Objective-C++ helper methods from
`src/text/font_hb_apple.mm`.

Source parsing now includes source-local and qualified nested classes. The
qualified-class correction is necessary for
`CommandServer::CommandFileAssetLoader`: the old class-name parser reduced that
owner to `CommandServer` and then misidentified `m_internalLoader(...)` as the
constructor.

### Namespace-scope definitions were invisible

The old body-only pass silently omitted scalar/static definitions, aggregate
registries, vtables, callback tables, and explicitly defaulted out-of-line
functions. The contradiction was observable without an edge case:
`src/layout.cpp` was a zero-unit owner even though it defines all nine static
`Alignment` values. `src/math/random.cpp` likewise appeared empty despite its
counter and stored-result definitions.

The new `source-statement-authority` census retains 80 such definitions. It
also replaces 35 previously invisible brace regions with complete statement
rows, including both `EM_JS` bodies, Lua registration tables, audio vtables,
script verification keys, and listener/inference tables. A statement is one
authority unit; entries within an authored table are not falsely presented as
separate C++ symbols.

### Source-local macros were invisible

The README claimed every handwritten macro definition was explicit, but the
implementation scanned only headers. Seventeen `.cpp` macro definitions were
missing, including the source-local bounds macro in
`src/lua/lua_buffer_ext.cpp`. They are now included, increasing the macro
definition census from 760 to 777.

### Direct `extern "C"` functions were treated as scopes

`_scope_kind` classified every header beginning with `extern` as a linkage
scope. It skipped the complete definition of `luaopen_rive_buffer_ext` because
that function uses `extern "C" int ... {}` rather than an enclosing
`extern "C" { ... }` block. The parser now distinguishes those forms, and the
corpus definition plus a focused regression are present.

### Array subscripts were treated as lambda captures

The lambda guard interpreted any `[...]` after a declarator as a capture list.
Constructor initializers such as `pts[0]` therefore collapsed constructors and
following operators into broad lexical fallback rows. In
`include/rive/math/bezier_utils.hpp`, the corrected census has 15 actual
functions and no fallback; `include/rive/math/raw_path_utils.hpp` has five
actual functions and no fallback. The fix recognizes capture-list expression
positions rather than rejecting ordinary subscripts.

### Template specializations and uncommon operators lacked adversarial cases

An explicit function specialization such as
`Traits::value<std::vector<int>>()` ended in `>` before its parameter list and
was not recognized. The name parser now retains explicit template arguments.
Focused corpus tests also bind `operator new`, `operator delete`, a literal
operator, `operator<=>`, `operator()`, conversion operators, subscripts,
constructors, destructors, nested classes, and anonymous classes.

### Receipt strings were not proof references

The disposition checker formerly accepted any nonempty receipt string,
including missing and out-of-repository paths. Owner and symbol receipts must
now be repository-relative existing files under
`docs/runtime-source-certification`.

This is structural validation only. The checker still cannot prove that a Rust
symbol name exists, that a test claim exercises the behavior, or that the named
reviewer is independent. Those assertions require the documented adversarial
human read; the README no longer claims that schema shape alone prevents a
fabricated bulk ledger.

## Representative manual reconciliation

The Artboard-family owners were manually rechecked as a representative mix of
large sources, templates, constructors, clone helpers, and nested lifecycle:

| owner | corrected units |
| --- | ---: |
| `src/artboard.cpp` | 133 functions |
| `src/nested_artboard.cpp` | 51 functions |
| `src/nested_artboard_layout.cpp` | 21 functions |
| `src/nested_artboard_leaf.cpp` | 2 functions |
| `src/artboard_component_list.cpp` | 88 functions |
| `include/rive/artboard.hpp` | 58 functions + 1 macro |
| `include/rive/nested_artboard.hpp` | 14 functions + 1 macro |
| `include/rive/nested_artboard_layout.hpp` | 3 functions + 1 macro |
| `include/rive/nested_artboard_leaf.hpp` | 1 function + 1 macro |
| `include/rive/artboard_component_list.hpp` | 13 functions + 1 macro |

Corpus anomaly queries also examined every remaining lexical fallback. The 20
header fallbacks are aggregate/default-member/lambda authorities, not hidden
function bodies. Every source brace fallback produced by the adversarial mode
is either covered by a complete source-statement row or is a braced constructor
initializer already covered by its enclosing function row.

## Frozen generated authority and replay

The generated authority remains:

- 369 `dev/defs` files, 275,927 bytes, corpus SHA-256
  `a9864aacb8db17fe6fc35553a2ccf720403dd64cee73ebacbf0c2c11f9c05e26`;
- 640 generated C++ files, 1,162,414 bytes, corpus SHA-256
  `3d1fb3e5b0769e9472703bf483d8295aae2e1c72677751db8e47df3456ca5f4b`;
- checked-in `schema.rs`, 1,119,655 bytes, SHA-256
  `63af07d388b96484122fd5efa1e1490d253790cf50ba153b5a66fc285268f46b`;
- two nuxie-codegen production inputs, 71,891 bytes, corpus SHA-256
  `55e9d0bde7e79214c1ba1f15042a4bfd928dddf2b090bf1f2506d519439c046b`.

The full gate regenerated the schema in a temporary directory, formatted it,
and matched the checked-in bytes exactly. A dedicated temporary-git regression
also proves that staged or unstaged changes under `src`, `include/rive`, or
`dev/defs` are rejected. Untracked additions cannot pass silently: source/header
discovery or generated-authority path/byte snapshots drift even though Git's
tracked-dirt check does not name them.

## Residual limits

- This is a deterministic lexical census, not a C++ compiler. It intentionally
  counts every authored conditional branch without selecting a target
  configuration. Configuration validity remains a separate compile concern.
- Declarations synthesized only by macro/include expansion do not receive
  invented post-expansion C++ names. Local macro authority and invocations are
  explicit; externally supplied macro behavior is bounded by the complete owner
  fingerprint and must be adjudicated in the owner receipt.
- Header declarations without authored bodies, including enum values, scalar
  default-member initializers, and `= default`/`= delete`, are not individual
  executable-body rows. Their owner bytes are frozen and their receipt must
  still read and adjudicate the complete header. Namespace-scope source
  definitions are individual rows because they own storage/initialization.
- Aggregate callback/registry tables are one source-statement unit, not one row
  per initializer entry.
- Ledger validation proves bijection, required fields, and resolvable receipt
  paths. Behavioral truth and reviewer independence remain human assertions.

## Verification

`CARGO_INCREMENTAL=0 make runtime-source-symbol-gate` passes all parser tests,
the exact 7,818-unit snapshot check, frozen generated authority, and byte-exact
schema replay.
