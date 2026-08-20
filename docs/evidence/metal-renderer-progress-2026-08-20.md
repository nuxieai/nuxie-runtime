# Metal renderer progress report — 2026-08-20

Upstream pin: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Process correction

Commit `106b19559` adopts the whole-renderer plan in
`docs/METAL_RENDERER_PORT_PLAN.md`. Mechanical source translation now precedes
compiler and behavior queues. Named fixtures and rendering features no longer
select implementation work.

## Source ledger at plan adoption

| Source | Ported ranges | Partial ranges | Missing ranges |
| --- | ---: | ---: | ---: |
| `render_context_metal_impl.h` | 5 | 3 | 0 |
| `render_context_metal_impl.mm` | 30 | 14 | 4 |

Both primary source rows remain `in-progress`.

## Last fully verified checkpoint

The last committed source checkpoint before preparation resumed is
`34bf0b7e4` (`Port Metal common draw pass`). Its recorded verification was:

```text
MTL_DEBUG_LAYER=1 MTL_SHADER_VALIDATION=1 cargo test \
  -p nuxie-renderer --features native-metal-experimental \
  --test native_metal_tracer
result: 24 passed, 0 failed

make renderer-metal-atomic-oracle-tracer
result: 8 selected, 8 exact, 8 byte-exact, 0 divergent

make renderer-native-metal-tracer-binary
result: rooted Mach-O passed no-WGPU/Naga/WGSL reachability scan
artifact size: 802,576 bytes

make metal-port-check
result: passed
```

These results are regression evidence. They do not imply that the complete
Metal implementation is ported.

## Held work

The current mixed feather-atlas edits are uncommitted. They are not a completed
feature slice and will not be promoted independently. They remain subject to
the complete source and ownership audit.

## Dashboard update contract

`make metal-port-progress` regenerates `docs/metal-renderer-progress.html` from:

- `docs/render-context-metal-file-map.tsv`;
- `docs/metal-port-manifest.toml`;
- `docs/metal-port-ownership.toml`;
- `docs/metal-renderer-progress.toml`;
- the checked-in Metal tracer corpus manifests and reference images.

`make metal-port-progress-check` fails when the checked-in page is stale.
