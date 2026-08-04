# VFIX lane 3 report: component-list settlement and mounted transforms

Date: 2026-08-03
Branch: `levi/vfix-component-list`
Pinned upstream: `rive-runtime@4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Outcome

V14, V18, V19, and V39 are exact in the ordinary golden lane against the
pinned C++ runtime:

| Row | Corpus entry | Ordinary result | Scripted classification |
|---|---|---|---|
| V14 | `artboard_list_overrides` | exact | diverges |
| V18 | `clipping_and_draw_order` | exact | exact |
| V19 | `component_list_child_origin` | exact | diverges |
| V39 | `virtualize_blendmode` | exact | diverges |

The focused ordinary comparison reports 4 exact entries, 12 exact segments,
12 side-channel segments, and no divergences. V14, V19, and V39 retain their
independent scripted draw differences through an explicit scripted-only status;
their settlement returns are exact in both execution modes.

## Main integration and commit sequence

The V12-bearing `origin/main` commit `d6a8984e` was integrated before the lane
changes and its five-pass, per-pass-reset settlement behavior was confirmed.
The branch later integrated the then-current `origin/main` at `bbc933aa` in
merge commit `0e054197`.

The requested implementation sequence is preserved as two commits:

1. `f6480545 Fix component-list settlement parity` closes V14, V19, and V39.
2. `e10a2c80 Fix mounted component-list child transforms` closes V18.

## V14, V19, and V39: settlement composition

The component-list retry path had three independently visible sources of false
keep-going state:

- A successful nested `tryChangeState` probe counted directly as a changed row.
  Pinned C++ composes only the following nested `advance` return; transition
  dirt remains responsible for requesting a settlement pass.
- An `ArtboardComponentListOverride` dirtied its host even when the override had
  no mounted rows. The C++ callback only walks retained `m_artboards`, so an
  empty occurrence is clean.
- A computed scroll setter fell back to dirtying the parent when an intent was
  unresolved or clamped to the retained offset. It now bubbles dirt only when
  the effective scroll offset changes.

The corpus entries were promoted to ordinary `exact`. Golden comparison gained
a symmetric `scripted-status:diverges` override so the scripted gate can retain
the pre-existing draw classifications without weakening ordinary exactness.

## V18: mounted-child transform

Pinned C++ `Artboard::onAddedClean` clears a mounted Artboard root's authored
`x` and `y` before component-list bindings and state machines run. Component-list
row construction now performs the same clean initialization while preserving
later live animation and data-binding writes consumed by `worldBounds()`.

The V18 path also required two directly related parity corrections:

- `SymbolListIndex` values now target numeric Double/Uint artboard properties,
  matching C++ `ContextValueSymbolListIndex` behavior used by row `itemIndex`.
- Ellipse control points use the pinned C++ fused multiply-add rounding boundary,
  making the resulting path stream byte-exact.

Regression coverage exercises empty override cleanliness, unchanged clamped
scroll intent, mounted root identity, row-index child data binds, and fused
ellipse rounding.

## Verification

- Pinned C++ revision check passed:
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `cargo test -p nuxie-runtime` passed (961 unit tests plus all integration
  suites).
- `cargo test -p nuxie --features scripting` passed (255 unit tests passed,
  1 ignored, plus all integration suites).
- Focused ordinary comparison passed: 4 exact entries, 12 exact segments, 12
  side-channel segments, and zero failed entries.
- `make scripted-golden-compare` passed with 363 entries, 334 exact entries,
  1,090 exact segments, 1,085 side-channel segments, 22 registered divergences,
  7 registered not-yet rows, and zero failed entries. The final pass used a
  clean workspace-local scripted runner via `SCRIPTED_RUST_GOLDEN_RUNNER` after
  the cached target artifact began exiting with SIGKILL across unrelated rows.
- `make runtime-frame-loop-port-check` passed (125 tests and all manifest,
  layout-style, and ownership checks).
- `make rust-attribution-check` passed (10 tests and complete Rust attribution
  coverage).
- `git diff --check bbc933aa...HEAD` passed.

## Review and workspace preservation

The required two-axis closeout review found no documented-standards violations
and no missing, incorrect, or out-of-scope implementation behavior. Its sole
initial spec finding was the absence of this report, now resolved.

Pre-existing formatting-only changes in renderer/runtime files remain
uncommitted and untouched. The pre-existing local `docs/v-row-triage.md` copy
also remains untracked. No `/tmp` path was used.
