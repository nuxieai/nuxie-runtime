# Private CSS gap integration

The real compiler now parses and computes gap, row-gap and column-gap in its private Candidate profile. It handles normal, px/em/rem/percent, shorthand and longhand ordering, CSS-wide resets, computed inheritance and variable substitution. Ordinary style fields encode physical row gap vertically and column gap horizontally in every flex direction. Normal and zero omit fields, preserving existing default output.

All16 prior gap experiments now compile directly from their original HTML/CSS into exactly the previously rendered Rive files and source maps. The old adapter supplied computed gap values; this checkpoint no longer needs that adapter. This establishes correspondence to the retained128/128 geometry and124/128 pixel evidence, including four intrinsic-percentage column failures. It is not a new native render or a broader visual qualification.

The public compiler continues to reject all gap declarations, including unmatched declarations and variables, until helper/baseline/intrinsic/percentage contexts and numerical bounds are qualified. Private candidate success is experimental only. Nonzero parent gaps explicitly exclude the current gap-free flex arithmetic model.

Validation:200 Rust tests,35 Node tests, WASM build and strict TypeScript pass. All482 public regression files and48 prior private flex files/maps remain exact. The first integration run exposed missing gap admission in the variable whitelist; tests.log preserves the failure and tests-r2.log the corrected full pass. Source hashes and immutable-source guard pass.

See gap-integration-receipt.json for bindings. The accompanying flex-native-defaults-review.md audits13 immutable source files and distinguishes automatic minima on fixed axes from zero minima on Fill axes; it does not supply ancestor world-error proofs.
