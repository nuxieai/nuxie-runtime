Q05 finite malformed-source mutation campaign passed against the frozen public
content-owner compiler. No compiler counterexample was found. This review
records a deterministic robustness sample, not completion of fuzzing or broad
HTML/CSS conformance.

The initial receipt is preserved below. A full second run, described at the
end, also passes and adds a deliberate worker-error/restart control. Use
`output/public-malformed-mutations-r1/lifecycle-r2/receipt.json` for the latest
harness qualification; all 1,000 requests and compiler responses agree with
the initial run.

The run completed all **1,000 cases** on 2026-09-12, from 08:45:44.452 to
08:45:54.856 UTC. There were **174 successful compilations and 826 structured
rejections**, with zero native abnormal exits/signals, WASM traps, timeouts,
determinism differences, transport differences, output-ownership failures, or
failed recovery checks.

| Cases | Accepted | Rejected |
| --- | ---: | ---: |
| 20 unchanged source-fixture seeds | 20 | 0 |
| 960 mutations | 148 | 812 |
| 20 fixed boundary controls | 6 | 14 |
| Total | 174 | 826 |

The seeds are copied from ten files in the same public source freeze: baseline
color/cascade, selectors, custom properties, variable cycles, numeric token
escapes, content-box layout, data attributes, invalid-value recovery,
variable recovery, and content-owner scenes. `seeds.json` retains the original
fixture file/index/name/hash and complete request. All seed controls were
required to succeed before considering mutation outcomes.

The generator uses xorshift32 seed `0x514f0531`. Each seed receives 48
mutations: six insertions, deletions, duplications, and truncations in each of
HTML and CSS. This yields 120 cases in each field/operator combination.
Locations use deterministic lexeme spans, including escape sequences,
identifiers, numbers, punctuation, and whitespace. Insertion payloads exercise
quotes, comment delimiters, tags, attributes, selector syntax, var/fallback
syntax, cycles, dimension overflow/underflow, CSS escapes, NUL, CSS and non-CSS
whitespace, accented identifiers, and emoji. Unicode scalar boundaries are
preserved; no lone surrogate is generated. Each recorded edit is checked by
reconstructing the mutated string from its original prefix, inserted text,
and suffix. Full requests and removed text remain in `cases.json` and each
case directory.

There are **929 distinct complete requests** among the 1,000 cases. The 71
repeated requests are retained with their seed/operator identities; they are
not extra unique-input coverage. The largest compact UTF-8 request is 1,489
bytes. This campaign targets malformed source structure, not large-input
memory or allocation limits.

The fixed controls preserve successful ordinary boxes, cyclic-variable
fallback, invalid-variable recovery, escaped units, Unicode custom-property
names, and CSS whitespace. Fourteen rejection controls retain the documented
empty-document requirement, viewport limits, source coefficient limit,
nonfinite dimension, grid, absolute positioning, text, image elements,
transforms, nesting depth 129, non-CSS whitespace in a dimension, and an
escaped exponent-like unit. In particular, empty HTML is expected to reject
because the public contract requires at least one box; the oracle was taken
from the existing contract, not changed to match a campaign result.

Every request is well typed with finite numeric viewport fields and valid
Unicode strings. Native JSON is unversioned as required by the CLI; the WASM
worker uses the frozen public `createCompiler` wrapper and its versioned
document interface. Successful Rive bytes and source-map data must match
exactly across the transports. Rejections must have identical diagnostic
arrays. Malformed JSON, ill-typed requests, and invalid UTF-16 encoding are
excluded, so differing request-envelope parser offsets are not confused with
compiler differences.

The campaign made **3,653 native process calls** and **3,003 public WASM
compile calls** in one persistent worker. Each native process and each WASM
case batch has a 10-second timeout; worker termination and failure artifacts
handle hangs or traps. Each case compiles twice per transport for determinism.
The worker then recompiles a successful control and verifies that retained
Rive arrays and source maps from earlier calls remain unchanged. Initialization
also mutates a caller-owned result array and verifies that later compiler
output is unaffected. Every native rejection is repeated against preexisting
caller-owned Rive/map files, which must remain byte-identical; a subsequent
successful control must reproduce its original output. Rejected fresh output
paths must remain empty.

The longest observed native call was 8.271 ms. The longest observed WASM case
batch, including two case compiles and successful recovery, was 17.178 ms.
These timings establish only that this finite run stayed within its time
bounds, not a performance guarantee.

All 123 frozen source/input snapshots and both frozen binaries were verified
before and after execution. The CLI, WASM, public JS wrapper, campaign, worker,
source-binding manifest, and selected seed fixtures are copied under
`output/public-malformed-mutations-r1/frozen/`. Principal bindings:

| Artifact | SHA-256 |
| --- | --- |
| Public CLI | `746caad8f78f251acbf188f75d10bdc9340c6cd7d48a994a844e6b0898b30ad5` |
| Public WASM | `c475d543a5f9a4ffcf5f629ef05db5b970b102e2b7682be44419f4b200ed977e` |
| Public JS wrapper | `42d6342c2aa7f84457eac76ae69afdc48923595586cb20569a4e15cbb730361d` |
| Campaign script | `66e1c7c6620a2f64b27eb13009a47de6b170eeb7cabc19492a742a3fb46abb26` |
| Worker script | `3d7dfb949b21815ec1675b280ca010ebe879d943a4869f5b68eb6f6ecbff8c1e` |
| Full case manifest | `a117ca456bfeb421731978474f6161515c96ca9bcdb5d0b9d5f57b2cf925e7e0` |
| Main receipt | `be5a77203d8904664981e94cf43dced6214d12133d1f6a6f5d4cde83c4bf0fc5` |

The receipt retains every case result, native command/status/stdout/stderr,
WASM response, duration, and hashes of **10,652 case artifacts**. An independent
post-run check verified all those hashes and all 1,000 saved requests, compared
all 174 successful native map files byte-for-byte across repeats, checked all
826 retained/recovered native file pairs, and verified the recorded worker
ownership/recovery results. See `independent-verification.json`, SHA-256
`a650a431f0bc51e36ce7dac508528027cece78f7b36e71d238a4cc22d6c914dd`.

Commands, run from `tools/html-to-riv`:

```sh
node --check validation/public-malformed-mutations.mjs
node --check validation/public-malformed-worker.mjs
node validation/public-malformed-mutations.mjs
node validation/public-malformed-mutations.mjs --only mutation-0000 --output output/public-malformed-mutations-r1/replay-mutation-0000
```

The isolated replay passed and reproduced the original request and diagnostic
response exactly. Its receipt is retained beneath `replay-mutation-0000/`,
SHA-256 `f2bbff681e58cfe60900d36f96028348bbf5cdd658e2ce90e2fbebc8048dd4be`.
The harness refuses to overwrite an existing output directory. Replays can
select another case ID and a new directory under the same campaign output.

This does not establish coverage-guided fuzzing, arbitrary malformed-JSON
transport behavior, resource-bound completeness, native scene execution, or
Chrome geometry/pixel parity. No native scenes were rendered. Existing visual
limitations are unchanged. No production, test, runtime, dependency, or shared
goal-document edits were made for this campaign.

The supervisor lifecycle check was added after the initial run. Its worker
event handlers already captured the individual worker instance and ignored
events from a retired worker. The new control explicitly throws a JavaScript
exception in the worker without calling the compiler, observes the expected
error, retires that worker, and immediately starts another. The old worker's
exit occurred **while the new worker had a pending startup request**, exactly
the potentially hazardous ordering. It was correctly ignored, and the new
worker reproduced the successful control. This intentional supervisor
exception is separate from the zero compiler traps/failures.

Both revised scripts were syntax checked, then the entire campaign ran again:

```sh
node validation/public-malformed-mutations.mjs --output output/public-malformed-mutations-r1/lifecycle-r2
```

The second run completed at 08:49:17.757 UTC, 11.337 seconds after starting.
It again passed all 1,000 cases: 174 accepted, 826 rejected, zero unexpected
failures. It made 3,653 native calls and 3,006 public WASM compile calls across
two worker initializations; all case triplets ran in the persistent replacement
worker. The deliberate exception performs no WASM compile. The longest native
call was 8.656 ms and longest WASM case triplet was 17.803 ms. The frozen
compiler, wrapper, 123 source snapshots, and two binaries remained unchanged.

All 10,652 second-run artifact hashes were independently verified. All requests
and responses also matched the initial run, including native map-file bytes
and rejection stderr. Latest bindings:

| Artifact under `lifecycle-r2/` | SHA-256 |
| --- | --- |
| `frozen/public-malformed-mutations.mjs` | `72495d6de8c25f664d248879f0f8d545b50b3c095bad6ebac7ee4b302591195c` |
| `frozen/public-malformed-worker.mjs` | `3f72ca8393701f63459437735510e52c60c74b0719ff9cc0c2fd158cc2d4cb32` |
| `receipt.json` | `4491d89afee7dd10d02d1f2f9dda5f97453ec418797ff5c60cf1ea1cb51d7b34` |
| `independent-verification.json` | `f2d70b2c776897a4fcac7954f31985bb87a72b09ea22d3ffe9e1aa9e799452da` |

`control/worker-lifecycle.json` retains the expected error, old/new worker
identities, old exit code, and pending-state ordering. No compiler behavior or
campaign source case changed between the two runs. The initial receipt and
its frozen scripts remain intact.
