# Observe wrapping clips without changing the runtime

Read-only investigation, 2026-09-12. This document is the only new file written for this investigation. No build, new native observation, renderer execution or capture was launched. The plan complements `wrapped-mask-coverage-audit.md`; it is not a clip or public wrapping qualification.

## Existing observer is sufficient

`validation/baseline-probe.rs` already imports ordinary RIV bytes through `File::import`, settles the original, creates one clone and resizes/draws both occurrences. It has no compiler dependency and consumes no map. Each `frame-N.stream` contains full raw path snapshots at every `clipPath`, plus all save/restore/transform commands. The geometry JSON alone is insufficient: layout world matrices do not show the renderer state at clipping proxies.

The streams are cumulative. Parse the selected `frames[N]` from `frame-N.stream`, not every command in the file as one frame, and retain the explicit frame index for replay. `frameSize` is the ceiling of the requested viewport while the original fractional size remains in `frames.json` and the artboard raw clip. Do not confuse the pixel canvas bounds with the raw artboard clip.

The unchanged recorder serializes each clip path immediately, including verbs and points, so mutable path IDs need not and must not be used as object-ID identities. One path ID may have different snapshots later. Use frame/command occurrence IDs; join a source map only after import when annotating a report.

The existing public typed parser exposes `RenderStream.frames`, `Frame.commands`, `Command::{Save,Restore,Transform,ClipPath,DrawPath,...}`, and `Path.raw_path` in `crates/nuxie-render-stream/src/lib.rs:13–99`. Thus an independent executable can parse exact f32 coordinates without inventing a stream grammar. The native renderer's `RiveRenderer::IsAABB` is inside private `mechanical_port` (`native_root.rs:39`), so a downstream observer cannot call it directly through the public crate API. Do not expose it by modifying the renderer.

## Immediate commands

From `tools/html-to-riv`, after the candidate has been frozen and its RIV hash recorded, the existing executable can produce fresh stream evidence:

```sh
output/immutable-baseline-toolchain-r2/baseline-probe CANDIDATE.riv output/wrapped-clip-observe-r1 400x80 280.5x160.25 400x80
```

`CANDIDATE.riv` is a placeholder for the actual newly frozen composition, not a runnable existing artifact. Use a fresh output name. This example deliberately revisits the initial size; extend the actual fixture schedule for both axes, wrap directions, viewport edges, overflow and a maximum-bound case. The maximum-size case can be stream-only before allocating a 16384² pixel image.

Replay a chosen occurrence using the already frozen renderer:

```sh
output/immutable-baseline-toolchain-r2/renderer-replay --stream output/wrapped-clip-observe-r1/frame-0.stream --output output/wrapped-clip-observe-r1/frame-0.native.png --backend rust-metal --mode clockwise-atomic --frame 0 --clear 0xffffffff
```

Use a second output/background and the established Chrome comparison driver for actual pixel qualification. `--command-limit` truncates the command sequence; it does not expose clip rectangles or GPU matrices and is not needed to establish the stream-state observations. Do not use a truncated render as the final scene result.

## Minimal next validator

For the current pinned campaign a strict offline stream validator is enough; no new importing executable is required. It should:

1. Load `frames.json`, each exact stream, and the matching frame; initialize identity state and a checked save stack per frame. Reject unmatched restore, unbalanced stack, nonfinite coordinates, malformed or unrecognized commands affecting state.
2. At the first artboard clip, preserve the raw vertices, AABB, renderer matrix and viewport relationship. Check that it is the zero-radius top-left artboard rectangle, independently of later mask geometry.
3. For every clip occurrence, apply the source predicate exactly: at least four initial Move/Line/Line/Line verbs and four points; every point after index3 equals point0; either alternating x/y edge orientation matches; bounds are point0/point2 minima/maxima. Preserve the full verbs/points in evidence. Do not replace this with a generic bounding-box test: a curved path can share an AABB without taking the rectangle route.
4. Record the renderer matrix at the clip command. In this profile all transforms before a mask clip are identity or restored to the outer identity. An initially narrow validator can require exact identity matrix operands whenever a clip is established and reject an unsupported state rather than approximate multiplication. Paint translation commands inside a balanced saved scope are permitted, but must not leak into the next clip. For general transforms, use a tiny Rust executable with the baseline typed parser and source-shaped f32 matrix arithmetic (see below).
5. Maintain saved active clip state, with coordinatewise intersection for identical matrices. Assert active mask bounds `[0,0,32768,32768]` preserve the artboard rectangle, and inactive bounds shifted by65536 in the selected axis make its intersection empty. A subsequent clip inside an already empty state is still observed in the stream; report it as suppressed by existing empty coverage, since the actual renderer returns before AABB recognition in that state (`rive_renderer_cpp.rs:1626–1628`). Do not falsely claim every submitted clip invoked the native AABB branch.
6. At each draw, report the inherited clip chain and whether it is empty. Require an inactive replica with geometry crossing the viewport and no surviving contribution. Compare original/clone/revisited occurrences semantically, not by recorder resource IDs.

This validator establishes the ordinary runtime command inputs plus a source-derived rectangle-route conclusion. It does not instrument GPU internal state and must not label its inferred intersections as directly measured GPU state. Native pixel comparisons remain necessary for the complete integration argument.

For arbitrary finite matrices, implement a NEW standalone validation executable, for example `wrapped-clip-stream-observer.rs`, reading STREAM and FRAME only. Link the existing baseline `nuxie_render_stream` and `nuxie_render_api` rlibs selected from `build-messages.jsonl`. Reproduce `rive_renderer_cpp.rs:871–882` exactly: f32 products, `mul_add`, then the separate final translation add; transformation is `current * operand` at :1550–1552. Rust f32 parsing preserves serialized round-trips. A Python/f64 affine multiply is not an exact implementation. Bind a copied AABB predicate and matrix formula to the authoritative source hash/line ranges and test negative controls (curve, extra nonclosing vertex, escaped translation, malformed stack). This remains a read-only observation executable, not a renderer adapter, and is not linked into product output.

## Existing evidence inspected

Historical stream `output/wrapped-sizing-snapped-r1/render/combined-slot-row-wrap-flex-start-percentage-normal-main/probe/frame-0.stream` has SHA256 `5bb007b35d788179e57b6f6c38674809cf21c3e8ce9a205b9c36300644ae117b`. It visibly contains:

- Artboard clip at line71: four corners from signed-zero origin through400×80, under identity at line69.
- Active mask at line93: `(0,0),(32768,0),(32768,32768),(0,32768),(0,0)` under the restored outer state. Visible paint translation `[1,0,0,1,60,15]` occurs later at line95 in a nested save, then restores before the next mask.
- Inactive mask at line129: cross-y65536 through98304.
- Repeated clips at lines145/147 before a single draw scope.

These concrete snapshots support the proposed observation method. They use the older candidate and are not evidence for the new derived epsilon or anchored composition. No exhaustive scan or general state assertion was performed in this investigation.

## Identity and attribution requirements

Verified existing binary SHA256 values:

- Baseline probe: `7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a`.
- Renderer replay: `276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f`.
- Current baseline probe source: `91adadd3c6d2408d038eac082825519513a1fd506045397bc62028f3a6efdda8`.
- AABB source `renderer/include/rive/renderer/rive_renderer_hpp.rs`: `4f616bfde04b350ae76c4ed24931cd4c1682fe38052cb4daa8edf9b686dade21`.
- Renderer implementation `renderer/src/rive_renderer_cpp.rs`: `b3b7f6751a0e3cc5d5cd67855530ba12daf01f8107ce29d8b9a83865842c5004`.

The renderer source paths above are beneath `crates/nuxie-renderer/src/mechanical_port/source/`.

Preserve the existing baseline toolchain manifest, source identity, Cargo build messages, dependency feature tree, root Cargo.toml/lock hashes and commands. Run `validation/check-target-runtime.py` before and after the campaign. `validation/build-baseline.py` is the established baseline-owned build recipe if artifacts must be recreated: root `cargo build --locked -p rust-golden-runner -p renderer-replay --features renderer-replay/native-metal`, then direct rustc linkage to the uniquely selected Cargo artifacts. Never choose an arbitrary globbed rlib, use compiler-local dependency resolution, add a patch override, or edit root manifests for an observer. Preserve fresh observer source/command/rustc version/output/binary hashes and verify any reused rlibs against build identity.

For each fresh observation bind candidate source/build identity, RIV bytes, immutable observer binary, viewport sequence, original/clone occurrence, exact stream and report bytes, then renderer/Chrome identity for pixels. AABB support is a conditional renderer fact: non-MSAA frame support, equal clip matrices, active artboard clipping, and the closed rectangular shape are required. The earlier mask audit supplies source anchors for intersection-before-AA; neither this plan nor a command parser proves MSAA or arbitrary-host-transform qualification.
