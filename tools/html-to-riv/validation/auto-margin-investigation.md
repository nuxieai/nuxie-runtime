# L07 auto margins — investigation

Status: independent browser references captured; compiler implementation pending.
This is not a support or pixel-qualification claim.

The current compiler stores margins as four floats and parses both shorthand
and longhands through the nonnegative length parser. Four public CLI rejection
inputs and diagnostics are preserved in
`output/playwright/html-to-riv/auto-margin-initial/`, one for each direction.
All exit1 with `unsupported-value` for `auto`.

The format already has per-edge `margin*UnitsValue`; runtime `YGUnit::from`
decodes3 as Auto. `LayoutComponentStyle::apply_item_style` preserves the unit
for items with layout parents, and `layout_style_applier::auto` maps it to
Taffy's auto margin. This is a promising existing transport, not yet proof that
all CSS auto-margin behavior matches. Root/no-layout-parent handling explicitly
forces point units and needs a separate test.

`validation/auto-margin-oracle.mjs` captured Chromium153.0.8010.12 geometry:
128 scenes ×3 widths (240/390/768) =384 viewports. Four flex directions,
eight edge patterns (main start/end/both/shared, cross start/end/both, all),
and free-space/overflow/wrap/wrap-reverse modes exercise alignment interaction.
Each reference records original HTML/CSS and all element rectangles.
Output: `output/playwright/html-to-riv/auto-margin-oracle.json`.

The initial free-space column fixture was too short. Before adopting the
references, its height was corrected to280px and an assertion added to verify
positive remaining main-axis space. Item-size assertions guard against shrink
changing the intended case. The corrected generator completed successfully.
Rows at240 intentionally become constrained while wider rows have free space.

Implementation target: accept `auto` per physical edge and in one-to-four-value
margin shorthand, combined with currently supported nonnegative lengths.
Retain cascade, variable substitution, inherit/initial/unset and source errors.
Padding must continue rejecting auto. Percentage and negative margins remain
separate L15/L16 items. Use runtime layout when resizing the original bytes;
do not resolve auto margins to browser coordinates during compilation.

Next gates: typed margin representation and unit emission; public tests and
original/clone runtime comparison against the384 references; root/auto-size,
cascade, shorthand and mixed fixed/auto cases; realistic compositions;
native/WASM parity; native and vector geometry/pixels; actual image review;
full regression. No executable rebuild has been made for L07 while L06's
full native session47222 is still running.

Compiler source implementation added (not yet built or qualified): typed `Margin::{Auto,Px}` stores each edge; shorthand accepts one-to-four mixed auto/nonnegative lengths; longhands share parsing; emission uses point1 or auto3 with zero scalar for auto. Padding parsing stays length-only. Inherit copies the typed values and initial/unset retain zero. Public tests cover expansion, variables, case-insensitive auto, resets, inheritance, important and longhand precedence, and exclusions. A public runtime test imports each of128 original CSS scenes, installs existing alignment policies, and compares original/clone layout after all384 viewport references. Both test files were rustfmt parsed successfully, but execution is pending. No binary rebuild during active L06 regression.

Prepared136 visual fixtures in `validation/auto-margin-cases.json` (not yet merged into the active main corpus). Eight supplemental scenes cover four root shorthand patterns and four realistic compositions: toolbar, bottom-action cards, centered panel, and reversed wrapping. Their24 Chromium references and screenshots are in `auto-margin-extra-oracle/`; generator `validation/auto-margin-extra-oracle.mjs`. Browser-only narrow composition screenshots inspected. The initial cards extended below320px; reduced their height/type/padding before adoption so both actions are visible at240px while still having positive auto space. Geometry checked both action visibility and positive free space at all widths. This is fixture preparation, not native qualification.

Pre-build validation: `cargo check -p nuxie-html-to-riv --features native-glyph-controls --tests` passes (`/tmp/auto-margin-check.log`, session57086 exit0). This is compilation only, not executed test evidence. Added four root shorthand references to the runtime oracle:132 scenes/396 widths, doubled through original and clone. The supplemental Chromium generator reran successfully with permanent card visibility/free-space assertions (8 scenes/24 widths, `/tmp/auto-margin-extra-oracle.log`). Recorded active L06 executable and corpus hashes were checked unchanged after cargo check.

First executed tests: public auto-margin tests2/2 pass; original/clone runtime comparison fails with908 coordinate mismatches (`/tmp/auto-margin-first-tests.log`, session30911 exit101). First row/main-start/free example at390px yields a.x33.75 vs10, b.x247.5 vs200, c.x366.25 vs295. This suggests extra distributed alignment space after auto margins consume it, but cause is not yet verified. Preserve the entire132-scene oracle and failing test. L07 remains unqualified; next action is runtime/Taffy diagnosis, not tolerance changes.

Main-axis cause confirmed in Taffy `distribute_remaining_free_space`: positive remaining space was allocated to auto margins and reused for justification. Setting the remainder to zero after allocation reduces908 coordinate mismatches to48 (`/tmp/auto-margin-main-fix.log`, session39117 exit101). All residual failures are cross-axis overflow auto margins. Cross-axis resolution was distributing negative free space into the start margin; a second patch retains physical-start placement and assigns the deficit to the opposite margin, following [Flexbox cross-axis margin resolution](https://www.w3.org/TR/css-flexbox-1/#algo-cross-margins). Its oracle test is running under session44242 (`/tmp/auto-margin-cross-fix.log`). Neither patch is pixel-qualified yet.

Cross-axis fix verified: session44242 exit0; all132 scenes/396 viewports pass for both original and cloned scenes (`/tmp/auto-margin-cross-fix.log`). Added two independent Taffy regressions for main-axis auto-margin allocation before four justification modes and cross-axis overflow with one/both auto margins. All100 Taffy library tests pass (`/tmp/auto-margin-taffy.log`, session95739 exit0). The136 visual fixtures have now been merged into cases.json. Full public suite then native publisher/probe build is running under16701 (`/tmp/auto-margin-module.log`, `/tmp/auto-margin-build.log`); WASM build under22293 (`/tmp/auto-margin-wasm.log`). Pixel/parity qualification still pending.

Full public suite226/226 and native publisher/probe build pass (session16701 exit0, `/tmp/auto-margin-module.log`, `/tmp/auto-margin-build.log`). WASM build passes (session22293 exit0, `/tmp/auto-margin-wasm.log`). Added checked-host coverage compiling once at390 and probing240/390/768 with auto margin and space-evenly, including overflow. Current validation: native pixels408 viewports session48019 (`/tmp/auto-margin-native.log`), native/WASM parity session80700 (`/tmp/auto-margin-parity.log`), host tests session45335 (`/tmp/auto-margin-host.log`). Pixel runs remain sequential; vector/full native and actual image review follow.

Checked-host tests9/9 pass (session45335 exit0), including new compile-once auto-margin resize test. Native pixel and parity runs remain live.

Native/WASM parity9/9 passes (session80700 exit0, `/tmp/auto-margin-parity.log`), covering the expanded accepted corpus; typecheck passes (`/tmp/auto-margin-types.log`). Native408 pixel run session48019 remains active. Corrected stale SUPPORT summary entries for auto margins and already-qualified independent line alignment; auto-margin native qualification remains explicitly pending.

Focused native408/408 passes (session48019 exit0, `/tmp/auto-margin-native.log`). All12 composition pairs actually inspected across four sheets in `auto-margin-composition-review/`; no visible layout or text issue. Remaining396 box source cases are grouped into203 unique PNG pairs across34 sheets in `auto-margin-box-review/`; none of those sheets inspected yet. Partial native visual receipt records that boundary. Vector408 run active under17844 (`/tmp/auto-margin-vector.log`). Full native regression remains pending.

Native box review progress: sheets00..13 actually inspected,84 unique pairs covering213/396 source cases through exact within-run duplicates. No visible mismatch. Remaining119 unique pairs in sheets14..33. Updated partial native/contact receipts. Vector session17844 remains to be polled for terminal results.

Vector run terminal:17844 exit1,402/408 pass with6 composition pixel failures. Detailed geometry/image review still pending. Full native regression launched after vector terminal; execution hashes captured in auto-margin-full-input-hashes.json. Preserve those execution inputs while it runs.

Native visual review completed: sheets14..33 inspected, completing all203 unique box pairs and396 source cases (193 exact within-run duplicates). Together with12 compositions, all408 native pairs reviewed; receipts have no remaining inspections. Vector geometry independently audited408/408 with maximum0.0107421875px and matching source identities/hidden states. Six vector pixel failures are card and centered-panel compositions at all three widths. Four vector composition sheets generated for inspection; none inspected yet. Generation/comparison command session13573 needs terminal polling. Full native3085 regression remains active under15587.

Vector visual review complete: all four composition sheets inspected at all three widths. Glyph-edge differences visible; box/margin placement and wrapping match. Exact HTML/CSS plus both PNG hashes transfer all396 box pairs from completed native review. Vector receipt remaining[] means inspection complete, not pixel qualification:402/408 pixels pass and6 text failures remain. Geometry408/408 passes. Full native session15587 remains active and is the outstanding L07 native qualification gate.

Full regression15587 remains live but reports a failure in pre-existing `space-break-nonbreaking` at390. Browser/native geometry is identical. Browser PNG cannot be decoded by Pillow; native PNG decodes390x320, and metrics/diff were never written. This points to screenshot artifact failure rather than an evidenced layout regression. Preserve original artifacts and obtain terminal error details before deciding a scoped rerun. No qualification claim for this full run.

Disk diagnosis: free space fell to291MiB and the failed browser screenshot is zero bytes. Removed8 disposable executables in target/debug/deps matching only auto_margin/wrap_reverse/distributed_spacing/align_content test names (600853632 logical bytes). Source, test logs, screenshots, runtime publisher/probe/replay binaries retained. Free space observed afterward5.5GiB. Full process15587 still active; do not restart it until terminal. Review its terminal errors and rerun only affected invalid captures in a separate output.

L07 final native qualification: full regression15587 exited1 with3084 passes and one ENOSPC screenshot write failure (`/tmp/auto-margin-full.log`). The zero-byte Chromium image and failed run remain intact. Isolated same-input retry47545 passes1/1 (`/tmp/auto-margin-capture-retry.log`). All3074 remaining image pairs have exact source and both PNG identity with reviewed baselines; the one retry pair also matches its reviewed baseline exactly. All five input hashes verified unchanged through retry. Combined coverage3085/3085 is qualified for native; full receipt remaining[]. This is a full run plus one evidenced infrastructure retry, not a clean full-run exit. Six reviewed vector text pixel failures remain.
