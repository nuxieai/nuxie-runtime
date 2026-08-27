# Literal runtime translation plan

Decision date: 2026-08-27

Pinned authority: Rive runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

This is a fresh mechanical translation. The pinned upstream source is the
behavioral authority. The existing Rust runtime is reference material for the
later integration review; it is not an authority for what the translation
should do.

Continue to ignore and not use the `implement` and `tdd` skills.

## Translation

Translate every pinned C++ implementation and its primary header into one
corresponding Rust source file. Mirror the upstream source path beneath:

`crates/nuxie-runtime/src/mechanical_port/source/`

For example, `src/animation/foo.cpp` and `include/rive/animation/foo.hpp`
become `mechanical_port/source/animation/foo.rs`. Header-only owners receive a
corresponding Rust file as well. Generated owners may be emitted mechanically,
but their resulting Rust owner must still exist in the mirrored tree.

The first pass reads the upstream pair, not the old Rust implementation. It
translates the complete owner: types, retained state, defaults, constructors,
methods, meaningful branches, mutation and callback order, lifecycle, clone
and reset behavior, error behavior, and conditional compilation.

Do not build, test, redesign, refactor, certify, or create evidence while the
translation pass is running. Do not add placeholder bodies or comment-only
owners. The filesystem is the progress record: an upstream pair is translated
when its complete Rust counterpart exists. Completeness is derived directly by
comparing the pinned source tree with the mirrored Rust tree; there is no
manually maintained source ledger.

## Adversarial review 1: source equivalence

After the complete mirrored tree exists, reviewers receive:

- the pinned `.hpp`;
- the pinned `.cpp`;
- the translated `.rs`.

They look only for missing or altered source semantics: omitted state or
bodies, changed defaults, branches, ordering, integer and floating-point
behavior, error paths, lifecycle, conditional compilation, or virtual
dispatch. They do not redesign the Rust or use tests to invent behavior.

## Adversarial review 2: Rust integration and adaptations

After source-equivalence corrections, reviewers additionally receive the old
Rust files that the translated owner will replace and the small approved
adaptation set. They identify Rust-specific integration work without changing
the upstream observable contract.

Approved integration boundaries include safe Rust ownership, Taffy in place
of Yoga, and the existing Rust-native audio, scripting, and text stacks. An
approved library or backend replacement is not permission to omit the Rive
owner's surrounding state, ordering, callbacks, or error behavior.

This pass also checks lifetimes, aliasing, thread-safety, public API continuity,
crate boundaries, FFI representation, and whether old packed Rust code contains
necessary host integration that must be retained around the new owner.

## Integration

Apply both review passes, then make the mirrored owners live. Move or route the
translated modules into their final crates, preserve required public Rust
interfaces, and delete the superseded packed implementations. The old parity
implementation may help diagnose integration failures, but a disagreement is
resolved from pinned upstream source or an approved adaptation.

Only after a translated owner is live may its old implementation be removed.
Do not retain the old path as a hidden fallback.

## Validation

Once the complete translated tree is integrated:

1. compile the affected workspace and supported targets;
2. run the complete translated upstream unit-test suite;
3. run the existing C++/Rust differentials, silver corpus, rendering corpus,
   lifecycle checks, and larger validation harnesses;
4. diagnose every failure by returning to the exact upstream/Rust pair and
   the approved integration boundary;
5. fix translation or integration mistakes rather than inventing new runtime
   behavior;
6. perform final source-equivalence and integration rereviews over the frozen
   bytes that passed validation.

## Completion

The campaign is complete when every pinned source/header owner has a complete
Rust counterpart, both adversarial passes are resolved, the new owners are the
only live implementation, and the full validation suite has reached its
truthful supported-platform result with every approved adaptation explicit in
code.
