# Mixed-wrap paint-group resource audit

The geometry-selected ordinary-file paint composition has **quadratic object growth and material import/clone cost**. This benchmark supports further bounded experiments; it does not qualify public wrapping or pixels. No runtime, renderer, schema, dependency or public compiler changes were made.

## Exact construction cost

Generalized the existing `output/mixed-wrap-groups-alpha-r1/src/main.rs` construction from three items to N. It still receives authored item/paint IDs, creates the same ordinary Node/constraint/mask/foreground/draw-rule objects, and encodes an ordinary RIV. There is one painted descendant per flex item, and N≥1.

| Added group | Count |
|---|---:|
| Common origin Node | 1 |
| Leader gates: seven-record difference signal, two-record inversion, two-record mask | 11(N−1) |
| Membership gates and masks | 9N(N−1) |
| Foreground drawable + Fill + color replicas | 3N² |
| Leader and membership ClippingShapes | 2N(N−1) |
| Consecutive DrawRules/DrawTarget chain | 2(N²−1) |
| **Total** | **16N²−12** |

The prototype asserts this formula for every emitted scene. N=3 reproduces the existing 132 extra records; N=8/16/32 add 1,012/4,084/16,372. This fixture family has 6N+9 original records, so totals are 16N²+6N−3. Empty containers should emit no such composition; the formula does not apply at N=0. More painted descendants require another derivation: this is not a general arbitrary-subtree bound. For example, mechanically applying the one-paint formula to 8,192 siblings would add 1,073,741,812 records; the existing authored-element cap cannot bound this composition reasonably.

## Measurements

Each of eight fixed authored scenes was compiled, augmented twice to identical bytes, imported using the unchanged baseline probe, and resized through 320×1400 → 640×1400 → 120×1400 → 320×1400 on the original and its clone. A timed copy of that observer linked the identical immutable runtime/render-api rlibs; its only behavioral additions are monotonic timing reads and stderr output. In three trials per scene, every recorded geometry and stream file matched the stock probe byte for byte (192 timed frames versus 64 stock frames). Sources, commands, binaries, fixtures and raw logs are retained under `output/mixed-wrap-resource-r1`, with hashes in the receipt.

Times are milliseconds. Import and clone medians use three trials. Resize medians/maxima use changed-size steps 1/2/3 and 5/6/7 across those trials; draw medians use all eight recording steps. RSS is process peak measured by macOS `/usr/bin/time -l`, and includes the factory, imported scene, original/clone and accumulated command streams. It is not a per-scene heap allocation measurement.

| N | Flow | Added records | .riv bytes | Import ms | Clone ms | Resize ms (median / max) | Record draw ms | Peak RSS MiB |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 3 | wrap | 132 | 1755 | 1.83 | 0.81 | 0.28 / 0.59 | 0.23 | 18.97 |
| 3 | wrap-reverse | 132 | 1754 | 1.70 | 0.75 | 0.27 / 0.45 | 0.20 | 18.97 |
| 8 | wrap | 1012 | 11516 | 8.91 | 7.47 | 1.53 / 1.92 | 1.44 | 25.41 |
| 8 | wrap-reverse | 1012 | 11511 | 8.58 | 7.04 | 1.51 / 2.06 | 1.45 | 25.30 |
| 16 | wrap | 4084 | 44402 | 65.43 | 60.63 | 5.11 / 5.34 | 5.39 | 45.02 |
| 16 | wrap-reverse | 4084 | 44388 | 64.90 | 60.71 | 5.13 / 5.64 | 5.48 | 44.98 |
| 32 | wrap | 16372 | 176274 | 812.39 | 793.55 | 20.45 / 21.03 | 22.15 | 117.92 |
| 32 | wrap-reverse | 16372 | 176274 | 811.24 | 789.97 | 20.65 / 21.20 | 22.63 | 119.05 |

The machine was an Apple M5 Max (18 logical CPUs, 128 GiB RAM); immutable dependencies are the existing unoptimized baseline toolchain. These are local engineering observations, not controlled cross-device performance guarantees. First-launch and scheduling outliers remain in the raw logs. The first N=3 augmentation process took 227 ms, versus 3.2 ms for the other three-item launch; no claim is made that this startup-inclusive timing models pure compiler lowering. N=32 augmentation launches took about 25.5 ms.

Unaugmented cost controls have the same authored boxes without mixed-wrap objects. For N=3/8/16/32, median imports are 0.51/0.68/0.93/1.46 ms and clones 0.12/0.24/0.40/0.80 ms. Their peak RSS is 16.73/17.41/18.20/19.80 MiB. These controls isolate the added composition's scale, not equivalent CSS output. The amplified import/clone growth warrants investigation before large admission.

## Evidence-based future limits

No universal product performance budget has been specified, so this audit does not install an arbitrary public guard. It establishes explicit candidate envelopes that can be validated on target devices:

- A first bounded interactive experiment can cap a mixed container at **8 items / 64 foreground replicas / 1,012 added records**. That is a measured envelope: resize updates stayed at or below 2.06 ms here, and recording draws were about 1.45 ms median. A scene-wide cap must also limit the sum of each container's expansion; a per-container guard alone would allow many costly groups.
- **16 items / 256 replicas / 4,084 added records** is a separately measured larger tier, with about 65 ms import and 61 ms clone. Resize plus recording draw is roughly 10.5 ms median on this high-end machine, leaving limited room for actual rendering in a hypothetical 16.7 ms frame budget. It needs explicit target-device/lifecycle acceptance rather than automatic promotion.
- **32 items / 1,024 replicas / 16,372 added records** is already unsuitable for a hypothetical 16.7 ms resize frame on this build: resize update alone is about 20.5 ms and recording draw adds about 22 ms. Import and clone each approach 0.8 seconds. This is an evidenced cost limitation of the current composition, not proof that mixed wrapping is impossible.

Any future guard should charge actual emitted records/paint replicas across the scene and report the exceeded bound clearly. These limits are candidates derived from measured points, not qualified guarantees for untested intermediate sizes, multiple containers, deeper subtrees or different devices. A cheaper ordinary-file composition, graph-cost improvements outside this compiler's immutable scope, or a different target performance contract could change them. No larger experiment was needed to establish the present concern.

No Metal rasterization or Chrome comparisons were performed for these generalized resource scenes. Stream equality verifies that timing instrumentation preserves the stock observer's output, not browser semantics or pixels. The failed initial timed-probe build command (wrong source argument replacement) and corrected second command are both retained; this was a harness command correction, not a runtime mutation.
