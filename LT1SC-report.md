# LT1SC lane report

## Status

**PARTIAL — side-channel delivered; one fixture differential exact; four source
rows intentionally remain pending.**

LT1SC extends the OR-1/OR-2 stream with the complete semantic tree diff and
semantic action/focus outcomes. The C++ and Rust runners use the same fixture
context selection, and the comparator supports an exact semantic projection
for fixtures whose ordinary drawing stream is outside this ticket.

The external nested-boundary focus/provider case is exact. The two other
required fixture probes expose real Rust runtime residuals. Because the ticket
requires green cited evidence before promotion, none of B6-0327 through
B6-0330 is promoted.

## Provenance

- Lane: `levi/pend9-lt1-side-channel`.
- Start point: `121230e1`, which was the then-current `origin/main` when the
  lane's required first check ran. The shared `origin/main` ref advanced while
  this lane was active; the lane was not rebased afterward.
- Pinned C++ runtime: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- The initial fixture refresh and pin check completed with
  `rsync -a /Users/levi/dev/nuxie-runtime/fixtures/ fixtures/ && make fixtures`.
- The pinned runtime has no tracked modifications from this lane. Build outputs
  and other pre-existing untracked files remain outside this worktree.

## Delivered commits

- `fe0abb34` — specify the complete semantic side-channel records.
- `49975123` — emit semantic oracle records from the pinned C++ runner.
- `389c6fb3` — specify side-channel-only fixture context selection.
- `cc1eaa49` — scope exact semantic fixture projections.
- `7d98e798` — select authored fixture defaults in the C++ runner.
- `4b00f0ef` — emit and compare the Rust semantic fixture channels.

The canonical OR-1/OR-2 document in this repository is
`docs/side-channel-format.md` (`docs/side-channel-spec.md` did not exist). It
now defines all seven `SemanticsDiff` fields, added/updated node payloads,
action/focus outcomes, authored default-ViewModel selection, and the exact
semantic-only comparison projection.

The C++ change is emitter-side only in `tools/golden-runner`; no pinned
upstream source was edited. Both runners still emit the ordinary stream. The
comparator projection is limited to `advance`, `semantics`, `semanticAction`,
and `semanticFocus`, remains line-exact, and introduces no tolerance or
divergence carve-out.

The emitter-side-only restriction applies to the pinned C++ checkout. The
small Rust runtime changes are required code-port enablers for an honest
oracle: retained nested/list boundary nodes mirror pinned
`src/artboard.cpp:2155-2269` and `src/artboard_component_list.cpp:683-688`,
while accepting `SemanticData`/`SemanticInput` as static Text metadata siblings
prevents them from being mistaken for unsupported draw content. They are
focused-tested and do not add a fabricated comparator behavior.

## Differential evidence

### Exact: external nested-boundary focus/provider

Corpus row `semantic_provider_focus_lt1` is exact for all three samples. Its
projection matches nested boundary ids `1, 4, 7, 10, 13, 16`, labels
`Element 1` through `Element 5`, root/parent/sibling structure, the focus
outcome and Focused flag, all initial bounds, and five post-scroll geometry
updates.

Focused comparator summary:

```text
summary entries=1 exact=1 exact-segments=3 side-channel-segments=3
```

### Red: data_binding_lists action/update

Corpus row `semantic_data_binding_action_lt1` remains `not-yet` because its
exact semantic differential reports:

- initial C++ `Selected=1` where Rust reports `Selected=0`;
- non-degenerate C++ Text bounds where Rust reports degenerate points; and
- after `semanticAction nodeId=2 tap`, C++ removes ids
  `[6,5,9,8,12,11,15,14]` and changes Selected state, while Rust removes no
  nodes and emits unrelated geometry changes.

This blocks promotion of `semantic_data.cpp` and contributes to the retained
`semantic_manager.cpp` pending status.

### Red: Simpsons full-diff Text inference/provider

Corpus row `semantic_text_inference_lt1` remains `not-yet`. Initial inferred
labels, ids, and hierarchy are exact, proving the Text inference path is
exercised, but the required full diff is not exact:

- the root list bounds differ (`128..1936` in C++ versus `135..1929` in Rust);
- after the semantic action, C++ reports moves from pre-layout bounds while
  Rust reports the new-layout bounds; and
- C++ updates the Selected flags for ids 4 and 6 while Rust does not.

This blocks promotion of `semantic_inference_registry.cpp`; the provider/root
bounds and manager updates also keep `semantic_provider.cpp` and
`semantic_manager.cpp` pending.

## Gates

Focused lane gates are green:

```text
cargo check -p rust-golden-runner -p nuxie-render-api
cargo test -p nuxie-render-api --test side_channel                 # 3/3
cargo test -p nuxie-runtime --test semantic_focus_runtime          # 4/4
cargo test -p nuxie-runtime semantic_metadata_siblings             # 1/1
cargo test -p rust-golden-runner side_channel_is_stream_mode_only  # 1/1
cargo test -p golden-compare --bin golden-compare semantic_fixture_projection # 1/1
make port-manifest-check                                           # 20/20; 456/456 rows
make rust-attribution-check                                        # 10/10; all sources classified
```

Corpus-wide `make golden-compare` is green:

```text
golden-compare summary: entries=356 exact=325 exact-segments=673
side-channel-segments=672 diverges=0 unsupported-feature=0 not-yet=31
```

The new exact focus row accounts for three exact/side-channel segments. The
two red semantic evidence rows are honestly parked as `not-yet`; both C++
oracle streams completed successfully in the same corpus run.

The orchestrator's full battery was not run, per lane discipline.

## Ledger disposition and residue

| Row | Disposition | Evidence |
| --- | --- | --- |
| B6-0327 `semantic_data.cpp` | pending | action/update differential red |
| B6-0328 `semantic_inference_registry.cpp` | pending | labels exact, required full diff red |
| B6-0329 `semantic_manager.cpp` | pending | nested manager boundary exact; two fixture diffs red |
| B6-0330 `semantic_provider.cpp` | pending | focus bounds exact; Simpsons provider bounds red |

The four row-status pending floors are unchanged and were not loosened; the
green ordinary corpus floor tightens from 324/670 to 325/673 exact entries and
segments, with side-channel segments tightening to 672. Evidence/residue is
recorded in the source correspondence ledger, generated and generator-owned
port manifests, test correspondence ledger, F6 gap/closeout documents, and
the four Rust attribution headers. The existing justified crate-boundary
scatter exception remains limited to `semantic_manager.rs` plus
`semantic_runtime_tree.rs`; the scatter ratchet was not increased.

No comparison tolerance changed. No filed side-channel divergence was added.
No lane command or artifact used `/tmp`.
