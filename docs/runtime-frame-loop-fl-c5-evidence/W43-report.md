# W43 FL-B reacceptance correction round two

Production commit:
`edddf4916e0ff0b7f55e41686704d5d988fae9f4`.

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

- Runtime library: 715/715.
- Live C++ differential suite: 815/815.
- `nuxie` library: 146/146.
- Ordinary and scripted golden comparisons: 317/317 entries and 647/647
  segments each, with zero divergences.
- Frame-loop structural tests: 59/59; the joint publication trace and final
  checker are owned by the evidence step.
- Formatting and working-tree whitespace checks are required green before
  evidence staging.

The W43 production work is landed in `edddf491`; FL-B remains pending
round-three reacceptance. This report makes no promotion claim.
