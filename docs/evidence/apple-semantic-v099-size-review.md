# Apple semantic runtime v0.9.9 size review

The five-target build of `a96f7345f715857a649f668287c1c4eb56b0fefd` completed compilation and packaged C/Swift smoke checks, then failed the frozen size gate. It has not been published. The feature-bearing source since v0.9.8 includes portable semantic snapshots/actions, ancestor eligibility, rendered clipping and exact native text-run association. Android Vulkan surface work in that range is gated out of the Apple feature closure.

Published comparison: [v0.9.8](https://github.com/nuxieai/nuxie-runtime/releases/tag/apple-runtime-v0.9.8), source `6c7ac16617835b5f581784ff08a9e779bb52faf3`, using its downloaded `SIZE_REPORT.json` and `artifact-set.json`. Current measurements come from `target/nux-capi-apple/SIZE_REPORT.json` after `make nux-capi-xcframeworks`.

| Measurement (bytes) | Published v0.9.8 | v0.9.9 candidate | Existing ceiling | Proposed ceiling |
| --- | ---: | ---: | ---: | ---: |
| iOS arm64 device slice, both artifacts | 47,751,032 | 48,301,144 | 48,234,496 | 49,283,072 |
| iOS arm64 simulator slice, both artifacts | 47,701,320 | 48,251,352 | 48,234,496 | 49,283,072 |
| iOS-only expanded archive | 144,023,611 | 145,755,411 | 144,703,488 | 146,800,640 |

Only these five budget fields change (two shared slice measurements each occur in both artifacts). Each ceiling rounds the measured value up to the next 1 MiB boundary under the existing policy. The independent archive and linked-binary constraints remain intact. This is a proposed allowance for added runtime functionality, not a performance or memory-use qualification.

The full compressed archive is 69,641,040 bytes against its unchanged 78,643,200-byte ceiling. The iOS-only compressed archive is 42,972,477 bytes against 47,185,920. Full expanded size is 238,255,151 against 241,172,480. Representative linked C/Swift sizes remain below the unchanged 30,408,704-byte ceilings: macOS arm64 28,492,496 / 28,519,856, and iOS arm64 28,449,552 / 28,476,696. Every current measurement remains below the immutable v0.4.0 baseline retained by the release contract.

A fresh build and the ordinary native readiness gate are required after committing the reviewed ceilings. Immutable publication still requires a landed source commit and exact matching tag/archive provenance, followed by downloaded-asset verification. No existing release asset or baseline measurement is modified.
