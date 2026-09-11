# Compiler-derived main-axis spacing

The public compiler admits `justify-content:normal`, `flex-start`, `space-around`, `space-evenly`, `inherit`, `initial` and `unset` in its single-line flex profile. Normal/initial/unset use ordinary flex-start behavior. Other positional/distributed keywords require separate admission and currently diagnose.

For a nonempty distributed container, N authored children produce N+1 zero-cross-size LayoutComponents with main-axis Fill. Around uses fractional weight0.5 at the edges and1 internally; evenly uses1 throughout. Weight is serialized on the participant, and scale mode on its style. The spacers consume positive free space; physical-start main alignment handles safe overflow, including reverse flows. Empty containers add no spacer. Existing no-distribution output is unchanged.

Spacers are emitted around the order-modified children without adding HTML nodes or source identities. They do not participate in selector matching, variable environments or baseline child summaries. Baseline measurement helpers receive no extra spacing shares. At most2N spacers across a document add at most4N records, bounded by32768 at the8192-element input limit.

Nonempty distributed-column baseline metrics remain rejected because packed-child first/last summaries do not account for spacing offsets. Fixed/provable intrinsic used heights remain useful independently of baseline topology. This guard propagates through ancestors rather than silently constructing an incorrect baseline. General wrap/line alignment and authored grow/shrink/basis remain outside current admission.

## Validation

The112-scene public corpus passes all896 geometry/pixel frames and1792 clear controls on pinned Chrome153.0.8010.12 and immutable rust-metal RasterOrdering. It covers both distributions in all four directions, positive/negative free space, empty/single/zero items, parent intrinsic/responsive/min/max sizing, ordering/selectors, variables, nested alignment wrappers and inheritance, percentage/bounded and intrinsic child sizing, and first/last baseline groups. Taller viewport variants expose the full fixed-column extent that earlier experiments clipped. Same original and cloned scenes resize without recompilation.

All98 Rust tests and28 Node tests pass, including owned output, public CLI/WASM parity and rejected-context diagnostics. Strict TypeScript and the immutable source guard pass. All370 previous unique corpus requests retain identical Rive bytes and source maps. The frozen compiler reproduces all112 newly rendered files and maps exactly. Unit tests cover the8192-element input boundary and spacer record counts. Read-only review added percentage and intrinsic child cases before qualification; no concrete source defect was found.

All172 distinct viewport pairs were directly visually reviewed, with no visible divergence found. The724 exact decoded RGBA crop/white-extension transfers covering the remaining original/clone frames. See public-spacing-receipt.json for final review coverage and evidence bindings. This qualifies the admitted single-line main-axis profile, not wrap, distributed align-content or unsupported nested baseline metrics.
 Existing fractional pixel failures elsewhere remain preserved and tolerances unchanged.
