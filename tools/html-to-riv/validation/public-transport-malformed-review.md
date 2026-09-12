Q05/Q06 transport validation found and repaired two compiler-boundary defects:
positional arrays were accepted as named request objects, and an unreadable
JavaScript request could throw while converting its original error to text.
The repaired public build passes all 253 finite transport cases and preserves
all 794 prior valid Rive files and source maps exactly. Runtime, renderer,
dependencies, and build configuration were not changed.

The documented contract is a named-field design object. README.md states that
CLI input accepts exactly `html`, `css`, `width`, and `height`, and rejects
unknown fields; SUPPORT.md records the same fields, while the JS interface
adds `languageVersion`. `js/index.d.mts` declares these named fields, and the
public JS validator explicitly rejects arrays. Nevertheless, serde's derived
struct deserializer admitted positional sequences. The frozen prior CLI
accepted `[html,css,width,height]` and produced exactly the object control's
file/map. The raw ABI also admitted that input and `[languageVersion,input]`
envelopes, including arrays at both levels. The failing bytes, commands,
outputs, and responses are preserved in
`output/public-transport-malformed-preflight-r1/` and the complete red campaign
at `output/public-transport-malformed-r1/`.

The JavaScript counterexample uses an `html` getter that throws
`Object.create(null)`. The wrapper catches the original access failure, but
its `String(error)` then throws a TypeError. The public return contract calls
for a structured failure during input normalization. The campaign preserves
that exception and demonstrates that earlier output and subsequent successful
compilation still worked; this was an error-reporting defect, not a WASM trap.

The fixes are deliberately small:

- `src/request.rs` requires a map-shaped container, then delegates field
  deserialization directly to serde's typed `MapAccessDeserializer`.
  `CompileInput` and the private ABI `Request` use it from `src/lib.rs` and
  `src/wasm.rs`. No intermediate JSON Value or float reserialization is used.
  Field order, typed binary32 parsing, and duplicate/missing/unknown-field
  checks remain with serde. Original struct names remain in diagnostics.
- `js/index.mjs` guards only error-string conversion. Ordinary printable
  errors retain their messages. Unprintable thrown values return the stable
  `invalid-request` message `Cannot read or serialize design document`.
- `tests/transport.rs` covers CLI rejection without replacing existing output
  files. Two Node regressions cover all three ABI array forms, deterministic
  rejection/recovery, and printable/unprintable JavaScript errors with retained
  output ownership.

The new regressions ran red before production edits: one Rust test failed on
array acceptance, and both Node tests failed for the intended ABI acceptance
and secondary TypeError. Logs and exact commands are in
`output/public-transport-malformed-red-tests-r1/`. Their targeted green run,
including a normal WASM build, is in
`output/public-transport-malformed-green-tests-r1/`.

The finite campaign comprises these 253 cases:

| Family | Cases |
| --- | ---: |
| Successful object controls | 3 |
| Truncated JSON prefixes and other JSON syntax corruption | 131 |
| Missing/wrong/unknown input fields and root types | 45 |
| Duplicate input keys | 4 |
| Positional input array | 1 |
| Numeric transport boundaries | 8 |
| Invalid Unicode surrogate escapes | 3 |
| Invalid UTF-8 byte sequences | 6 |
| ABI envelope shape/version/field controls | 17 |
| JavaScript primitive, object, getter, and proxy values | 28 |
| Unprintable thrown JavaScript value | 1 |
| ABI allocation/reset/empty-request sequences | 6 |

The CLI takes unversioned request JSON, the raw ABI takes a versioned envelope,
and the public JS interface receives an object. These are intentionally
distinct boundaries. Exact diagnostics are required for repeats within each
boundary. Equivalent malformed requests compare diagnostic classes across
boundaries, not line/column offsets inside different envelopes. Raw malformed
bytes that cannot be converted to a valid JS value are not passed through a
replacement-character decoder and misrepresented as the same JS input.

The JS prevalidator checks language version before ABI field parsing, so a
missing or wrongly typed version can yield `unsupported-language-version`
where direct ABI parsing yields `invalid-request`. Duplicate JSON input keys
are rejected by serde; the explicit JS `JSON.parse` recipe has already
collapsed identical duplicates into an ordinary object, so it succeeds.
These differences are declared in the immutable case manifest. Frozen and
null-prototype JS objects, and inherited language-version properties with
ordinary own input fields, follow the existing wrapper normalization rules.
They are not classified as new compiler defects.

The green run made 799 bounded native calls, 685 raw ABI compile calls, and
306 public JS compile calls. It checked 201 CLI cases, 224 raw ABI cases, and
101 JS cases: **526 boundary/case observations**. Each native process and worker
request has a 10-second timeout. Native rejections must not create new output
or replace previously owned output files, and a subsequent valid request must
reproduce the control. Raw ABI rejections expose no stale Rive bytes;
allocation/reset controls clear metadata and result lengths. ABI callers copy
their output, as the private ABI requires. Public JS arrays/maps retained from
earlier calls remain unchanged across repeated failures and successful
recovery. Printable and unprintable host errors are handled inside the bounded
worker. No arbitrary pointer/free calls or large admitted allocation are used.

The red run completed all cases with four failing controls and no unexpected
termination. The green run completed all cases with zero contract mismatches,
panics, signals, WASM traps, worker errors, timeouts, or recovery/ownership
failures. Five cases have red-to-green response changes across seven boundary
observations: the four repaired controls, plus the already rejected empty
input array `[]`. That empty array now reports a sequence-type error immediately
instead of an invalid sequence length; its `invalid-request` code is unchanged.
All other **519 boundary responses are exact**, including missing, duplicate,
and unknown-field diagnostics. The same full case manifest is used in both
runs; no expected result was changed to make the repaired run pass.

Full qualification used the existing `public-value-build.py` workflow with
locked dependencies and the default native/WASM configuration. It passes
**271 Rust tests, 48 Node tests, strict TypeScript, both builds, and the target
runtime guard**. Its 132 source/input snapshots and two binaries remain
unchanged throughout the freeze. The separate output regression binds every
request to the **actual** latest content-owner checkpoint files/maps, including
the 75 files changed at that checkpoint. All 794 compare byte-for-byte; stale
pre-owner outputs are not reused. The earlier 1,000 malformed-source campaign
was not rerun: this change is covered by the new transport campaign, full
public transport tests, and exact output regression.

Reproduction commands, from `tools/html-to-riv` (output directories must be
fresh):

```sh
node validation/public-transport-malformed-campaign.mjs --output output/public-transport-malformed-r1
python3 validation/public-value-build.py output/public-transport-malformed-build-r1
node validation/public-transport-malformed-campaign.mjs --frozen output/public-transport-malformed-build-r1/frozen --output output/public-transport-malformed-r2
python3 validation/public-transport-malformed-regression.py output/public-transport-malformed-build-r1/frozen output/public-transport-malformed-regression-r1
```

The campaign supports `--only CASE_ID` plus a fresh `--output` for isolated
replay. The red campaign explicitly uses the prior frozen public build by
default; the green command supplies the repaired freeze. All raw inputs are
retained as binary files and hexadecimal manifest values, and JS recipes are
retained with their frozen generator/worker code.

| Evidence | SHA-256 |
| --- | --- |
| Prior public CLI | `746caad8f78f251acbf188f75d10bdc9340c6cd7d48a994a844e6b0898b30ad5` |
| Prior public WASM | `c475d543a5f9a4ffcf5f629ef05db5b970b102e2b7682be44419f4b200ed977e` |
| Repaired public CLI | `ca8f561f470dffa80a287aa5cb4d0d6866e7e77475d1cce8631136386fa6061a` |
| Repaired public WASM | `1383b476e655423130f3450a4f3803593ca4f80bef0fe97c9dd2ac12637681f9` |
| Repaired public JS | `5ef3739cdfc787f7713726af2885c440a167c56b4268877760097fe07f45893d` |
| Full build source bindings | `1a45b65ec2c79e98f86cdbe3147d4b902a3da935cf1b5ebf20e82031c384efdd` |
| Full build summary | `169cc19768d909e7969540587fd73a5de6d67aa6ee0ed7983cc885023567837c` |
| Case manifest, red and green | `7af57d3d6e329c91842fe095b912cbc01302fd44e90b63b34294963edda767a7` |
| Red transport receipt | `9d7c4fd81f9b1cc7dd4bf220cc3973d2a5065d8753cce35ae0e5db55137182ed` |
| Green transport receipt | `986db9a24125861eb8b2cfcd0d3eb43e87fe6b1fbe45109442c37c7d50e2e4c4` |
| 794-case output regression receipt | `5ff348c42fd5ccaffead0972d47327c2f98006fe67e6b33201dc87357a165554` |
| Independent artifact/delta verification | `f1ca8815f67845d26eb1f57094f2e792163ae11108c4332387a3936539dce218` |

The independent check verified all 2,527 green case-artifact hashes, all 526
recorded boundary checks, the seven response deltas, and all 794 copied
request/old-output/new-output triples. It is retained as
`output/public-transport-malformed-r2/independent-verification.json`.

This is finite malformed-request and transport validation, not coverage-guided
fuzzing or a proof for arbitrary JavaScript callbacks, allocation exhaustion,
all malformed encodings, or filesystem I/O failures. Large successful
allocations and direct pointer misuse remain outside this campaign. No native
scene rerender or browser comparison was performed: exact ordinary-file/map
preservation transfers existing visual evidence only, including its existing
limitations. No runtime, renderer, dependency, build-configuration, or shared
status-document changes were made for this work.
