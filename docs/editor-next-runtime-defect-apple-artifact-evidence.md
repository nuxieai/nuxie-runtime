# Apple exact-runtime artifact qualification

This is the immutable evidence index for `LOC-015`, `LOC-016`, and
`LOC-017`. It records a local qualification artifact and does not claim a
public release or `nuxie-ios` `main` consumption.

## Runtime artifact

- Runtime version: `0.2.0`
- Source revision: `b1f58004332a73564ffdd9f8585838209604c4d1`
- Runtime identity:
  `0.2.0@b1f58004332a73564ffdd9f8585838209604c4d1`
- XCFramework zip SHA-256:
  `478c0a5b95bf7f2e96ff8407d8467d137849f7d6d6c8601d3b073d18a1c61442`
- `artifact.json` SHA-256:
  `890a3e0e738ee73705c60519ee8fb1a8be7521b1bc5897da9b0f2dea285bbba7`
- Contract fingerprint:
  `5c6277af4806f27262e7ce6cfe871e38b04daf9a8770286e41a8e22a3a13aee3`

The identity, generated header, exported symbols, device and universal
simulator slices, archive purity, Swift import, C import, checksum, and
XCFramework validators passed. Clients bind the exact runtime version and
source revision. They do not query or negotiate a separately client-versioned
ABI.

## Editor producer

Editor correction PR #5080 merged to
`levi/editor-next-cutover-assembly` at
`233552c13929b09666a62ddff541eb8620d1882b`.
It corrects the exact production artifact used by the Apple qualification,
including target-0 WGSL plus target-16 binding identity, opaque external-image
pixels, the complete named animation operation/easing corpus, and projection
behavior that follows pinned C++ rather than inventing derived item writes.

Pinned C++ `d788e8ec6e8b598526607d6a1e8818e8b637b60c` and Rust both preserve the
ordinary-assets icon bytes and opacity `1`. The ordinary-assets `flow.riv`
SHA-256 is
`5a6d88f9ad7ed3869e9d04da2624be3bcfe0994cd318e0a49967ed2e3a09609e`;
the icon PNG SHA-256 is
`2086c499d5844a751792f3608d0f0dfb74e4a8e35b2c49a3513c15382352378c`.
The C++ and Rust diagnostic streams have distinct full-stream SHA-256 values
(`906bebf6568e49ec3e0ec1d5108349054243d7e39441e2efefa15c657d008408`
and
`f70a97c8acf9dc02cc50e93bd94840acd868a44ad75c8813953cf3640f745b02`);
only the exact asset payload and opacity claim is made.

## iOS consumer qualification

The qualification-only `nuxie-ios` branch commit is
`f9528fe4295de0a55d121fd7e5290374b22f03c5`. It is intentionally pushed
without a PR or `main` merge because the public SwiftPM fallback still points
at the older runtime. The LOC qualification stages the hash-addressed
XCFramework directly and never downloads over it.

Artifact-consumption run
`5ef5769f-d521-4471-8b91-b9f83acdd065` completed all six consumers:

| consumer | sentinel SHA-256 |
| --- | --- |
| standalone runtime | `2541e9c216dfae4473e26c077103aa1ab6a1dd3e69de403d63e362ad5f2f4ab6` |
| iOS native runtime | `9783741c6827e6cd28bf97e22eedc8b20feaf65431c722045905624a1120d428` |
| iOS SDK pipeline | `15a7f37de86ed9e5fa13bea1ad5eaf79e6defc3c98c1f2737610500c570adafa` |
| signed GPU canvas pixels | `fd051af84f5c516988f53e2167ca8181b86c74f9d9ecbd7df3329a068d32b38c` |
| native corpus pixels | `233ef9a3a892e0daef1b7b8ef43d8c08daa81f2368d39ec1963c265927d5151b` |
| native runtime archive | `f00b20a0d6933e127f363e7bb2fc96afb47be297ed5c783ccb3babfb8ddd8e47` |

The run manifest SHA-256 is
`2b4619291c5250b0e33e182e5efb31423e02a411f4a8eb8b592c9b07cc730512`.
The full Metal matrix passed for nine screens, signed GPU canvas, all 28
named operation/easing cases at start/quarter/end, and the declared behavior
operations. The iOS unit floor was 863 passed, 0 failed, 2 skipped (865 total).
The exact
producer, standalone runtime, runtime adapter, archive-purity, and
XCFramework-validator gates also passed.

## Disposition

- `LOC-015`: the terminal first-draw report came from a stale binary/artifact;
  the exact-runtime-identity framework draws the production corpus.
- `LOC-016`: the runtime implementation existed; the missing proof was exact
  artifact/consumer verification. Typed default, state-machine, and named
  linear-animation selection pass against the exact framework.
- `LOC-017`: the historical native capture was invalid. The corrected Editor
  producer and typed player/time/host composition pass the full Metal corpus.

Public release URL/checksum updates and default SwiftPM consumption remain
downstream distribution work. They are not closure gates for these three
artifact qualifications.
