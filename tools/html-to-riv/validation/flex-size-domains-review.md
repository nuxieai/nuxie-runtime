# Conditional parent-first rigid size domains

A private propagation pass now consumes completed compiler descriptors in parent-first object order, regardless of child-first capture order. The caller explicitly supplies viewport intervals and conversion error; there is no implicit zero-error viewport. Direct nonflexing boxes with definite point/percentage dimensions, zero minima and absent maxima receive both-axis domains. Percentage descendants inherit the parent domain through the checked native-operation envelope.

The pass checks direct ownership, native scales/fractions, typed scalar provenance, raw dimension units/bits and both-axis native bounds. Nonzero original factors cannot become exact CSS zero merely through native rounding. Raw fixed automatic minima are not relabeled point zero: in this no-inset definite profile, pinned flexbox min-content is capped by the preferred size, so it cannot enlarge the nonflexing target. Cross-axis bounds are now retained in descriptors instead of inferred from computed values alone.

Flexible/intrinsic targets, local overrides, helpers/wrappers, clamps and local constraint/origin ownership remain unresolved. Point-size resets can regain a local definite dimension after an unknown parent; this does not restore ancestor/world proof. No positions, transformed corners or whole-scene admission are certified. Numerical/public flex admission remains unchanged.

Tests exercise actual child-first compilation, nested percentage propagation, an explicit parent-error contribution, flexible target rejection, mutated native cross bounds and constraint context. The64-scene capture retains488 bounded axis domains (including explicit root domains); other axes remain unresolved. Root envelopes for that capture are[0,16384] with0.001 absolute error. These are conditional arithmetic results, not new sampled native or Chrome validation.

Validation passes207 Rust tests,36 Node tests and WASM build;482 public regression files and64 private files/maps remain exact. Final compiled source hashes and immutable-source guard pass. The world arithmetic audit in flex-world-arithmetic-review.md identifies one rounded ancestor translation addition per axis under the required identity premises, plus separate corner arithmetic. Those stages remain to be connected without double-counting the analyzer's existing parent-world addition.

See flex-size-domains-receipt.json and output/flex-size-domains-r1 for source and artifact bindings.
