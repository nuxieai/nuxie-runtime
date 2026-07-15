# Renderer R3 Exit Audit

Date: 2026-07-14

## Verdict

R3 corpus convergence is complete at `exact=1,377`, `diverges=0`,
`gated=91`, `total=1,468`. Every non-gated entry passes its committed
contract on the macOS Metal CI backend, and every retained gate has a specific
feature, backend/compiler boundary, or harness diagnostic. No
`algorithm-core` placeholder remains.

This verdict establishes an honest corpus classification; it does not retire
the actionable gates or authorize R4. R3.1 owns their burn-down before
performance work.

The corpus manifest has no renderer-selection field separate from its status:
`status = "exact"` is the production Rust-renderer ratchet, while
`status = "gated"` retains the documented boundary.

## Entry Gates

- The GPU semantic-trap audit is closed. Shader provenance, clip-plane routing,
  decoded-image ingress, compiler semantics, and accepted findings are recorded
  in `docs/renderer-status.md`.
- The dual-renderer fuzz-replay entry gate is closed. The deterministic hostile
  stream families, deadlines, findings, and CI smoke gate are recorded in
  `docs/renderer-fuzz-replay.md`.
- The full renderer corpus and V2 regression floor pass: renderer
  `1,377/0/91`, normal V2 `584` exact segments, scripted V2 `35` exact
  segments, and `cargo test --workspace`.

## Retained Gate Taxonomy

| Count | Diagnostic |
| ---: | --- |
| 50 | `metal-webgpu-subpixel-edge-coverage` |
| 10 | `rust-wgpu-msaa-gradient-stroke-composite` |
| 5 | `native-clockwise-atomic-advanced-feather-parity` |
| 5 | `rust-wgpu-msaa-feather-gradient-advanced-blend` |
| 5 | `rust-wgpu-msaa-gradient-edge-residual` |
| 3 | `platform-image-decode-color-profile` |
| 2 | `metal-webgpu-atomic-intermediate-precision` |
| 1 | `dawn-wgpu-msaa-advanced-blend-intermediate-precision` |
| 1 | `dawn-wgpu-msaa-interleaved-feather-color-precision` |
| 1 | `dawn-wgpu-msaa-image-rect-dither-accumulation` |
| 1 | `dawn-wgpu-msaa-stroke-edge-coverage` |
| 1 | `metal-webgpu-fixed-function-color-output` |
| 1 | `reference-harness: C++ Metal does not implement MSAA flush` |
| 1 | `rust-wgpu-msaa-feather-gradient-stroke` |
| 1 | `rust-wgpu-msaa-gradient-advanced-blend` |
| 1 | `rust-wgpu-msaa-gradient-path-clip` |
| 1 | `rust-wgpu-msaa-incompatible-clip-rectangles` |
| 1 | `rust-wgpu-msaa-repeated-path-clipped-strokes` |
| **91** | **Total** |

Operationally, these rows collapse into three groups:

| Rows | Disposition |
| ---: | --- |
| 0 | Reference/oracle harness gap |
| 61 | Reviewed backend, decoder, or precision boundary |
| 30 | Unsupported feature or remaining algorithm-parity boundary |

The actionable set is 30 rows: five prior clockwise-atomic feather boundaries
plus 25 concrete renderer findings exposed by strict replay. The other 61 rows
remain parked unless same-backend evidence exposes a Rust defect.

R3.1 promoted `riv-bullet_man-frame-0-clockwise-atomic` after porting C++'s
incompatible transformed-rectangle fallback to the ordinary clip stack. Its
native Metal comparison is byte-exact under the unchanged `2/32` contract.
R3.1 also promoted `gm-beziers-msaa`: the unchanged row moves from
5,385 pixels/max delta 152 immediately before the dedicated C++ MSAA stroke
depth state (`90c8fd52`) to 8 pixels/max delta 3 immediately after it. The
existing focused duplicate-contour GPU regression pins that self-overdraw
behavior, correcting the stale cubic-raster classification.
The same historical replay closes `gm-cliprectintersections-msaa`: it moves
from 240 pixels/max delta 55 before `90c8fd52` to byte-exact/max delta 1 after
the stroke depth state. The retained edge components were stroke self-overdraw,
not a clip-intersection raster boundary.
R3.1 reclassified `riv-coin-frame-0-clockwise-atomic` after eight draw-prefix
comparisons showed the first excess on a clipped zero-feather ring, not an
advanced-feather draw. Its final 48 outliers form 13 one-pixel-wide path/clip
edge components, largest 12, matching the retained Metal/WebGPU subpixel-edge
boundary without a tolerance change.
R3.1 promoted `riv-bankcard-frame-0-clockwise-atomic` after proving that Rust
hoisted atlas blits ahead of ordinary paths in mixed atomic flushes. Preserving
the authored draw order reduces the native-Metal comparison from 1,485,510
pixels/max delta 20 to 22 pixels/max delta 18, passing the unchanged `2/32`
contract. A focused mixed path-to-atlas regression pins the ordering behavior.

Strict gradient and render-buffer replay now cover the complete 732-case Dawn
registry. One continuous capture preserved all 686 prior PNGs byte-for-byte
and added the 46 previously blocked references. Isolated Rust probes promoted
`riv-interactive_scrolling-frame-0-msaa` byte-exact and converted the other 45
rows into three executable renderer queues. The manifest and
`tools/cpp-atlas-mask-oracle/msaa-reference-inventory.json` now contain no
actionable strict-replay placeholder.

The first ordinary MSAA gradient slice now renders and binds the shared C++
color-ramp texture for direct path fills and strokes, including destination-read
accounting and gradient auxiliary transforms. Seventeen of the 37 gradient
rows promote under unchanged `2/32`. The remaining 20 are no longer grouped
behind a generic gradient-path gate: they are queryable as repeated path-clipped
strokes, gradient advanced blending, feathered gradient strokes, incompatible
clip rectangles, a clipped gradient path, ten gradient stroke-composite rows,
and five identical 45-pixel/max-delta-3 edge residuals.

The C++ MSAA image-mesh path is now ported with the generated WGSL, typed
position/UV/index streams, authored sampler and blend state, path and
rectangle clipping, and draw-order depth. `gm-mesh-msaa` and
`riv-tape-frame-0-msaa` promote under unchanged contracts. Disposable C++
Dawn command-prefix captures isolate Jellyfish: the background and all 19
meshes have no pixels beyond delta 2, while its three subsequent translucent
image rectangles accumulate 3,691/max 3, 8,548/max 4, and 11,988/max 5.
Jellyfish therefore retains a measured image-rectangle dither-accumulation
precision gate rather than an image-mesh feature gate.

### Post-R3.1 Decoder Update (2026-07-15)

The production macOS CoreGraphics JPEG decoder closes the final three
`platform-image-decode-color-profile` rows under their unchanged `2/32`
contracts. Same-backend Metal comparisons against the committed references are
zero/max 0 for `riv-clipping_and_draw_order-frame-0-msaa`, zero pixels over
threshold/max 2 for `riv-data_binding_images_test-frame-0-clockwise-atomic`,
and zero/max 0 for `riv-data_binding_images_test-frame-0-msaa`.
`make renderer-decoder-oracle` independently reports zero decode delta for the
reachable JPEG. The active ratchet is now exact=1,405/diverges=0/gated=63;
the retained reviewed-boundary set falls from 61 to 58. The 91-row taxonomy
above remains the historical R3 exit snapshot.

### R3.1 Repeated Singleton Promotions (2026-07-15)

An independent Sol review rendered four fresh Rust wgpu/Metal rounds for each
of `gm-dstreadshuffle-msaa`, `riv-jellyfish_test-frame-0-msaa`, and
`gm-strokes_poly-msaa`. Every row was byte-stable across those fresh rounds.
Against the unchanged Dawn references and `2/32` contracts, the comparisons
are 0 pixels/max delta 1 at 530x690, 0/max 1 at 2080x2080, and 12/max 46 at
400x400 respectively. Stream hashes, reference hashes, provenance identities,
dimensions, and unique reference ownership all remain valid. Exactly these
three rows advance from gated to exact; the active ratchet is
exact=1,408/diverges=0/gated=60.

### Post-R3.1 Hunter X Adjudication (2026-07-15)

Two native-Metal prefix and preparation investigations localize Hunter X's
remaining 222-pixel/max-delta-18 clockwise-atomic residual to one-pixel
feather edges beginning at command 1,378, an Overlay atlas draw. C++ and Rust
agree on atlas selection, thresholding, batching, and CPU preparation, and
only one alpha sample exceeds delta 2. The row is reclassified from
`native-clockwise-atomic-advanced-feather-parity` to the existing
`metal-webgpu-subpixel-edge-coverage` boundary. Status, reference, `2/32`
contract, and aggregate counts are unchanged; the active taxonomy now has 51
subpixel-edge rows and four advanced-feather rows.

### Post-R3.1 Echo Show Adjudication (2026-07-15)

Fresh native-Metal/Rust prefix replay confirms Echo Show first exceeds its
contract at command 16 on a clipped opaque unfeathered SrcOver path
(`34` pixels over delta 2/max delta 5). Advanced-feather draws can reduce the
residual, while all-SrcOver and zero-feather controls still fail; the largest
late cliffs begin on unfeathered Screen draws. This rejects the prior
advanced-feather-only classification without proving a backend boundary. The
row now carries the actionable
`native-clockwise-atomic-clip-edge-and-composite-parity` diagnostic pending a
same-backend atomic oracle or production fix. Status, reference, `2/32`
contract, and aggregate counts are unchanged; the active taxonomy has 51
subpixel-edge rows, three advanced-feather rows, and one clip-edge/composite
parity row.

### Post-R3.1 Rewards Adjudication (2026-07-15)

A fresh empty-directory C++ Dawn recapture and independent Rust Metal replay
close Rewards' remaining command-21 question. CPU spans and the pinned draw
schedule agree, the unclipped control passes, and only six normalized coverage
words differ. The clip plane differs at 802 sparse edge words; 797 are C++
partial coverage versus Rust full coverage, and all 254 isolated pixels beyond
delta 2 lie exactly on those words. The native full-frame residual remains
1,575/max delta 33 but consists of 1,517 tiny components, largest six pixels.
Rewards is therefore reclassified from advanced-feather parity to the existing
`metal-webgpu-subpixel-edge-coverage` boundary with no status, reference, or
tolerance change. The active taxonomy now has 52 subpixel-edge rows, two
advanced-feather rows, and one clip-edge/composite parity row. Full evidence is
in `docs/renderer-rewards-command21-audit.md`.

### R3.1 Subpixel-Edge Cohort A Claim (2026-07-15)

This cohort is claimed from shared `HEAD` `d244ac33abd76c9571998ad33f37d9bf9394cdcf`.
It is the first 13 IDs in `LC_ALL=C` lexical order among `corpus-r.toml`
entries whose gate reason is exactly `metal-webgpu-subpixel-edge-coverage`,
excluding the already-adjudicated
`riv-rewards_demo-frame-0-clockwise-atomic`. The immutable claim is:

```text
riv-align_target-frame-0-clockwise-atomic
riv-audio_script-frame-0-clockwise-atomic
riv-bindable_artboard_nesty-frame-0-clockwise-atomic
riv-coin-frame-0-clockwise-atomic
riv-collapse_data_binds-frame-0-clockwise-atomic
riv-collapsing_elements-frame-0-clockwise-atomic
riv-component_stateful-frame-0-clockwise-atomic
riv-computed_values_test-frame-0-clockwise-atomic
riv-data_bind_test_cmdq-frame-0-clockwise-atomic
riv-data_binding_artboards_source_test-frame-0-clockwise-atomic
riv-data_binding_artboards_test-frame-0-clockwise-atomic
riv-data_binding_test-frame-0-clockwise-atomic
riv-data_converter_to_number-frame-0-clockwise-atomic
```

### R3.1 Subpixel-Edge Cohort A Adjudication (2026-07-15)

Three fresh serial Rust wgpu/Metal rounds used the read-only replay producer
`/Users/levi/dev/rive-rust/target/debug/renderer-replay`
(`61b8f965c422304ad60daa7868d6d90fe9153de317ba56622f113a87af317ded`).
It was built immediately after `b918ad2a`; the renderer, stream parser,
replay tool, comparator, and lockfile paths are unchanged from that revision
through this cohort's claimed `d244ac33` revision. The sole manifest change is
the excluded Rewards row's later gate reclassification. The producer ran on
Apple M5 Max / macOS 26.4.1. PNG byte hashes vary on eleven rows, but decoded
round-1/round-2 differences are only 1 channel at 1--67 pixels and never reach
the contract's `>2` threshold. All three reference comparisons have the same
results below.

The native C++/Metal FFI producer
`/Users/levi/dev/rive-rust/target/renderer-ffi/debug/renderer-replay`
(`785d96fe6fa19afde22b8993bb0614786f354d3a2fe25a55eb26c438264c796c`)
replayed every claimed stream to the immutable committed reference at exact
`0/0`. Parent review independently byte-compared all 13 outputs with those
references. This validates the reference producer and rules out an image
decode or reference-promotion explanation for the Rust masks.

| Row | Rust result (`>2` pixels/max delta) | `>2` RGB components/area/largest | Alpha `>2` area | Retained boundary |
| --- | --- | --- | --- | --- |
| `align_target` | 77/52 | 26/77/8 | 0 | Transformed glyph and circle edges; unclipped SrcOver. |
| `audio_script` | 251/36 | 120/251/6 | 0 | Seven text-outline edge bands; unclipped SrcOver. |
| `bindable_artboard_nesty` | 79/60 | 26/79/7 | 0 | One transformed glyph-outline edge; unclipped SrcOver. |
| `coin` | 48/58 | 12/48/12 | 26 | Five clip commands plus Overlay; 12 sparse edge components, retained separately from the unclipped family. |
| `collapse_data_binds` | 61/31 | 17/61/6 | 0 | Four glyph outlines plus two transformed rectangle edges; unclipped SrcOver. |
| `collapsing_elements` | 37/31 | 37/37/1 | 0 | One-pixel fractional stripe edges; unclipped SrcOver. |
| `component_stateful` | 35/52 | 17/35/6 | 0 | Two white glyph-outline draws; unclipped SrcOver. |
| `computed_values_test` | 46/24 | 17/46/8 | 0 | Translated glyph-outline components; unclipped SrcOver. |
| `data_bind_test_cmdq` | 793/95 | 300/793/10 | 0 | Many tiny text/vector boundary components; unclipped SrcOver. |
| `data_binding_artboards_source_test` | 130/55 | 57/130/8 | 0 | Small glyph/vector boundary components; unclipped SrcOver. |
| `data_binding_artboards_test` | 65/66 | 33/65/6 | 0 | Tiny text/vector boundary components; unclipped SrcOver. |
| `data_binding_test` | 1,163/96 | 414/1,163/10 | 0 | Many tiny text/vector boundary components; unclipped SrcOver. |
| `data_converter_to_number` | 424/65 | 156/424/9 | 0 | Six text-band boundary components; unclipped SrcOver. |

The control census found no `decodeImage` or image commands in any claimed
stream. Twelve rows have no `clipPath` commands and only `blendMode=3`
(`SrcOver`); Coin is the sole exception, with five clips and `Overlay` in
addition to SrcOver. Thus neither a decoder, a general clip implementation,
nor a general blend/composite implementation can explain the twelve-row
unclipped family. The masks are all sparse subpixel boundaries, but the only
shared explanation left is the reviewed Metal/WebGPU raster coverage boundary,
not a falsifiable Rust algorithm candidate. No source mutation can currently
be paired with an oracle that would separate a production fix from a backend
coverage shift, so this cohort retains
`metal-webgpu-subpixel-edge-coverage` without changing tolerance, reference,
or status.

Artifacts are retained outside the repository at
`/tmp/rive-rust-r31-subpixel-edge-cohort-a-artifacts`: Rust rounds `round-1`,
`round-2`, and `round-3` contain actual and heatmap PNGs; `cpp-native`
contains the exact native-Metal controls.

### R3.1 Subpixel-Edge Cohort C Adjudication (2026-07-15)

This independent audit used a fresh clone at shared `HEAD` `90512824` and
claims exactly the following rows:

```text
riv-listener_view_model-frame-0-clockwise-atomic
riv-local_bounds-frame-0-clockwise-atomic
riv-modifier_test-frame-0-clockwise-atomic
riv-modifier_to_run-frame-0-clockwise-atomic
riv-multi_listeners-frame-0-clockwise-atomic
riv-nested_hug-frame-0-clockwise-atomic
riv-nested_solo-frame-0-clockwise-atomic
riv-nested_solo-frame-1-clockwise-atomic
riv-nested_solo-frame-2-clockwise-atomic
riv-nested_solo-frame-3-clockwise-atomic
riv-nested_solo-frame-4-clockwise-atomic
riv-pointer_exit-frame-0-clockwise-atomic
riv-replace_vm_instance-frame-0-clockwise-atomic
```

Three fresh serial Rust wgpu/Metal rounds used the local producer
`target/debug/renderer-replay`
(`efb79a390dde09d44a7a8085e9aca310d995b42a62212795ab1a629150485e7c`).
Each round has the same committed-reference result below. Decoded round-1 to
round-2 and round-2 to round-3 pixels never exceed delta 1, so the result is
stable at the unchanged `>2` contract even where PNG bytes differ.

The independent C++/Metal FFI producer
`target/renderer-ffi/debug/renderer-replay`
(`a9fc7cfed3064529d29055fd5bad88504c6fc82c430714dcc3e53481daeb3e82`)
replayed each immutable stream and was decoded-pixel exact (`0/0`) against its
committed reference. The audit script fails closed on any nonzero native delta
or any round-to-round Rust pixel above delta 2, identifying the producer hashes
in its report. Parent review independently byte-compared all 13 native outputs
to their committed references. This rules out reference promotion and a native
producer mismatch as explanations for the Rust masks.

| Row | Rust `>2` pixels/max | RGB components/area/largest | Alpha `>2` | Stream command census |
| --- | --- | --- | ---: | --- |
| `listener_view_model` | 118/81 | 45/118/7 | 0 | 0 clips, 0 images, 4 paths, SrcOver only. |
| `local_bounds` | 144/56 | 55/144/8 | 0 | 0 clips, 1 decode + 1 image draw, 11 paths, SrcOver only. |
| `modifier_test` | 151/89 | 67/151/7 | 0 | 1 clip, 0 images, 2 paths, SrcOver only. |
| `modifier_to_run` | 563/91 | 230/563/8 | 0 | 1 clip, 0 images, 8 paths, SrcOver only. |
| `multi_listeners` | 655/60 | 277/655/8 | 0 | 0 clips, 0 images, 12 paths, SrcOver only. |
| `nested_hug` | 285/82 | 120/285/10 | 0 | 0 clips, 0 images, 12 paths, SrcOver only. |
| `nested_solo` frames 0--4 | 42/14 each | 7/42/14 each | 0 each | 1 clip, 0 images, 4 paths, SrcOver only. |
| `pointer_exit` | 44/47 | 40/44/5 | 0 | 0 clips, 0 images, 6 paths, SrcOver only. |
| `replace_vm_instance` | 71/57 | 30/71/6 | 0 | 0 clips, 0 images, 2 paths, SrcOver only. |

The one decoded-image stream, `local_bounds`, is covered by the native exact
control and `make renderer-decoder-oracle`; its production decoder contracts
pass, with no alpha residual. The mix of clipped and unclipped SrcOver-only
path streams also excludes a shared general clip or blend defect. Across all
13 rows, the residuals are RGB-only, sparse connected boundary components
(largest 14 pixels), while the native producer and all decoded round-stability
controls are exact or below threshold. No shared falsifiable Rust defect, and
therefore no mutation-sensitive production fix, was demonstrated. All rows
retain `metal-webgpu-subpixel-edge-coverage` with unchanged status, reference,
and `2/32` contract.

Artifacts are retained outside the repository at
`/private/tmp/rive-rust-r31-subpixel-edge-cohort-c-artifacts`: `round-1`,
`round-2`, and `round-3` contain Rust outputs and heatmaps; `cpp-native`
contains the independent native controls; `analysis/report.json` is the
fail-closed decoded-pixel and stream-command report.

## Reproduction

```sh
make renderer-golden RENDERER_JOBS=4
make golden-compare scripted-golden-compare
cargo test --workspace
rg 'gated = "algorithm-core"' corpus-r.toml
```

The final command must produce no output. Gate counts can be regenerated from
the `[[entry]]` blocks in `corpus-r.toml`; every currently gated block must
contain a nonempty `gated` field. The 91-row table above is the historical R3
exit snapshot, not a current-manifest invariant.
