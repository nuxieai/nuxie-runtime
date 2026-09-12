# Public output regression after private fixed-layout normalization

The new frozen public CLI and WASM binaries differ from the preceding wrapped-rounded build, so executable identity cannot transfer its public regression evidence. Both new binaries were actually exercised on all 1,076 existing accepted corpus entries: 282 image requests and 794 accepted transport requests. Every CLI scene and source-map byte sequence matches its recorded prior result. Every raw-WASM scene byte sequence and parsed source-map structure also matches. The two corpus groups may overlap; this is 1,076 entries, not a claim of 1,076 unique requests.

Frozen CLI SHA-256: `0ba75e2469f8611d36f0ce6963f3da22786c088c0e9858d9cef49da86265c073`.
Frozen WASM SHA-256: `956638b04931ad485c719e9ee21d0d864dc9e1329e5783bfbc5787f3a20818c8`.

`wrapped-normalized-public-regression.py` first verifies all prior request/scene/map hashes and 308 frozen source bindings, then copies the new executables and each request into a fresh evidence directory. The CLI runs separately for every entry with a ten-second timeout; all runs exit successfully without stdout or stderr. A generated Node worker instantiates the actual new WASM binary, checks ABI version 2, submits every request through the documented request buffer/compile functions, copies metadata and Rive bytes, then resets and verifies cleared output lengths after each entry. All 1,076 WASM calls succeed with the expected language version. No JS wrapper is substituted for the raw ABI in this campaign.

Actual output hashes and parsed maps were independently rechecked after both campaigns completed. Recipes and older output artifacts were rehashed after execution to detect changed references. The evidence retains each request, CLI logs and output, WASM metadata/output, commands, executable identities and source bindings under `output/wrapped-normalized-public-regression-r1`. The compact receipt links the execution receipt and its constituent manifests.

No scene was rerendered: exact ordinary `.riv` bytes preserve the prior renderer input. This does not claim new native/browser visual qualification, rerun malformed-input behavior, rerun a historical JS-wrapper corpus, or cover new public HTML/CSS semantics. The separate normalized product build's 431 Rust and 56 Node tests remain their own bounded evidence. Public feature counts remain unchanged.
