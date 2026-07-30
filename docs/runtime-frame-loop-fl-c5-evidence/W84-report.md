Implemented the W84 round-10 corrective without committing.

- Binding fail-closed detector rules, macro fragments, qualified paths, use trees, module aliases, and exact trait/type-qualified anchors implemented.
- Exact bidirectional registry matching and substitution negatives added.
- Candidate checklist and publication/citation corrections completed.
- Structural tests: 77/77.
- Runtime/tools/nuxie: 726/726, 823/823, 147/147.
- Both goldens: 317/317 entries, 647/647 segments, zero divergences.
- Clean-cache `--locked` detector build, formatting, and diff checks pass.
- Standards review: zero findings.
- Spec review: zero findings.
- Live checker reports only the two intentionally deferred E8 fingerprint/provenance gates.
- Nine intended modified files; no untracked files; no commit.

No rm-style command was directly issued. A Make target attempted an internal protected upstream cleanup, so the scripted golden comparison was run directly with the same runners and passed.

Goal usage: 641,640 tokens over approximately 45m 43s.