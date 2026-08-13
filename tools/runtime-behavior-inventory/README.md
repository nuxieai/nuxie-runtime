# Runtime behavior inventory

This gate closes the gap between file correspondence and behavior
correspondence. It discovers behavior-bearing members in the pinned C++ tree
and behavior-bearing items in shipped Rust runtime/support/renderer crates,
then compares them with `runtime-behavior-inventory.json`.

## Scope and records

C++ discovery covers `src` implementation files plus `src` and
`include/rive` headers. Every source is classified as `implementation`,
`behavioral-header`, `declaration-only`, or `generated`. A brace-aware lexical
scanner records function and method bodies, including inline and template
definitions. Records retain a stable path/name/overload identity, source span,
signature/content hashes, and tags for setters, callbacks, virtual overrides, mutation
guards, dirt/dependency operations, lifecycle, ownership, ordering loops, and
scalar edge operations. Behavior-bearing macro definitions are retained as
condition-aware logical-macro records on their source file; their directive
syntax is masked before member discovery so expansions cannot become fake
members or hide the following declaration. Behavioral headers without a file-correspondence owner
enter as exact-member `reviewed-gap` records: a later member in the same header
is still unmapped and fails until separately reviewed.

Rust discovery follows production-capable Cargo reachability across all Rust
sources in the shipped runtime-support and renderer crates listed in
`behavior_inventory.py`; custom roots, `#[path]` modules, includes, and build
script helpers are not limited to a conventional `src` directory. Functions,
impl methods, and trait defaults receive signature-derived stable records.
Every effective Cargo build script in that crate scope, including a
manifest-selected custom path and excluding `package.build = false`, is also an explicit generator root:
its functions and file hash are inventoried and its shipped Rust, bytecode,
native-link, or provenance outputs are named in the record. The one checked-in generated schema
source is an explicit generator-bound allowlist entry; mixed sources use
dedicated `// @generated-region begin/end` marker comments, and partial
item/range overlap is invalid. Marker text in strings, documentation, or prose
comments grants no provenance.
A banner or `generated/` directory name alone grants no provenance. A legacy whole-file `codegen`
classification is deliberately not enough to hide a handwritten item. The
source-file hash is a conservative backstop for behavior in macros, constants,
or syntax the lexical item scanner does not understand. Test-required Rust
constructs and their inert line trivia are erased from that shipped-source
proof, so adding or removing test-only code cannot stale production evidence.

Rust items mapped by the correspondence manifest carry baseline owners. Only
the named D3/D16/D17/D18 and A1-A8 adaptations, plus X1/X2/X3 extensions,
point to a deduplicated seam policy that records exact C++ owners, allowed call
direction, forbidden baseline effects, and required evidence. Host/product
support with no parity claim remains explicit as `host-support` rather than
being misrepresented as a C++ port. That approval is item-bound: a new
otherwise-unmapped item does not inherit `host-support` merely because it
shares a file with reviewed support
code. Snapshot regeneration is the explicit approval boundary for those item
IDs.

## Commands

Run the unit tests and checked snapshot gate:

```sh
make runtime-behavior-inventory
```

After reviewing an intentional upstream or Rust behavior change, regenerate:

```sh
make runtime-behavior-inventory-snapshot
```

The tool requires `RIVE_RUNTIME_DIR` to be checked out at the exact
`port-manifest.toml` pin with no tracked or untracked changes under the
inventoried `src` and `include/rive` roots or configured upstream native
generator inputs. Ignored files with an inventoried C++/header suffix are also
rejected because discovery would otherwise read them. New, removed, changed,
duplicate, or malformed records fail with owner-family diagnostics. Never
regenerate merely to make a red gate green: inspect the member/item delta and
update parity evidence or an approved seam first.
