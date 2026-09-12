# Authored domains, closed wrapping composition and conditional normalizer

This checkpoint connects source-derived dimensions to the existing private wrapping emitters and implements the positive-branch normalizer arithmetic proof. It does **not** admit public wrapping, certify a complete line gate, or claim new native pixels. The runtime, renderer, schema, shared dependencies and host behavior remain unchanged.

## Implemented behavior

`wrapping_domains.rs` resolves parent dimensions from the caller's entire viewport domain, then slot dimensions from those original parent content axes. It joins authored NumericStyle metadata to the actual checked record fields, rejecting duplicate/missing/extra metadata, stale values and unsupported automatic minima. Cross-axis selection follows the bound native direction. Zero-inclusive viewport enclosures stay zero-inclusive; a percentage-only cross size cannot acquire a fabricated positive lower bound. Its owned facts are private and read-only.

`wrapping_composition.rs` consumes those domains, accepts explicit positional fractions and an experimental epsilon, and checks exact sizing plus paint expansion against the caller's record budget before emission. The existing emitters append ordinary records to a clone, leaving the caller's input unchanged even on failure. The candidate owns generated records, trace IDs, domain bounds, alignment inputs and charged costs without a mutable accessor or public export route.

An independent post-emission check compares the base prefix byte-for-byte, except for the identified original colors deliberately hidden by paint replication. Generated kinds and typed parents exclude all native layout participants. Measurements target only the independent slots; scalar helpers reference earlier helper Nodes and root-only scalar parent chains, preventing feedback through the moved visible owners. Clipping/draw references must identify their expected generated types. This closes layout-input preservation for this construction path. The native source justification is in `wrapped-composition-layout-isolation.md`; constraint field arithmetic and pixel correctness remain separate.

`wrapping_normalizer.rs` proves the conditional positive scalar branch: finite normal square/sqrt/division/product ranges; the actual rounded length threshold; a correlated projection interval; rounded subtraction and strength-one FMA interpolation error; and a strict lower bound above32768 with finite doubling, sufficient for exact65536 saturation under the bound clamp shape. It rejects zero/tiny inputs for this positive-branch lemma and large finite cancellation cases. Zero has its separate conditional early-return behavior. The helper still requires final graph binding of the zero target/anchor/orthogonal coordinate, identity transforms, distance/mode/strength and doubling/clamp. It is not yet supplied with a source-derived dead interval.

## Tests and review

Nine new tests cover:

- Original parent percentage bases through conflicting point bounds, correct physical cross-axis selection, no viewport floor, stale metadata and exact role ownership.
- Candidate counts1,2,3,8 across native direction/reversal/alignment combinations, deterministic ordinary encoding, exact record budget, untouched inputs on failure, layout prefix mutations, forbidden participants, invalid helper attachment and measured-target references.
- Normalizer threshold-adjacent values, zero/subnormal/overflow inputs, broad correlated intervals and binary32 rounding boundaries. A scalar operation test at2^42 demonstrates projection65536 followed by interpolation0; the proof rejects it. This is a scalar arithmetic test, not a new native scene capture. Conservative proof rejection is unresolved, not an impossibility claim.

A parallel reviewer found no correctness blocker in the domain mapping, normalizer or closed composition within their explicitly conditional scopes. A separate immutable-source audit confirmed generated layout isolation, including the essential exclusion of LayoutParticipant. Root checked provider traversal, alignment mapping, encapsulation, generated parent/reference checks and test results. None of these reviews substitutes for the remaining full graph numerical proof or Chrome/native qualification.

## Frozen validation

Build: `output/wrapped-composition-build-r1/frozen`, with293 bound source/artifact entries. All **379 Rust /56 Node tests**, strict TypeScript, native/WASM builds and the immutable source guard pass. Inputs are unchanged throughout build and transport checks.

- CLI SHA-256: `4202e0e145fbfa2103570dd6b6255401c65c6f5f95a814e2fc54f5f8764ab62f`.
- WASM SHA-256: `c92d9224607d1ea5d023bd33869309bca528df094b2c98eac0e9f2882f064a4c`.
- All282 prior image request/file/map outputs reproduce exactly in `output/wrapped-composition-existing-r1`.
- All794 historical outputs reproduce exactly in `output/public-transport-malformed-wrapped-composition-regression-r1`.
- `python3 validation/wrapped-composition-evidence.py` verifies7,853 current/frozen/artifact bindings, actual regression bytes and build logs without compiling or rendering. Output: `output/wrapped-composition-verification-r1.json`.

No new browser/native capture or visual inspection is claimed. The public compiler does not call these helpers and its checked emitted files are unchanged. Prior native evidence and every retained text/image/threshold failure remain historical evidence with their original scope. Private candidate serialization tests do not certify rendering.

## Remaining work

Implement a validated native coordinate accumulator envelope and carry-max reconstruction bounds. Derive source-dependent epsilon and separated-line dead intervals from those bounds and the already resolved slot dimensions. Bind every scalar normalizer and saturation premise to the actual candidate records. Establish active/inactive mask coverage, including rounding/raster margins, and charge any changed mask graph costs. Only then connect a bounded public profile and perform the full public API, original/clone resize, native geometry/pixel and visual workflow for changed output.

The earlier proposed B/R bounds and experimental1/64 epsilon are not defaults or completed proofs. The native max reconstruction error remains part of the carry obligation. All99 backlog items remain in scope, with counts13 qualified /23 partial /4 investigating /59 pending. This replacement remains local and unpublished.
