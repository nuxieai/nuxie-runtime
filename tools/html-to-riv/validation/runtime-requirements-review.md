# Runtime requirements and retained font policy

Historical qualification receipt for the initial cluster-only capability. The
compiler now requests the distinct CSS spacing capability described in
[optional-ligatures-review.md](optional-ligatures-review.md). Existing cluster-only
capability semantics and this envelope contract are preserved.

The compiler now emits RuntimeRequirements alongside every successful output.
Version 1 currently has one capability, text-cluster-spacing-v1. It is requested
for every emitted nonzero letter-spacing text style, irrespective of the current
string's clusters. Plain scenes emit an empty capability list. Ordering is
deterministic. Unknown capability names/fields fail deserialization; unsupported
versions/capabilities fail ensure_supported.

The CLI emits .requirements.json beside .riv and .map.json. WASM metadata and the
TypeScript success result expose runtimeRequirements. Corpus parity now compares
requirements as well as Rive bytes and source maps. Browser-produced card output
passes its requirements through the checked native host as well.

The reference host reads/checks the manifest before import, then selects a
retained FontAsset mode for requested cluster spacing. No environment override
is required. NUXIE_DISABLE_CLUSTER_SPACING=1 is a rejection control, not an
alternate renderer. Direct raw Rive imports retain legacy behavior.

A FontAsset policy is unset by default. Once selected it governs occurrence
replacement, the direct setter, decoding, host restoration and clear/reload.
The native integration test imports two files, selects one, replaces its font
through the public mutation paths and verifies both retention and isolation;
explicitly restoring RiveGlyph also survives subsequent replacement. Existing
font-option propagation and legacy shaping tests remain applicable.

The new checked-host test verifies successful supported import; refusal before
draw for missing manifests, future versions, unknown capabilities and fields;
restoration after a bad manifest; and loading ordinary scenes when the capability
is unavailable. This test is now in validation/run.sh.

The manifest-driven full glyph gate passes **310/311**, with only the existing
A09 fractional-edge failure. All 52 Rust tests, five JS/WASM tests (including
requirements parity), two gallery checks, TypeScript, Clippy and boundary checks
pass. The new checked-host test passes separately and is included in future full
runs. Renderer-state controls pass 27/27. New-only vector spacing passes 27/27.
The final guard simplification in the probe was recompiled and its checked-host
test rerun after Clippy; it does not change the selection logic.

All 303 native scene PNGs are byte-identical to the prior experimental full
cluster-spacing gallery, whose corrected cluster images were visually inspected.
The new host path changes metadata/policy delivery rather than rendering output.
No tolerance was widened. Chromium 153.0.8010.12, DPR 1; actual Rust Metal replay.

Reproduce:

```sh
bash tools/html-to-riv/validation/run.sh native-glyphs
node --test tools/html-to-riv/tests/runtime-requirements.test.mjs
```

The artifact envelope is not cryptographically authenticated or self-contained:
its files must be published together by the host. Arbitrary raw .riv import does
not know about the metadata. The reference adapter shows the checked path; wider
runtime integrations must honor the same contract, including replacement.
A12 still needs broader optional-ligature, fallback and Unicode-script coverage;
the capability name specifies cluster spacing, not complete CSS typography.

Artifacts: output/playwright/html-to-riv/requirements-full/gallery.html and
requirements-vector/gallery.html. The checked-host test and Rust requirements/
replacement tests are durable reproductions. Next: strengthen letter-spacing
qualification with a font that forms optional ligatures, fallback fonts and
additional clusters before moving on to word spacing.
