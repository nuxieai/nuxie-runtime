# Compiler-private wrapped sizing primitive

`src/wrapping.rs` ports the successful ordinary slot measurement/alignment graph into the compiler, generalizing fixture-specific arrays to a variable-length list of slot/visible handles and alignment fractions. It preserves logical packing order, axis projection, forward/backward line maxima, wrap-reversed positional offsets and measurement independence. It emits no host logic and changes no runtime/schema/dependencies.

The module is staged behind a private module declaration. Public CSS compilation does not yet call it; public wrapping syntax remains rejected. This is implementation groundwork, not a new supported backlog row. Coordinate/gate bounds in `wrapped-coordinate-admission.md`, slot admission, optimized paint completion and public-interface qualification remain required.

## Exact output and tests

A harness compiles the actual `src/wrapping.rs` and `src/wire.rs` with the immutable schema, reads the48frozen raw combined fixtures, applies their original native parent configuration, and invokes the new primitive. All48sizing-stage outputs are byte-identical to `wrapped-combined-slots-r1/render/*/scene.sized.riv`. Thus substituting this sizing stage preserves the downstream paint experiment exactly. Its existing384frame combined native/Chrome evidence applies to these three-item outputs; no new raster run or broader item-count qualification is claimed.

The helper preflights exact added record count: zero for no items, otherwise52N−24. It checks arithmetic overflow, ordinary ID capacity, handle kinds and positional fractions before appending. Actual emitted count must agree. Unit tests cover empty, single,2,3,8,16and32item graphs across both axes, both cross reversals and all three positional line fractions. Schema encoding succeeds for each graph. These tests establish construction/cost consistency, not native resizing behavior for previously untested item counts. Invalid fractions and self-target/missing handles diagnose before emission.

Validation: 100 Rust tests and28Node transport tests pass; native and WASM builds and TypeScript checks pass. `check-target-runtime.py` verifies no tracked/index/nonignored additions outside the compiler differ from the fixed baseline. No public routing, reset, source-map or syntax behavior changed. Test logs and exact harness build/source/dependency hashes are bound in `wrapping-module-receipt.json`.

## Remaining integration work

The primitive's caller must independently prove slot/visible structural independence and all coordinate/line-separation/carry bounds. Its existing DistanceConstraint arithmetic retains the known tiny/zero and rounding concerns; validation of plan shape is not a semantic admission certificate. It does not add a scene-wide cost ceiling, paint masks, nesting support, normal/stretch distribution or intrinsic sizing. Those belong to public wrapping integration and cannot be inferred from48byte-identical examples.
