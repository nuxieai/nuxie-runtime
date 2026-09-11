# Reverted integration changes

The other source inventories cover 106 runtime/renderer/vendor paths. This file accounts for the remaining 12 paths outside the 660 compiler-module paths in the 778-path reverted diff. All are absent or restored to their pre-PR contents by #629.

| Path | Purpose of reverted change | Migration decision |
| --- | --- | --- |
| `.github/workflows/html-to-riv.yml` | Compiler CI, WASM, browser geometry, modified native-glyph renderer gate | Rebuild validation against immutable baseline; no restored claim of native-glyph parity |
| `.gitignore` | Compiler/generated artifact exclusions | Keep future compiler artifact exclusions module-local |
| `Cargo.toml` | Enrolled compiler, excluded Taffy/ICU forks | Keep runtime workspace resolution unchanged; isolate compiler-owned manifest |
| `Cargo.lock` | Compiler dependencies and runtime Unicode dependency changes | Do not restore broad lockfile; compiler lockfile must not alter runtime graph |
| `docs/README.md` | Links to compiler/research documents | Reintroduce documentation links only with the revised contract |
| `docs/html-css-to-riv-design.md` | Original design and capability assumptions | Historical; TARGET.md supersedes execution contract |
| `docs/pure-runtime-boundary.md` | Compiler exclusion and source/dependency guard documentation | Old dependency direction check did not prohibit runtime behavior changes; immutable identity gate is stricter |
| `docs/rml-research.md` | RML/upstream research | Reusable historical research, not baseline file-encoding proof |
| `tools/pure-runtime-boundary/check.py` | Protect excluded Taffy and reject runtime imports of compiler | Reverted with PR; new module-local gate checks changes in either direction and root resolution files |
| `tools/pure-runtime-boundary/test_check.py` | Tests of that dependency-direction guard | Historical tooling evidence; new gate has independent negative controls |
| `tools/renderer-replay/src/bin/gradient-benchmark.rs` | Retained native renderer timing/correctness harness for custom exact gradients | Not usable as unchanged-runtime qualification; future experiments live in compiler-owned tooling |
| `tools/renderer-replay/src/main.rs` | Ordinary Atomics selection, opacity group precomposition and checked canvas/replay behavior | Restore original runner; no CSS-specific stream/host orchestration can be required by compiled files |

The generated runtime registry change is inventoried in immutable-runtime-audit.md. No schema crate file was changed by #628, but absence of a schema diff is not self-contained file compatibility: custom requirements and rendering stream operations still supplied substantial behavior after import.

Compiler-module files include production parsing/emission, package interfaces, tests, examples, fixtures, benchmark scripts and historical reports. Reuse is selective: pure authoring algorithms and source oracles can survive; policy emission/installation and modified-renderer test hosts must not. Source references and complete before/after Git blob identities for every path are retained in reverted-mutations.json.
