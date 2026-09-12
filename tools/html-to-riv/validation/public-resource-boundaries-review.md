# Public resource boundaries: finite Q06 campaign

All **40 cases / 120 interface observations pass** against the frozen public compiler from `output/public-transport-malformed-build-r1/frozen`. Each interface produces 25 successful results, nine `input-limit`, three `depth-limit`, and three `object-limit` diagnostics. No production, runtime, renderer, dependency, build configuration, or shared progress file changed. No concrete compiler defect was found.

This supplies bounded compiler/transport evidence for the documented authoring limits. It does not close Q06 or establish a general memory, throughput, or renderer guarantee.

## Bound source and commands

- CLI SHA-256: `ca8f561f470dffa80a287aa5cb4d0d6866e7e77475d1cce8631136386fa6061a`.
- WASM SHA-256: `1383b476e655423130f3450a4f3803593ca4f80bef0fe97c9dd2ac12637681f9`.
- Public JS SHA-256: `5ef3739cdfc787f7713726af2885c440a167c56b4268877760097fe07f45893d`.
- Frozen binding manifest SHA-256: `1a45b65ec2c79e98f86cdbe3147d4b902a3da935cf1b5ebf20e82031c384efdd` (132 source snapshots and two binaries).

Run from `tools/html-to-riv`:

```sh
node --check validation/public-resource-boundaries-cases.mjs
node --check validation/public-resource-boundaries-worker.mjs
node --check validation/public-resource-boundaries-campaign.mjs
node validation/public-resource-boundaries-campaign.mjs --output output/public-resource-boundaries-r1
node --check validation/public-resource-boundaries-verify.mjs
node validation/public-resource-boundaries-verify.mjs output/public-resource-boundaries-r1
```

The output directory must not already exist. A repeat campaign needs a fresh `output/public-resource-boundaries-*` path. The scripts use only the explicitly pinned compiler and wrapper; parent work in the current source tree does not enter this run. Exact executed commands, source copies, all requests, and file hashes are retained under the output directory.

## Coverage and observations

| Family | Cases | Construction and expected result |
| --- | ---: | --- |
| Successful control | 1 | A single painted box; also the recovery oracle. |
| Combined HTML/CSS UTF-8 source bytes | 18 | 1,048,575 / 1,048,576 / 1,048,577 bytes in six inert-comment profiles: HTML ASCII, CSS ASCII, split ASCII, HTML two-byte `é`, CSS three-byte `雪`, and split four-byte `😀`. First two compile exactly like the small control; the last rejects. |
| Serialized JSON versus decoded source | 3 | The same three decoded source sizes, with every non-ASCII UTF-16 unit serialized as a JSON Unicode escape. Valid surrogate pairs remain intact. |
| Authored nesting | 6 | 127 / 128 / 129 nested boxes with plain fixed sizing or nonzero content-box padding. First two compile; 129 rejects. |
| Authored element count | 6 | 8,191 / 8,192 / 8,193 elements, both flat plain siblings and two groups of content-box children. First two compile; 8,193 rejects. |
| Combined boundaries | 6 | Source exactly at the byte limit with depth 128/129 or element count 8,192/8,193; source one byte above the limit with each excessive structure. At-limit structures compile, the first excess receives its structural diagnostic, and excessive source receives `input-limit` before the structural guard. |

The decoded UTF-8 distinction is observable. At exactly 1 MiB, the split emoji input contains 262,201 Unicode scalars and 524,326 UTF-16 code units. The escaped-JSON control contains 1,048,576 decoded source bytes while its raw ABI envelope occupies 3,145,693 bytes; it still compiles. Its one-byte-larger source is rejected. The public JS wrapper receives the same decoded strings and chooses its own JSON serialization, while the CLI and raw ABI receive the retained escaped wire forms.

Every successful request preserves the authored ID/path map, with no synthetic content-owner identities leaking into the map. The grouped padded 8,192-element scene contains 8,192 source nodes and 1,199,046 Rive bytes. Every successful request has an independent no-op control: inert source comments, explicit zero padding for plain structures, or an inert comment for padded structures. Both the Rive files and serialized CLI map files are byte-for-byte identical to their repeat and no-op control.

The guards match the documented contract in `SUPPORT.md:45` and frozen `src/compiler.rs`: the combined Rust string length check at line 475 precedes HTML/CSS parsing, `prepare_children` checks depth above 128 at line 601, and authored-element checks occur at lines 616 and 631. This campaign tests ordinary finite requests; viewport, selector, numeric, variable expansion, and serialized ABI maximum limits are separate contracts.

## Determinism, ownership, and bounds

The campaign made **136 native CLI calls, 121 raw ABI calls, and 146 public JS calls**, using one worker with separate raw ABI and public-wrapper WASM instances. Each case was repeated through each interface. Complete successful Rive bytes/source maps or diagnostic arrays match across all three interfaces; raw ABI metadata also repeats exactly.

All 15 rejection cases publish no fresh native `.riv`, `.map.json`, or `.requirements.json` output. Repeating the rejection against copies of the preceding successful files preserves those files exactly, including large element-limit outputs. A subsequent control compiles successfully. The public JS result objects and byte arrays remain unchanged across later calls; raw ABI allocation clears old output lengths, rejected compilation exposes no Rive data, and reset clears response lengths. The raw ABI buffers themselves are compiler-owned; the harness copies them before reset as required.

Each native invocation and each worker transaction has a 10-second timeout; native output capture is capped at 8 MiB and the worker old-generation heap is capped at 256 MiB. The entire run took 43.03 seconds on this host. These are harness bounds and observations, not product performance guarantees or measurements of total process/WASM memory. The largest serialized ABI request was 3,145,694 bytes. No admitted 192 MiB allocation, arbitrary-pointer fuzzing, browser capture, or native rendering was performed.

## Retained evidence and handoff

- [Campaign receipt](../output/public-resource-boundaries-r1/receipt.json), SHA-256 `8d0fa6d223786f3eabbe37de08335fe2053ba6b1bcebae0360fcacb593bdac4a`.
- [Independent artifact verification](../output/public-resource-boundaries-r1/artifact-verification.json), SHA-256 `bebbd6a8b716398d28dacde291b6c02499e76daae6e611856b70164127c572c5`.
- [Full request/case manifest](../output/public-resource-boundaries-r1/cases.json), SHA-256 `0fbab43147fc3cf3cd63a1614e64158df6065338a7123f5b058fb5d1c8f0b288`.
- Per-case `cli-input.json`, `abi-input.json`, `case.json`, commands, outputs, worker responses, and `result.json` are retained in `output/public-resource-boundaries-r1/cases/<case-id>/`. All expected rejection controls are retained; there were no unexpected failing cases.
- The verifier rehashed **530 recorded case artifacts and all 134 compiler bindings**, regenerated every request from the frozen generator, checked source/serialized byte counts and Unicode counts, compared complete transport results, checked authored identity paths, and independently compared repeat/no-op/recovery files.

This bounded subtask is complete. No follow-up implementation is indicated by these results. Remaining overall Q06 work should retain its existing scope; this campaign does not cover every resource interaction, expansion-heavy CSS, all structural shapes, broad adversarial distributions, or cross-host/browser execution. No further campaign was started after the request to prepare the goal restart handoff.
