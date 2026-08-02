Completed P2-a without committing.

Highlights:

- Added WorkPool-based async image decoding with cancellation and VM-thread Promise settlement.
- Wired read-only `width`/`height`; documented non-ORE `view` omission.
- Completed predecode `nil` and factory-backed image assignment behavior.
- Propagated runtime images through data bindings to Image draw owners.
- Updated all manifests and counts: partial 20, pending 86.
- Exact scripted golden result: 320 entries, 652 segments, zero divergences.
- All requested gates pass.
- `Cargo.lock` was updated only by Cargo.
- Preserved `.p2e-body.txt` untouched.

Full details: [P2A-report.md](/Users/levi/dev/worktrees/nuxie-p1c-importers/P2A-report.md)