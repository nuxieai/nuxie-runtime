# Apple 0.10.0 video size review

The first clean video candidate (`b57e6e9b1195a4cb13f2d36094acc01b369335fd`)
built all five slices and passed packaged C/Swift consumer linkage. The size
gate rejected only iOS-only expanded bytes: 146,887,432 against 146,800,640.

Compared with `SIZE_REPORT.json` downloaded from the immutable
[0.9.14 release](https://github.com/nuxieai/nuxie-runtime/releases/tag/apple-runtime-v0.9.14):

| Artifact | Measurement | 0.9.14 | Video candidate | Delta |
| --- | --- | ---: | ---: | ---: |
| Full Apple | Compressed | 69,709,236 | 70,269,990 | +560,754 |
| Full Apple | Expanded | 238,439,640 | 240,143,311 | +1,703,671 |
| iOS only | Compressed | 43,020,674 | 43,348,484 | +327,810 |
| iOS only | Expanded | 145,867,090 | 146,887,432 | +1,020,342 |

The added runtime includes video scene import, state/clock control, captions,
synchronization and the native ABI surface. It preserves the release toolchain
and feature configuration. Round the iOS-only expanded maximum independently
to the next whole MiB: 147,849,216 (141 MiB), up from 140 MiB. Keep compressed,
individual-slice and representative-linked ceilings unchanged; all were within
their existing limits for this candidate.

This review does not qualify a later commit. The final candidate, including the
image snapshot API, must regenerate the exact artifacts and pass all budgets
before publication. Its source identity and measurements belong in the final
artifact-set and size report.
