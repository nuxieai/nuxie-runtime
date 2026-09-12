# Private rectangle mask domain binding

`wrapping_masks::prove` consumes the already checked base scene and its declared viewport intervals. The base inspector excludes artboard clip overrides, nonzero artboard origins, radii, styles/insets and extra transforms. The pinned Artboard constructor supplies clip=true; generated LayoutComponent width/height defaults are0. The mask proof bounds the existing geometry rather than modifying it.

For both declared axes, viewport.upper must be at most32768. The active mask is exactly [0,32768]². The inactive mask shifts that rectangle by65536 on the physical cross axis. All constants and sums are exactly representable binary32. Coordinatewise intersection of the active mask with any declared [0,W]×[0,H] artboard clip preserves that clip; the inactive intersection is empty. Zero endpoints are conservative interval enclosures, not claims about visible zero-sized output.

This private threshold does not change the public16384viewport limit. Larger private domains diagnose an unresolved mask profile, rather than asserting that fixed masks cover it or declaring all alternative compositions impossible. A rectangle at32768.next_up supplies a direct truncation counterexample for the existing construction.

The initial serialized artboard dimensions must also be finite and contained in the same intervals. This prevents initial import/draw from bypassing the proof before the first resize. Missing dimensions have the pinned native0default. A later host resize remains within the declared domain by the output-profile premise; arbitrary external changes cannot inherit this certificate.

The returned token stores the viewport and both rectangle bounds alongside the same owned derived candidate. The paint-record reader separately binds actual rectangle size/center, default origin/radii, parent signal, clips and leader inversion. Its fixed constants must stay consistent with this geometry analysis. Numeric proof and paint-field binding supply the exact0/D signals; neither a finite normalized scalar nor a few observed line transitions substitutes for that premise.

The pinned non-MSAA renderer can intersect AABB clips before antialiasing. The actual command paths and matrices must establish that route for the scene, with source evidence for the renderer's intersection behavior. No universal artboard-unit AA margin is inferred. This proof does not establish unclipped overflow semantics, another renderer mode, arbitrary singular hosting transforms, or pixel equality to Chrome.

Tests cover both axes, zero/subnormal/fractional/maximum endpoint domains, coordinatewise active intersection/inactive emptiness, the next float beyond the current rectangle, and invalid or out-of-domain initial serialized sizes through the owned composition API. Successful private construction remains distinct from public support and visual qualification.
