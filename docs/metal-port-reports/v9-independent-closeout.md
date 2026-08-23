# V9 — independent closeout

Status: GREEN on 2026-08-22.

Two independent final rereviews were performed on the frozen post-V4 bytes against pinned upstream `4ac7b32798da0482e441ef09304dc3b480ed3ee5`:

- Source/spec review: CLEAN, zero P0–P3 findings.
- Ownership/lifetime/ABI review: CLEAN, zero P0–P3 findings.

The closing defect was a mechanically incomplete Objective-C selector call: `generateMipmapsForTexture:` received only the texture handle even though the execution adapter requires argument 0 to be the blit-encoder receiver and argument 1 to be the texture. The corrected translation passes `[encoder, texture]`, retains the authored pre-pass placement, then performs `endEncoding`, clears the dirty flag (including nil messaging), and releases the encoder in source order.

Closeout evidence:

- Focused selector-role regression: 1/1.
- Downscaled transparent image: byte-exact, zero differing pixels, maximum channel delta 0.
- V0–V8: all GREEN.
- Strict `-D warnings`: renderer product, ORE default, and ORE tools all GREEN.
- Machine campaign tools: 73/73; C/C++ assert authority: 41/41 units and 522/328/1484 assertion census.
- `cargo fmt --all -- --check` and `git diff --check`: GREEN.

Reviewed artifact digests:

- Metal implementation: `6de9771955c83193d8e2b681e43faeb7f3463f1da33754ccd1a1588c856b6844`.
- Objc2 execution adapter: `2c139fe8fc8d2531cc424862ff46cf260294d9cf9929794f41185236f77e9aa5`.

No cleanup, idiomatic redesign, or tolerance widening was used to obtain closure.
