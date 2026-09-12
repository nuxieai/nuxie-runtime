# Historical wrapping clip command check

2026-09-12. Ran `python3 tools/html-to-riv/validation/wrapped-clip-stream-check.py` against the frozen `wrapped-sizing-snapped-r1` streams. No runtime/compiler source was changed, no RIV was recompiled, and no native process or capture was launched.

The offline checker passed all 384 original/clone resize frames. It observed 4,992 clip commands and 2,304 draws under an empty inherited intersection. Every mask clip occurred at the outer identity renderer matrix, every raw clip matched the baseline AABB predicate within the deliberately restricted straight-line profile, active masks preserved the existing artboard clip, and inactive masks made the intersection empty. The checker follows balanced save/restore scopes and exact binary32 translation addition; unknown state-changing commands and nonidentity linear matrices fail. It parses the selected occurrence from each cumulative stream, independently checks the artboard rectangle against the original fractional viewport metadata, and hashes each source stream and frames manifest.

Four mutated command-stream controls were rejected: nonidentity clip matrix, a cubic initial clip verb, incomplete save stack, and an unknown command. These controls demonstrate that the checker is not merely counting clip commands.

This is evidence for the older experimental fixed-epsilon candidate only. It does not qualify the new derived-epsilon/anchored implementation. Intersections are inferred from the unchanged renderer's source after observing its ordinary runtime command inputs; GPU internal clip state was not instrumented. In an already-empty clip scope, the actual renderer returns before inspecting subsequent raw paths; the receipt marks these occurrences `alreadyEmpty` rather than claiming that every submitted clip executed native AABB recognition. The 2,304 empty draw observations do not independently prove that those draws' underlying paint geometry crossed the viewport; the forthcoming native pixel campaign must include that explicit control.

The checker accepts resource/draw records only as the known non-clip command categories needed for state traversal. It is not a full serializer conformance validator, general affine-transform interpreter, or a proof of shader coverage. No existing pixel failures were changed or reclassified.

Receipt: `output/wrapped-clip-stream-r1/receipt.json`.
Receipt SHA256: `e7b2b0411a812d800fb0264d9f8592e9afa2cc7c15788f526680ba069140f2bf`.
Script SHA256: `9177d932e7e755ebc3ea1793705f78fb5bab4f3aa716b3d6ff305dce65894447`.

See `wrapped-clip-observation-plan.md` for the immutable probe/renderer identity, source anchors for intersection-before-AA, and commands for fresh observations. See `wrapped-mask-coverage-audit.md` for the remaining construction and viewport-domain premises.
