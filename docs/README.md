# Docs index

The runtime and renderer ports are complete. Ongoing work preserves the pinned
upstream behavior while integrating product APIs, platforms, and intentional
adaptations. Source, executable tests, and validation harnesses are the living
evidence; completed campaign ledgers and receipts are recoverable from Git
history, not maintained as another implementation authority.

## Source maintenance

- [Runtime source-port method](runtime-bun-style-source-port-plan.md): mechanical
  translation, two separate adversarial passes, then integration validation.
- [Porting manual](PORTING.md): C++→Rust idioms, numeric behavior, and approved
  adaptation boundaries.
- [Parity workflow](PARITY_WORKFLOW.md): shared review and validation principles.
- [Upstream sync](upstream-sync-map.md): reference-pin updates and triage; dated
  sync reports remain in `sync/`.
- [Metal porting guide](METAL_PORTING.md): native ownership and platform rules.
- [Metal validation contract](METAL_RENDERER_VALIDATION.md): oracle hierarchy and
  retained manual platform and source-oracle validation commands.
- [Metal port postmortem](METAL_RENDERER_PORT_POSTMORTEM.md): lessons from the
  39-hour migration, retained to prevent repeated process failures.

## Product and adaptation contracts

- [Runtime boundary](pure-runtime-boundary.md)
- [Player scheduling](player-scheduling-contract.md)
- [Project-data seam](project-data-runtime-seam.md)
- [Browser renderer packaging](browser-renderer-wasm-packaging.md): explicitly
  selected WebGPU and WebGL2 products, both `wasm32-unknown-unknown`.
- [Apple release contract](nux-capi-apple-release.md)
- [Android release contract](nux-capi-android-release.md)
- [Audio parity](audio-core-parity.md)
- [Luau fork](luau-fork.md) and [Symphonia fork](symphonia-fork.md)

## Validation and evidence

- [Golden side-channel format](side-channel-format.md)
- [Renderer parity workflow](renderer-parity-workflow.md)
- [Renderer exactness metrics](renderer-exactness-map.md)
- [SDK size budget](SIZE.md)
- [Upstream microbenchmarks](upstream-microbenchmarks.md)
- [Performance and size evidence](perf-size-evidence.md)
- [Wasm performance evidence](wasm-perf-evidence.md)
- [Composed end-to-end evidence](e2e-composed-evidence.md)
- [Upstream nested-semantics use-after-free report](watch-cpp-nest-semantic-uaf.md)
  and its candidate patch remain live upstream evidence.

`evidence/` retains dated measurements and product-contract evidence. Corpus,
golden, Silver, performance, and browser validation manifests remain executable
inputs. Metal ownership expectations live with the renderer tests at
`crates/nuxie-renderer/tests/fixtures/native_metal/metal-native-owner-expectations.tsv`.

## Historical references

Older source comments and postmortems may cite retired campaign files, phase
identifiers, or commands. Resolve those citations in Git history; they do not
require restoring the ledger systems. In particular, `backend-port-*`,
`metal-port-*`, `render-context-metal-*`, and the completed renderer campaign
plans are historical artifacts. Do not use their former completion statuses as
proof of current source parity.
