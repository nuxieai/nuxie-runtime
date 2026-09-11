# Immutable-target validation

The target identity is pinned in TARGET.md. Revert verification compares the complete repository tree, not a sample of runtime files. Native qualification from the reverted implementation is invalid for this target.

Run `python3 tools/html-to-riv/validation/check-target-runtime.py` before and after any build or feature gate. It checks staged/committed and working-tree changes against the pinned baseline outside this compiler module. The compiler must own its package, lockfile and authoring dependencies without changing the runtime's resolution or features. Source identity alone is necessary but not sufficient: each native receipt must also bind the effective dependency graph, build command/features, environment, binary hash and unchanged baseline import path.

For each feature:

1. Record accepted computed semantics and context-dependent rejection conditions. Identify an existing wire record/property or describe the proposed file-level composition.
2. Test Rust, CLI and WASM/JavaScript output and diagnostics. Parse the emitted file using the baseline importer. No runtime policy artifact may be required.
3. Render through unchanged public import/draw APIs. Diagnostic source maps may only read geometry. Include a test that imports `.riv` with all compiler metadata discarded.
4. Compile once, then resize the same original and cloned artboards through narrow/wide/return sequences, including changed height/aspect ratio. Never recompile inside that test.
5. Compare pinned Chrome geometry and actual native PNGs under the existing tolerances. Inspect complete unique image pairs, retain failures, and bind evidence to exact inputs and binaries.
6. Include composition, malformed/resource-boundary and unsupported-context controls. Broaden regression according to affected semantics.
7. Mark qualified only for the proved profile. If native semantics or a composition fail, preserve the reproducer and investigate another existing-object encoding or record the limitation. Runtime enhancement proposals are separate work.

Current evidence: PR #629 restored the full pre-PR tree; the mutation inventory covers 778 changed paths; source audits cover all 36 former capabilities, 44 renderer/interface/stream files and 62 runtime/vendor paths. These are audit facts, not replacement visual qualification.
