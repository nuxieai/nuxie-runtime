# Wave C14 scroll-velocity exact test-port candidate

Status: **CANDIDATE; PENDING INDEPENDENT REVIEW**

This narrow Wave C14 slice covers all four active Catch cases in pinned
`scroll_velocity_test.cpp` at upstream SHA
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. It changes correspondence
evidence only, makes no production behavior change, and does not declare the
slice accepted.

## Exact census

- four of four cases are distinct discoverable executable ports;
- classifications: four structured C++ host-API adaptations;
- outcomes: four live-owner passes, zero expected-red, zero pending;
- the former claim that Rust omits `velocityX`, `velocityY`, and
  `scrollActive` is stale and is not repeated in this shard.

## Body-level evidence

Every test imports the exact pinned fixture, instantiates the default artboard,
selects the first real ScrollConstraint occurrence, performs the initial
artboard advance, and observes the retained constraint and owned physics
through `RuntimeScrollConstraintSnapshot`. The first three also instantiate
the exact `State Machine 1` and preserve every pointer coordinate, pointer
order, advance duration, release, intermediate assertion, and terminal
assertion. Case 2 retains the exact `for` bound of 600, `dt = 0.016`, early
break on stopped physics, and terminal zero-velocity assertions. Case 4 sends
the exact programmatic `scrollPercentY = 0.5` update to the live occurrence and
retains all three idle assertions.

The first three cases use a structured adaptation because pinned
non-deterministic C++ ScrollPhysics samples
`std::chrono::high_resolution_clock` while Rust requires the host to supply the
pointer timestamp. Timestamp `1.0` deterministically creates the elapsed tick
needed by the pinned nonzero-velocity assertion; no numeric velocity magnitude
is invented. Case 4 uses a structured adaptation only for the absent generated
C++ setter symbol: Rust resolves the exact schema key and writes the same value
through the public generic setter to the same retained occurrence.

No fixture substitution, diagnostic recomputation of velocity/activity,
test-local physics, proxy assertion, placeholder panic, aggregate test, or
expectation change is used. The existing four test bodies already carry the
exact live evidence, so no locator rename or test-source edit was necessary.

## Gates

- focused non-incremental suite: 4/4 green;
- strict pinned identity, ordinal, source-line, exact-name, classification,
  outcome, structured adaptation, and evidence-locator validation: 4/4 green;
- pinned checkout, source, and both RIV fixture blob identities: green;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence-checker unit suite: 24/24 green;
- JSON parsing, scoped diff, and default non-test artifact checks: green.

Every relied-on Cargo invocation uses `CARGO_INCREMENTAL=0` and disables
incremental compilation for the invoked test or release profile.
