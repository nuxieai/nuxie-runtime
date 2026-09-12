# Conditional final wrapping positioning bound

`src/wrapping_position.rs` bounds the last eleven-record-per-item positioning stage of the existing private graph. This is not public admission, a Chrome comparison, a mask certificate, or a full record/evaluation certificate. Runtime sources and renderer are unchanged.

## Arithmetic result

Let p be the actual native slot cross location, h its machine cross extent, L the maximum native extent in its actual line, f the physical slot alignment fraction and a the desired physical fraction. The current emitted graph targets

```
p + (L - h) * (a - f).
```

The new proof consumes the source-derived coordinate and carry facts and returns finite binary32 intervals for excess, offset and target, plus absolute errors against that expression. Its input association remains conditional: the coordinates, row, fractions, logical item order and final records must belong to the same closed graph. A source-level function signature does not certify generated record fields or runtime dependency evaluation.

The top-landmark error uses the existing conservative 31 eta budget. The f=0 landmark operations against the actual machine slot location are a subset of that ledger; charging its native-placement and pair terms again is conservative. The top machine range expands the native location envelope by this error with directed endpoints.

For nonnegative measured height and line maximum, the exact difference of rounded operands has magnitude at most their maximum upper bound. The graph's exact sign flip followed by its destination-local addition contributes one rounding error. Add both incoming measurement errors. Multiplication by a-f uses one further rounding allowance; signed half factors can round at subnormal values, so this is not described as exact. The final top-plus-offset sum adds both incoming errors and its own rounding allowance. All binary64 bound operations round outward, and every actual binary32 endpoint operation rejects infinity.

The audited `TranslationConstraint::constrain` source multiplies the copied coordinate by its factor, transforms destination-local coordinates with the parent, and ends with `old*(1-strength) + new*strength`. Under the required identity-linear, zero-anchor, no-offset, no-clamp, strength-one template, the last visible world copy is exact for finite old/new coordinates. `wrapping_coordinates::prove` already encloses both initial visible world coordinates. The new proof encloses the target, establishing finite subsequent copies inductively, conditional on current dependency evaluation. This differs from the subtract/FMA form of `DistanceConstraint` interpolation.

These statements are scalar machine arithmetic bounds. They do not bound native-versus-Chrome layout error, assure an acceptable pixel tolerance, establish raster coverage, or prove lifecycle scheduling.

## Visible-size semantic discrepancy remains

The current emitter measures the slot extent h, while checked base records may give its visible child another definite extent v. Consequently the formula above cannot generally mean CSS alignment of the visible child.

For example, consider a native line of height L=80, a start-aligned slot of height h=50 at p=0, and a visible child of height v=20. Requested end alignment has a=1, f=0. The graph currently targets 30; the visible child's desired coordinate is 60. This is an algebraic counterexample, not a native rendering observation.

With the shared native line origin implicit in p, the desired formula is

```
p + (L-h)*(a-f) + a*(h-v).
```

Thus the missing correction is `a*(h-v)`. It vanishes for equal sizes or physical start alignment. The private arithmetic proof deliberately bounds the expression actually emitted and does not impose size equality to redefine supported scope. Repairing the emitter must obtain v without introducing a constraint feedback cycle from a visible transform back into the graph that moves it. That requires separate record topology/evaluation and numeric validation. Merely adding this term to an error budget would conceal a semantic error.

## Tests and remaining integration

Five unit tests exercise every partition of five unequal extents across positive/negative/zero positions and all five possible signed correction factors; propagate nonzero top/measurement errors through cancellation; exhibit subnormal half rounding; reject final sum overflow, invalid errors and invalid factors; and execute the unequal-visible-extent counterexample across every positional fraction. They model scalar arithmetic directly and are not native scene tests. The module is declared privately alongside the other wrapping modules. All five tests pass in the frozen scalar/position R2 build, which records396 Rust and56 Node tests.

No fixed epsilon, visual tolerance, public route or runtime mutation was introduced. The closed composition now binds this result together with the sizing-record grammar; the visible-size correction must be repaired and retested before treating the present formula as general visible alignment.
