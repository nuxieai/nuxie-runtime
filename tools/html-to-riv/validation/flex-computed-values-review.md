# Staged flex computed values

`src/flex.rs` now models grow, shrink and basis independently and parses longhands, shorthand, CSS-wide values and supported px/em/rem/percentage/auto bases. It remains compiler-private and does not change the public declaration whitelist. Native admission and whole-parent guards are separate work in `flex-factors-plan.md`.

The authoring reset remains 0 0 auto; CSS initial/unset resolve to 0 1 auto. Inheritance copies computed lengths and unresolved percentage descriptors. Shorthand omitted basis remains 0%, while explicit numeric zero is a point length. Basis-first and basis-last forms are accepted around the contiguous factor pair; the unsupported middle-basis form is rejected. Failed updates preserve the previous value. Factors have a provisional finite nonnegative limit of1000000, which is an explicit compiler limit, not a CSS grammar rule or aggregate arithmetic proof.

The four focused tests cover shorthand expansion, independent factors, comments, signed zero, relative basis lengths, reset/inheritance, longhand isolation and invalid values without mutation. The pinned Chrome probe at `output/flex-factors-native-r1/shorthand-chrome.json` independently checks basic shorthand grammar and omitted basis behavior. Signed-zero normalization follows CSS zero-length parsing; it is not a separate native layout qualification.

Primary grammar reference: https://www.w3.org/TR/css-flexbox-1/#flex-property . The target remains pinned Chrome, including its omitted percentage basis behavior. No claim is made for intrinsic/content basis keywords, calc or broader CSS units.
