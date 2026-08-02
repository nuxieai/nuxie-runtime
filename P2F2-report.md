Completed P2F2 and left the worktree uncommitted.

Implemented:

- AudioEvent activation with dense asset ordinals, source retention, multiplied volume, default/configured mixer scheduling, and Artboard lifecycle teardown.
- Recursive Artboard engine/volume propagation and clone-safe playback ownership.
- State-machine event-system integration and public `nuxie` APIs.
- Expanded C++/Rust differentials for D18, `sound.riv`, and `sound2.riv`.
- Honest manifest and expected-count updates.

All six requested gates pass. Formatting and diff checks also pass. `Cargo.lock` is untouched.

Full results: [P2F2-report.md](/Users/levi/dev/worktrees/nuxie-p2f-audio/P2F2-report.md)

The optional broader `make port-manifest-check` still encounters the documented pre-existing missing inventory row for `core_uint64_type.cpp`; the audio manifest tests pass.