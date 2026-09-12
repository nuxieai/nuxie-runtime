# Public image asset boundary and malformed-input campaign

The final frozen compiler passes **86 cases / 252 transport observations**: 86 CLI, 86 raw WASM ABI and 80 public JavaScript observations. This is compiler/resource/transport evidence only. It does not qualify native rendering or Chrome pixel parity.

The runnable driver is [public-image-assets.mjs](public-image-assets.mjs). The actual run and complete per-case inputs, binary fixtures, commands, responses, Rive files and maps are retained under [public-image-assets-r1](../output/public-image-assets-r1/). The [receipt](../output/public-image-assets-r1/receipt.json) records all results; [verification.json](../output/public-image-assets-r1/verification.json) records an independent post-run artifact check. There were no native panics/signals, WASM traps, timeouts, contract mismatches, unexpected diagnostic differences, stale outputs or recovery failures.

## Frozen command and inputs

Run from `tools/html-to-riv`:

```sh
node validation/public-image-assets.mjs \
  --frozen output/public-image-build-r2/frozen \
  --expect-cli 8f49ac9a7272f0770cc3ae67eb29755830219ff5af0716741b2f77735aa218df \
  --expect-wasm 2e636686877bdebf9e8c70352ae28a98f3ecba4cb00f224386267f0363c899bf \
  --output output/public-image-assets-r1
```

For a repeat, use a fresh output suffix; existing evidence is never overwritten. `--prepare` constructs and saves all expected inputs without invoking a compiler; `--only CASE_ID` reproduces one case. Cross-case alias/control equality is checked in the full campaign; a single-case run does not claim that additional check unless its reference is also present.

| Input | SHA256 |
| --- | --- |
| CLI | `8f49ac9a7272f0770cc3ae67eb29755830219ff5af0716741b2f77735aa218df` |
| WASM | `2e636686877bdebf9e8c70352ae28a98f3ecba4cb00f224386267f0363c899bf` |
| Frozen JS wrapper | `5f4c90b8d2a905c66c2152bf2499ebc10b923891bac57feecf0d0f36b4aca5f1` |
| Frozen driver | `fe5f03096fb0baea2f32bc0783990822c80b80e170373bb7a838593c6e4a6e97` |
| Source bindings manifest | `45702ec210a79e543637afae033dfe5986e9b9894d4fa6f8369e7595f5cbacee` |
| Run receipt | `e1fd1d32cbfc72c2484017b015bfdbc29fe44cd84cf2f2cc9fc949ca584330f4` |

The run lasted approximately 27 seconds. It made 322 native CLI calls, 344 raw ABI calls and 326 JS calls, including repeats, control calls and recovery checks. Every frozen input and its retained copy remained unchanged. Post-run verification rehashed **2,373 case artifacts, 62 encoded fixture files and 153 frozen compiler bindings**, compared saved results to the aggregate receipt, and reread complete successful and recovery Rive bytes/maps. Observation hashes index saved outputs; the driver also compares complete byte buffers, not hashes alone.

Preparation-only attempts remain separately preserved at `output/public-image-assets-prepared-r1` and `output/public-image-assets-prepared-r2`; neither is compiler validation. Before the final run, harness review fixed an aliased retained source-map snapshot and added a retained driver copy. The executed final driver includes those repairs. Expected classifications were derived from the source contract before running the frozen compiler; no expected outcome was changed after execution.

## Exact scope

| Family | Cases | Boundary or failure being tested |
| --- | ---: | --- |
| Seven format controls | 7 | Ordinary opaque/alpha PNG, baseline/progressive JPEG, lossy/lossless/alpha WebP all succeed. |
| Empty-map and representation controls | 3 | Omitted/empty maps preserve box-only bytes/maps; numeric arrays equal typed-array image input. |
| Supplied-name count | 3 | 255 and 256 exact-byte aliases succeed and deduplicate; 257 names produce `asset-limit`. |
| Independently valid encoded-size controls | 3 | PNG files of exactly 65,535, 65,536 and 65,537 bytes all succeed independently. |
| Encoded aggregate | 3 | 16,777,215 and 16,777,216 supplied bytes succeed; 16,777,217 produces `asset-limit` at the deterministic final key. |
| Per-axis boundary | 6 | Real complete PNGs at 8191, 8192 and 8193 pixels on each axis; the first two succeed and the last rejects. |
| Decoded RGBA boundary | 3 | Complete PNGs at 16 MiB−4, exactly 16 MiB and 16 MiB+4 declared RGBA bytes; every axis remains below 8192. |
| Dimension header guards | 8 | PNG zero/u32-max dimensions, and JPEG/WebP axis/footprint excess, reject before pixel decoding. Oversized header mutations deliberately do not provide matching pixels. |
| PNG integrity | 12 | All three chunk CRCs, missing IEND, trailing file data, missing/wrong Adler32, trailing compressed data, short/long rows, invalid filter and overflowing chunk length. |
| Unsupported metadata | 8 | Late PNG EXIF/ICC/animation chunks, late JPEG APP1, and late WebP EXIF/ICC/animation chunks remain diagnostic. |
| JPEG entropy/container | 8 | Baseline and progressive empty entropy, a removed last byte in the first scan, missing EOI and trailing file data. |
| WebP integrity | 6 | Truncation, trailing data, duplicate payload, payload-header-only input, nonzero RIFF padding and overflowing chunk length. |
| Keys and unused assets | 3 | Empty key, unused malformed asset, and Unicode scalar ordering for deterministic diagnostic selection. |
| Raw JSON parser | 6 | Duplicate names, escaped-equivalent duplicate names, duplicate rejection before an invalid value, duplicate kind/bytes fields and a floating-point byte token. |
| Typed request parser | 7 | Null/array map, array asset, unknown kind/field, negative byte and byte 256. |

The aggregate controls use legal empty stored DEFLATE blocks and contiguous empty IDAT chunks, with real checksums and exactly one complete zlib stream. This produces exact source byte counts without invalid padding or large emitted scenes. All three component PNG encodings first pass independently. The at-limit request has 256 aliases of the same bytes and its entire emitted Rive/map equals the one-name control; aliases demonstrably count before deduplication.

The decoded controls use `2047 × 2049 × 4 = 16,777,212`, `2048 × 2048 × 4 = 16,777,216`, and `1985 × 2113 × 4 = 16,777,220`. These are the nearest representable RGBA-pixel footprints on either side of the cap, with no competing axis-limit failure.

Each CLI/ABI has 23 successes, 12 `asset-limit`, 27 `invalid-image`, 10 `unsupported-image`, one `invalid-asset` and 13 `invalid-request` outcomes. JS has the same semantic outcomes, with seven typed `invalid-request` outcomes. The six raw lexical/duplicate cases have no equivalent JS object representation and are explicitly omitted from JS observations.

## Transport, ownership and parity rules

Each case repeats through CLI and the raw ABI, then performs a successful recovery. Each representable JS case repeats and recovers using a reused public compiler instance. The original successful JS Rive bytes and a deep source-map snapshot remain intact after subsequent calls; the current result also remains owned after repetition/recovery. The raw ABI checks allocation/reset output clearing, copies both output buffers, verifies no stale Rive on failure and preserves earlier owned copies. CLI rejection creates no new Rive/map, and retrying rejection against pre-existing successful files preserves both files byte for byte.

Well-typed semantic requests require exact CLI/raw ABI/JS diagnostics or successful bytes/maps. Raw parser diagnostics are deterministic within each boundary; CLI and ABI typed reasons must match after removing their final line/column suffix because the ABI adds an envelope. The seven ill-typed JS cases preserve their distinct wrapper-level diagnostic wording/source and require the declared `invalid-request` class, no output and recovery. Those seven differences are intentional recorded transport behavior, not suppressed compiler parity failures.

## Limits of this evidence

- Each CLI call and each worker has a 60-second timeout. Each case uses a fresh worker with a 768 MiB V8 old-generation limit; ABI/JS instances are reused within that case. This is not a long-duration allocation-growth test or a whole-process memory cap.
- The largest serialized ABI request is **44,748,464 bytes**. No 192 MiB request allocation, arbitrary-pointer ABI operation or unbounded fuzzing is performed.
- The raw ABI instance's largest observed linear-memory size is **64,290,816 bytes**, in the below-limit encoded aggregate case. It is an observation of that instance, not a bound on the JS instance, V8 heap, buffers, CLI process, combined process RSS or future inputs. The end-of-case `process.memoryUsage()` samples are not peak-memory measurements.
- `main.rs` reads and deserializes the whole request before semantic asset limits run. The JS wrapper copies/normalizes byte arrays and serializes the request before its envelope-size check. This campaign does **not** prove bounded pre-serialization JS memory, bounded CLI request reading/deserialization, or graceful behavior for arbitrarily larger inputs.
- The decoded limit is per image; this campaign does not establish an aggregate decoded-memory cap, native GPU/resource lifetime bound, or arbitrary asset-name-length bound. It exercises exact-byte deduplication and a small number of unique images, not the maximum simultaneous unique decoded workload.
- Header-overflow cases prove the declared guard and diagnostic path for those exact supplied files. They are not successful oversized image decodes. JPEG scan/refinement breadth is covered separately by [public-image-jpeg-review.md](public-image-jpeg-review.md), and image layout/rendering qualification is separate.

No production, runtime, renderer, dependency, build configuration or shared progress document was changed for this campaign.
