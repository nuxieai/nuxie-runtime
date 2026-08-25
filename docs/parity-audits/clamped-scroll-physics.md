# `ClampedScrollPhysics` paired audit

Upstream owner: `src/constraints/scrolling/clamped_scroll_physics.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

Rust owner: `crates/nuxie-runtime/src/constraints/scrolling/scroll_physics.rs`.

Verdict: behaviorally equivalent. The concrete clamped variant shares the
retained `ScrollPhysics` lifecycle in both implementations; a shared Rust file
does not make this C++ owner partial.

The scrolling paired audit verified `advance`, `run`, range limiting, stop
state, and the pinned `fminf`/`fmaxf` clamp edge behavior for NaN and reversed
bounds. The deterministic host timestamp is the approved Rust host-clock
adaptation already recorded for the scrolling owner family.

This audit supersedes B6-0134 and the older F4/F10 candidate note, both of
which predated the concrete retained physics owner and its parity fixtures.
