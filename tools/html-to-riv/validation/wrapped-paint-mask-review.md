# Private paint and mask binding checkpoint

The owned source-derived candidate now retains a closed paint-record binding and a geometric mask-domain proof alongside its sizing, coordinate/carry, normalizer and anchored-position analyses. No public compiler route, renderer/runtime modification or mask expansion was introduced.

The paint reader consumes the actual complete suffix: source-cache and source-local differences, all scalar defaults/operands, normalization/clamping, exact leader inversion, rectangle geometry, foreground colors/ownership, required clips, and the complete draw-order chain. Extra fields, malformed roots/handles, wrong operands, missing/extra instructions and changed paint metadata reject. Three tests exercise96emitter configurations plus targeted mutations. This supplements, rather than replaces, the arithmetic/scheduling audit.

The mask proof establishes active intersection and inactive exclusion over the declared viewport domain, checks both initial serialized artboard dimensions, and uses the base inspector's clipped top-left zero-radius artboard. Private widths/heights above32768 remain an unresolved mask profile; the public16384limit stays unchanged. Three tests cover endpoints, truncation at the next float and initial import outside the declared domain. No arbitrary AA margin is assumed.

## Validation

Frozen build `output/wrapped-paint-mask-build-r1/frozen` passes403Rust/56Node tests, strict TypeScript, native/WASM builds and the immutable source guard, with299frozen source/artifact entries. All282image and794historical outputs reproduce exactly under the new binary. No new Chrome/native capture, pixel comparison or direct visual inspection was performed.

The new offline stream checker evaluates384historical snapped frames. It checks4,992submitted clip paths (384artboard,2,304active,2,304inactive), balanced renderer state, exact identity clipping matrices and AABB geometry. It infers2,304draws under empty clip intersections. Four controls reject a changed clipping matrix, non-AABB verb, incomplete stack and unknown command. Intersections follow the audited pinned renderer route; they are not instrumented GPU state. Clips submitted while already empty are reported as such, since the real renderer can return before another AABB evaluation.

The verifier recomputes all observations from actual streams and binds each stream/frames manifest back to the preserved historical checkpoint hash. These are older experimental-epsilon scenes, not the current derived/anchored candidate. Their evidence supports the intended mask mechanism but cannot qualify new output. `wrapped-paint-mask-evidence.py` verifies8,306artifact bindings without rerendering unchanged history.

## Next deliverable

Freeze and exercise the complete current source-derived/anchored/paint-bound candidate through the ordinary native importer and pinned Chrome comparison. Use the new command-stream checker on its actual original/clone resize output, plus edge/overflow pixel cases and visual inspection. Preserve first-import/settling failures, tiny-line diagnostics and all historical visual failures. Only then consider a public compiler admission path; broader wrapping topology, intrinsic sizing and all other99backlog items remain in scope. Counts stay13qualified/23partial/4investigating/59pending. The replacement remains local.
