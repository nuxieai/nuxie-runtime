# Wave C9 Catch-margin correction

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

This is the one-line correction required by independent rereview receipt
`32313e8b0773cc62e0043feb47fae40421e7716e`.

`state_machine_event_test.cpp#8` now widens `f32::EPSILON` before applying
Catch's `100 * epsilon * abs(expected)` relative-margin arithmetic. The
rejected expression performed the multiplication in `f32` and widened only
the rounded result.

No fixture, action, assertion, classification, outcome, evidence locator, or
other Wave C9 row changed. The corrected Wave C9 topology remains 28 passing
cases, two genuine expected-red cases, and 16 pending source-owner blockers.
This correction does not self-accept the wave.
