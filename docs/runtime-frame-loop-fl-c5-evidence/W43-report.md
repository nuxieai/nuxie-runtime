# W43 FL-B reacceptance correction round two

Immutable combined production candidate:
`95333c41fe68ab6a2a5486874ffd0c59cd4381be`.

## Corrections

- `loopValue` now resolves the definition through the instance's retained
  typed animation handle. A caller-supplied mismatched definition can no
  longer change the observed or overridden loop mode.
- After an invalid keyed object, a following keyed property replaces the
  property cursor as a sink bound to that doomed object. Its keyframes are
  erased with the invalid object instead of leaking into the preceding valid
  property.

The two new live differentials were red first (Rust `2` versus pinned C++ `1`
for both loop-definition selection and stale-frame count), then passed with
the corrections. The seven round-one differentials remain green.

## Combined-candidate receipts

- Runtime library: 716/716.
- Live tools-enabled C++ differential suite: 816/816.
- `nuxie` library: 146/146.
- Ordinary and scripted golden comparisons: 317/317 entries and 647/647
  segments each, with zero divergences.
- Frame-loop structural tests: 66/66; the joint publication trace and final
  checker are owned by the evidence step.
- Formatting and working-tree whitespace checks are required green before
  evidence staging.

The W43 corrections remain ancestors of P3; round-four production adds the
retained-definition call path and its live differential. FL-B remains pending
post-E2 independent reacceptance. This report makes no promotion claim.
