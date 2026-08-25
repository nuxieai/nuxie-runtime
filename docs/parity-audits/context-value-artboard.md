# `DataBindContextValueArtboard` paired audit

Upstream owner: `src/data_bind/context/context_value_artboard.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

Rust owner: `crates/nuxie-runtime/src/data_bind/context/context_value_artboard.rs`,
with mounted-occurrence lifecycle in `data_bind/data_bind_context.rs` and
`artboard.rs`.

Verdict: adapted and behaviorally equivalent under Rust's retained-cell and
arena ownership rules.

row_id: "B6-0160"; upstream: "src/data_bind/context/context_value_artboard.cpp"; verdict: ADAPTED;

- Source synchronization is push-driven through the retained view-model cell;
  there is no generation poll or reconstructed candidate in this owner.
- Integer-backed source compatibility is retained by `matching`.
- Non-referencer targets use the typed integer target adapters.
- Nested Artboard referencers dispatch through `apply_to_nested_host`. Live
  sources replace the mounted occurrence without writing the generated
  `artboardId`; file-backed sources follow the generated-id path after the
  initial forced mount.
- Explicit `-1`/`u32::MAX` clears the mounted child. Unresolved and ancestor
  targets preserve the outgoing child. Cross-file live sources preserve their
  source-file identity.

Verification:

- `cargo test -p nuxie-runtime --lib artboard_data_bind::tests`: 44 passed.
- `cargo test -p nuxie-runtime --lib nested_artboard`: 14 passed.

This current-pin audit supersedes B6-0160's historical divergent verdict,
which was recorded while the RB-1 retained-cell migration was still in flight.
