# Binary Import Completion Contract

Date: 2026-06-28

This document narrows the formal goal for `nuxie-schema` and `nuxie-binary`.
It is the operating contract for finishing binary import parity without letting
`nuxie-binary` grow into the whole Rive runtime.

## Formal Goal

Finish `nuxie-schema` and `nuxie-binary` to C++ parity for schema generation and
binary import, as defined by this document.

The goal is complete when Rust can decode the supported C++ fixture corpus into
a normalized imported model with the same byte-level, object-level, property-level,
and import-time relationship behavior that C++ has immediately after file import,
and when the remaining post-import runtime behavior is explicitly documented as
out of scope for `nuxie-binary`.

## Scope Lock

`nuxie-binary` owns file loading. It does not own runtime execution.

The external seam for `nuxie-binary` is:

- Input: `.riv` bytes plus generated schema metadata from `nuxie-schema`.
- Output: a normalized imported file model containing object slots, sparse
  serialized properties, typed decoded views, import status, and finite
  import-time relationship facts.
- Errors: the same import-result categories C++ reports for malformed,
  unsupported, or import-invalid files at the file-loading phase.

Everything past that seam belongs to `nuxie-graph` or later runtime crates unless
it is needed to prove immediate import parity.

## Owned By `nuxie-schema`

`nuxie-schema` is complete for this goal when it covers:

- All `dev/defs` interpretation needed by binary import.
- Object/type metadata, property keys, inherited properties, runtime field kinds,
  and generated default values used by C++ deserializers.
- Abstract/concrete/null-object classification needed by import.
- Generated Rust metadata and tests that compare against the generated C++ shape
  closely enough to catch schema drift.

`nuxie-schema` does not need to generate concrete runtime behavior structs for this
goal. That belongs to runtime crates.

## Owned By `nuxie-binary`

`nuxie-binary` is complete for this goal when it covers:

- Runtime header parsing and version/property-ToC semantics.
- C++-compatible varuint, primitive, string, bytes, bool, color, float, double,
  callback, and bitmask field decoding.
- Known object construction, known property dispatch, inherited stored-property
  lookup, duplicate stored-property overwrite behavior, and generated defaults.
- Unknown object and unknown property skipping, including malformed skip payloads.
- Null object, abstract object, missing object, dropped object, and import-status
  behavior that affects file-loading results.
- Embedded byte decoders that C++ runs during or immediately after import, such as
  encoded ID buffers, mesh triangle indices, CDN UUID strings, and manifest maps.
- Import-stack ownership and resolution facts that C++ establishes while reading
  the file.
- Finite import-time collections and relationships needed for parity comparison,
  such as imported artboards, assets, view models, enums, authored animations,
  state machines, data binds, data converters, file-asset references, shape/paint
  registrations, skin/mesh/path registrations, scroll-physics membership, and
  NSlicer registrations.

These relationships are allowed in `nuxie-binary` only when they are immediate
facts of C++ import or validation. They should remain snapshot/query surfaces over
the imported model, not live runtime schedulers.

## Test-Only Oracles

The C++ probe, source audits, and corpus comparators are allowed to be broader
than the stable public interface of `nuxie-binary`.

They may inspect C++ internals to prove that Rust import facts match C++ import
facts. They should not, by themselves, justify adding more public runtime-like
helpers to `nuxie-binary`.

When adding a new C++ probe assertion, classify it as one of:

- Import-owned: proves a fact `nuxie-binary` must expose or preserve.
- Test-only: protects assumptions but should not become public API.
- Runtime-owned: belongs in `nuxie-graph` or a later runtime crate.
- Out-of-scope: useful later, but not needed for this goal.

## Out Of Scope For `nuxie-binary`

The following are explicitly not part of the `nuxie-schema`/`nuxie-binary`
completion goal:

- Full artboard graph lifecycle after import.
- Dirt propagation and dependency scheduling.
- World/local transform update.
- Layout solving.
- Animation advancement and interpolation over time.
- State-machine execution.
- Data-binding live scheduling, source mutation, target mutation, and frame update
  behavior beyond static import facts and narrowly audited imported-data helpers.
- Data-converter execution beyond what has already been added for parity tests;
  further converter/runtime behavior should move to a runtime crate unless an
  audit proves it changes immediate import results.
- Constraint solving.
- Text shaping/layout.
- Rendering.
- Audio playback.
- Scripting execution.
- Cloning, instancing, and mutable runtime object graphs.

These may be legitimate Rust porting work, but they are not blockers for marking
this goal complete.

## Admission Rule For New `nuxie-binary` Work

Before adding a new helper, relationship view, lifecycle audit, or C++ probe field
to `nuxie-binary`, answer these questions:

1. Does this affect decoding bytes from a `.riv` file?
2. Does this affect whether C++ accepts, rejects, drops, or keeps an object during
   file import?
3. Does this create an immediate import-time relationship fact that exists before
   runtime advancement?
4. Is this needed by the fixture corpus comparison to prove binary import parity?

If the answer is no to all four, do not add it to `nuxie-binary`.

If the answer is yes only because a later runtime crate wants the information,
prefer exposing the smaller imported fact instead of modeling runtime behavior.

## Completion Checklist

The goal can be marked complete when all of these are true:

- `nuxie-schema` has schema parity coverage for the metadata used by binary import.
- `nuxie-binary` passes focused synthetic tests for header, primitive, object,
  property, inheritance, unknown-skip, null/abstract, malformed, and import-status
  behavior.
- The C++ probe comparison passes for the supported fixture corpus.
- Source audits cover the C++ import hooks and immediate post-import validation
  hooks that can change file-loading results or imported relationship facts.
- Each public `RuntimeFile` helper is classified as import-owned, test-supporting,
  or a candidate to move out of `nuxie-binary`.
- Any remaining C++ runtime behavior discovered during audits is recorded under
  "out of scope" or moved to a later `nuxie-graph`/runtime plan.
- The final verification commands pass or their failures are documented with an
  explicit reason they are outside this goal.

Suggested final verification:

```sh
make schema
make test
make cpp-binary-compare
make cpp-compare
```

If `make cpp-compare` includes non-binary runtime graph checks, failures there
should be triaged against this contract before blocking `nuxie-binary` completion.

## What Actually Remains

The remaining work should be closure work:

- Audit the current `nuxie-binary` public surface against this contract.
- Mark each existing helper as import-owned, test-supporting, move-later, or
  accidental runtime scope.
- Document any C++ import hook still not covered by source audit or probe tests.
- Fill only the gaps that fail the completion checklist.
- Avoid adding new runtime lifecycle behavior to `nuxie-binary` unless the admission
  rule classifies it as import-owned.

The likely end state is not a much larger `nuxie-binary`; it is a clearer one.

Current surface audit: see
[`binary-import-surface-audit.md`](binary-import-surface-audit.md).

Current completion matrix: see
[`binary-import-completion-matrix.md`](binary-import-completion-matrix.md).
