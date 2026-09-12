# Next preferred-image constraint experiment

Source-backed proposal only; no new files have been compiled or rendered for this plan. Keep public diagnostics until actual native and public-interface evidence exists. Read against `src/image_preferred_constraints.rs`, `src/image_responsive_constraints.rs`, and the completed preferred propagation/coefficient experiments.

## First candidate: point minimum with percentage maximum

For one preferred percentage dimension `p × B`, point minimum `L`, and percentage maximum `q × B`, the used preferred dimension is:

`max(L, min(p, q) × B)`.

Here `B` is the original parent content dimension. Fold only the same-basis percentage coefficients: emit preferred coefficient `c = min(p, q)`. Remove the now-accounted percentage maximum. Retain the authored point minimum on the preferred axis and propagate its ordinary point minimum through the intrinsic ratio onto the automatic axis. Emit both minima explicitly. No maximum is needed on either axis. The Image remains on the existing ratio owner.

For width50%, min-width150px, max-width40%, emit width40%, min-width150px, min-height100px, heightAuto and aspectRatio1.5. The result stays responsive and the minimum can legitimately exceed the original maximum. The same operation works when q exceeds p: the source percentage maximum is redundant before the point minimum is applied. It must not subsequently cap that minimum.

This combines two already evidenced mechanisms rather than relocating percentages to the automatic axis. Preserve selected coefficient provenance and the point-transfer ideal/native arithmetic difference. Do not change authored Style or use a viewport to choose the branch.

## Second candidate: homogeneous preferred percentages plus independent automatic bounds

First fold homogeneous preferred-axis percentage bounds exactly as the current compiler does: `c = max(preferredMinCoefficient, min(p, preferredMaxCoefficient))`. Remove those preferred bounds. Then apply the existing opposite-axis constraint composition to the same original owner using the selected percentage c. Automatic point/percentage constraints keep their authored units and original parent-axis basis. Whenever an automatic maximum is present, its synthetic preferred-axis maximum must be c, not the original p.

For width50%, min-width75%, max-width60%, max-height100px, the source result is width75% with automatic max-height100px and synthetic max-width75%. Both minima remain explicit. A stale synthetic max-width50% is a discriminating wrong implementation: it changes the already-clamped authored dimension. This hypothesis does not require nested owners or host behavior.

## Finite first matrix: sixteen files

Use the existing opaque96×64 asset, zero image padding, flex-start alignment, default nonflexible reset, a following8×8 tail, and a definite100%×100% parent with20px padding. Run each row below with preferred width and preferred height, in row and column parents: four variants per row, sixteen files total. Mirror CSS property names; keep the numerical values specified.

| Family | Preferred source constraints | Automatic source constraint | Ordinary candidate |
| --- | --- | --- | --- |
| Mixed cap below preferred | preferred50%, preferred min150px, preferred max40% | defaults | preferred40%; preferred point min150; ratio-propagated automatic point min |
| Mixed cap above preferred | preferred50%, preferred min150px, preferred max75% | defaults | preferred50%; preferred point min150; ratio-propagated automatic point min |
| Both axes, automatic point maximum | preferred50%, preferred min75%, preferred max60% | automatic max100px | preferred75%; automatic max100; synthetic preferred max75%; explicit zero minima |
| Both axes, automatic percentage minimum | preferred50%, preferred min75%, preferred max60% | automatic min80% | preferred75%; automatic min80% in its original parent-axis basis; explicit opposite zero minimum; no maxima |

Reuse the same imported original and clone through240×240 →390×320 →768×560 →240×240, producing128 actual frames. Inspect preferred/automatic owner dimensions, Image content and following-tail position. Preserve full pixel failures at existing tolerances, including fractional tail edges. Check exact source/asset/file identities and every distinct full-resolution visual pair or verified transfer. No existing files need rerendering.

These rows distinguish coefficient selection, minimum-over-maximum priority, the point minimum crossing the responsive coefficient on resize, stale guard coefficients, and opposite percentage bases in a non-square padded parent. They do not establish different intrinsic-ratio rounding; keep the existing arbitrary-ratio numerical guard evidence separate.

## Do not generalize the reverse mixed-unit ordering yet

A preferred percentage minimum with point maximum has a different expression:

`max(l × B, min(p × B, U))`.

Simply emitting preferred `max(p,l)%` with maximum U is incorrect when l×B exceeds U, because the percentage minimum must win. Example: width50%, min-width40%, max-width120px gives width120 at parent content width240 but width288 at parent content width720. A fixed propagated automatic maximum of80 would incorrectly prevent the latter image height192.

The failed earlier outer-clamp/inner-ratio candidate already shows stale measurements in two cross-axis orientations; repeating it unchanged would add no evidence. A new candidate for this reverse ordering needs an explicit way to retain both the original percentage basis and the phase-dependent normalized maximum. Keep this as unresolved, not impossible. Likewise, simultaneous preferred point bounds plus automatic percentage bounds need dynamic combination of different bases; the sixteen-file matrix above does not claim them.
