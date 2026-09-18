# Apple 0.10.8 video visibility size review

Candidate `ae10159adaed48bd28579e8b6811e3c98f2f4dd9` built all five Apple
slices with the pinned release toolchain and passed C/Swift Metal smoke tests.
Its Intel iOS simulator archive measured 48,244,928 bytes, exceeding the
48,234,496-byte ceiling by 10,432 bytes (0.022%). Both distribution archives
contain that same slice, so both reported the same limit failure.

The candidate adds pre-decode video visibility, including transformed mesh
triangles and clipping, the native query, and intrinsic-size layout before the
first decoded frame. Its measurements remain inside every other existing limit:

| Measurement | Full Apple | iOS only |
| --- | ---: | ---: |
| Compressed bytes | 70,493,106 | 43,483,706 |
| Expanded bytes | 240,748,751 | 147,246,003 |
| Representative C linked bytes | 28,858,848 | 28,812,984 |
| Representative Swift linked bytes | 28,886,336 | 28,840,256 |

Increase only the Intel iOS simulator slice allowance by 128 KiB to 48,365,568
bytes in both distributions. Keep all other limits unchanged. This allows the
added visibility behavior without changing release compiler flags or excluding
supported geometry or platforms.

These measurements explain the budget change; they do not qualify the final
release commit. Rebuild and verify the exact landed source with its new input
digest and record its measurements in the immutable release size report.
