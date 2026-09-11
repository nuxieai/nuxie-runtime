# CSS order on the immutable runtime

The compiler precomputes each child's integer ordering key, stable-sorts siblings by `(order, original DOM index)`, then applies the existing direction-dependent file emission. Layout and painting follow the sorted sequence while selector matching, source identities and source-map order retain originalDOM positions. Host roots participate in the same algorithm. Parent order is copied only for explicit inherit; initial/unset/default compute to0.

The public range is one signed32-bit CSS integer (-2147483648 through2147483647). Decimal/exponent tokens, dimensions, multiple tokens, overflow and unadmitted CSS-wide forms reject with source diagnostics, including declarations that are unmatched or overridden. CSS math remains separately scoped.

The compiler now preserves Number token spelling through custom-property substitution. This prevents cssparser's serialized saturated integer field or rounded float from silently changing order or turning1.0/1e2 into an integer. Public tests cover direct and substituted boundaries, adjacent integers near2^24, signed zeros, comments and invalid lexemes.

The immutable layout component consumes attached child order. Pinned Chromium's [FlexChildIterator](https://chromium.googlesource.com/chromium/src/+/refs/tags/153.0.8010.12/third_party/blink/renderer/core/layout/flex/flex_child_iterator.cc#28) stable-sorts ascending order before direction reversal. No new runtime property or wire object is introduced. The pinned-Chrome reverse painting decision documented in flex-direction-review.md continues to apply.

## Validation plan and current evidence

public-order-cases.json has19 scenes: all four directions with stable ties and mixed negative/positive orders, true sorted overlap with alpha paint, host roots, CSSwide/default/inheritance, variables and numeric precision, originalDOM selectors, responsive percentages and intrinsic min/max sizing. Real native pixels, geometry and original/clone same-file resizing are required. Public transport parity covers the same corpus. The existing admitted corpora are also differentially compiled against the previous frozen compiler to detect any changed bytes/maps when order is absent.

The following final checkpoint qualifies L02 within the documented profile.

## Final checkpoint

All 19 scenes pass 152 geometry and pixel comparisons, plus 304 canvas-clear independence checks. The same emitted file is resized on original and cloned instances. Twenty-four viewport pairs were directly inspected; exact decoded image crop/white-extension checks cover the remaining 128 pairs. The compiler checkpoint regenerates all 19 scenes and source maps byte-for-byte.

All 70 Rust tests, 18 Node transport tests, TypeScript and the immutable source guard pass. A differential compilation of 168 distinct pre-existing fixtures produces exactly the same Rive and source-map bytes as the previous frozen L01 compiler. This protects previous output without turning existing rendering limitations into passes.

L02 is qualified for signed 32-bit integers in the admitted single-line flex box profile. CSS math, other layout modes, and future properties still need their own admission and combined validation. Runtime, renderer, schema and shared dependencies remain unchanged. Evidence and hashes: `validation/public-order-receipt.json`.

## Resource review before commit

The first sorting implementation retained a full computed style for every sibling. Each style owned a deep copy of inherited custom-property values. A 1,024-child document inheriting one 48,000-character custom value used 57,475,072 bytes peak RSS in the candidate, versus 6,438,912 bytes in the previous compiler, measured with `/usr/bin/time -l`. The accepted request, logs and outputs are preserved in `output/order-memory-r1`.

A lightweight order prepass replaces that batch of styles. It retains only child identity and integer order, resolving a temporary variable environment when the winning order needs one. Full styles are then computed sequentially during emission. This keeps existing input limits and semantics instead of rejecting large inherited environments. Final output-identity and resource measurements are recorded below.


After the fix, peak RSS was 6,471,680 bytes for the inherited-value request and 6,520,832 bytes when each child also resolved `order:var(--rank)`. Both final outputs and source maps are identical to the previous compiler. Logs and requests are in `output/order-memory-r2`. These are measured runs, while the structural fix is that the sorting batch no longer retains variable environments.

The final compiler still reproduces all 19 rendered scenes exactly; `output/public-order-checkpoint-r2/manifest.json` binds the current source/binaries to those unchanged native artifacts. The repeated differential check passes all 168 previous fixtures (`output/order-byte-regression-r2/manifest.json`). All 70 Rust tests and 18 Node tests pass, including losing invalid declarations and custom environments after the lightweight prepass.
