# Runtime performance closeout (2026-07-19)

The pure-Rust runtime is at parity with, and faster than, the clean upstream
C++ runtime in the focused release/null-renderer hot-loop gate.

## Reproducible inputs

- Rust commit: `d4adb01d9e0c950c35a8a3bc6fec78653913f8f8`
- Rust tree: `d704445ec68cd764652e84a1abf986f3e8798f9c`
- Rust runner SHA-256:
  `f8f883c190ada2a177c06863b4399ad097dc3e67af01192685b7ae21c57a42d8`
- Upstream C++ commit:
  `d788e8ec6e8b598526607d6a1e8818e8b637b60c`
- C++ runner SHA-256:
  `1d97cb01c22eee293bbba0e60a4c01cd91f70790a4e7d801c847de159558cf91`
- Corpus IDs: `advance_blend_mode`, `ai_assitant`, `align_target`,
  `animated_clipping`, `animation_reset_cases`, and `spotify_kids_demo`
  (11 sampled segments)
- Settings: release runners, null renderer, runner hot loop,
  `iterations=10`, `warmups=3`, `aggregate=min`, and
  `benchmark_repeat=10000`
- Runner ordering: three `cpp-first` reports and three `rust-first` reports.
  `perf-compare` records the order in new reports, and
  `PERF_RUNNER_ORDER=rust-first make perf-hot-loop` selects the reverse order.
- Acceptance: aggregate Rust/C++ must be `<= 1.0`.

The measurements ran on an Apple M5 Max with 128 GiB RAM, macOS 26.4.1
(build 25E253), AC power, Rust 1.94.0, and Apple clang 21.0.0. The report
metadata also records both source commits, the Rust tree, host, OS, and power
source. The runner hashes above make the executable inputs independent of the
temporary build paths.

## Final performance evidence

Six independent, order-balanced runs from the same source tree and runner
binary passed:

| run | order | C++ selected ms sum | Rust selected ms sum | Rust/C++ | result |
| --- | --- | ---: | ---: | ---: | --- |
| 1 | C++ first | 41.138360 | 36.971167 | 0.898703 | pass |
| 2 | C++ first | 40.753377 | 36.788542 | 0.902711 | pass |
| 3 | C++ first | 40.491337 | 36.310873 | 0.896757 | pass |
| 4 | Rust first | 39.413646 | 36.042833 | 0.914476 | pass |
| 5 | Rust first | 39.755193 | 36.130169 | 0.908816 | pass |
| 6 | Rust first | 39.742857 | 36.253208 | 0.912194 | pass |

The observed range is 0.897-0.914x, or 8.6-10.3% faster than C++ for this
gate. All three reverse-order runs passed, so the parity result does not depend
on always launching C++ first. Individual small fixtures may still favor C++;
the acceptance metric is the declared aggregate workload, matching the
repository's performance ratchet.

The six JSON report SHA-256 digests, in table order, are:

1. `129466ac62714c1aa037876414b6702a58da42d0cdca1b43c649d03d044922f7`
2. `dc294546d8ed91eab779edba20b1b16fd7b6b2d5e2e993d6f1e87b93e9279dcb`
3. `59b9f5d9420477cf48f5ff1277f60c06d6229ac5f98a95ce5dd9d5a0e5b5c655`
4. `5f18de215494d17f255649b9ae59834a832f4a197b6db1c078db8772beeae3d9`
5. `6c4104d43a478abd7ce9e18ca43a679182cb7e0be1fe2c2f5c6505077007788a`
6. `78bc68d9be82017d1bfcd00c7ed91ec5d4aafc79e69ff2918ddfa4b1c608f12f`

Reports 1-3 predate the explicit `runner_order` JSON field; their command path
was the then-hard-coded C++-first order. Reports 4-6 record
`runner_order=rust-first` directly.

## Exactness evidence

The same Rust tree passes both comparison modes against the candidate C++
checkout:

- default: 317/317 exact entries and 647/647 exact segments;
- forced scripting: 317/317 exact entries and 647/647 exact segments;
- both: 0 divergences, 0 unsupported features, and 0 not-yet entries.

The default and forced-scripted comparison log SHA-256 digests are
`65b1f0e28aa6dfa1155ad338180e0964c80e1e509c4c0c1d1c58a32ec709a4c5`
and `2bbf3c90bf206fb56c3c6fe18aeed53b08836792de0ab80d6d1fb13a36de309c`,
respectively.

The scripting-enabled Rust runner SHA-256 is
`8cc74eba0531170139f8819876c48805e580b04ae83eb0a51a96fdd0b6978d41`;
the scripting-enabled C++ runner SHA-256 is
`81074c42608b4c501d11626a1419a6c69c37c1afac7b578049981d80e48ff39c`.

## Accepted optimizations

The closeout keeps only changes that survived exactness and order-balanced
A/B checks:

1. Transition scans are skipped when no outer probe is owed. Continuous blend
   states still advance, while host-input writes and other transition-producing
   events explicitly schedule the post-update probe. This restores the C++
   separation between "keep advancing" and "evaluate transitions again."
2. The common one-solid-fill draw path retains one render path directly by
   shape local, matching C++ `ShapePaintPath` ownership and bypassing generic
   paint/effect path lookup. Revision and fill-rule changes still rebuild or
   reconfigure the retained object exactly.
3. Transition evaluation borrows one immutable context per layer tick instead
   of repeatedly passing roughly two dozen independent references through
   every state/transition/condition call. This removes hot-call plumbing
   without changing mutable input, trigger, event, or action ownership.

Two broader cache prototypes were rejected and backed out. Flattening nested
paint caches regressed the real 16-child scene by 18-28%; direct local slots
for every nested path cache produced only about a 1.3% draw-only improvement
on one fixture with inconclusive total time. Neither is part of the final tree.
