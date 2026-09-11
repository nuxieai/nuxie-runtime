# L20 overflow qualification

Current status: version21 overflow is qualified for the documented static subset. It has completed focused pixel,
visual, parity and lifecycle validation, plus6,271 full native regression checks
and6,261 audited exact-input/full-image baseline transfers. The303 accepted
overflow scenes are now included in the regular browser suite;930 integration
checks and all930 image audits pass. Historical entries below describe earlier
snapshots and do not override this status.

Supported behavior includes `overflow` with one or two values, `overflow-x` and
`overflow-y`, computed visible/clip/hidden coupling, rounded two-axis clips,
unrounded one-axis strips, and signed `overflow-clip-margin` with content/padding/
border origins. The public versioned host policy preserves resizing, cloning,
clearing/reinstalling policies and positioned descendants. Scrollable computed
`auto` and `scroll` remain intentionally rejected. Border origins coincide with
padding origins until border support lands; later border and per-corner radius
work must extend the existing clip policy. See
[clip-margin investigation](overflow-clip-margin-investigation.md) and
`output/playwright/html-to-riv/overflow-l20-receipt.json` for current evidence.

## Investigation history

Axis implementation investigation: [CSS Overflow3](https://www.w3.org/TR/css-overflow-3/#overflow-properties)
defines x/y shorthand order and computed-value coupling: visible/clip become
auto/hidden when the other axis is scrollable. Clip paired with visible has an
unrounded clipping region; two clipped axes can retain rounding. Clip margin
expands the selected box edge and does not affect hidden overflow.

Local implementation currently stores only `ComputedStyle.overflow_clip` and
uses the layout's rounded world path in both `draw_proxy` and
`begin_css_ancestor_clip`. Inference: axis support needs a shared clipping policy
for both paths, distinct specified/computed overflow states in the compiler, and
versioned host transport. It must retain live resize/clone behavior and cannot
substitute a finite arbitrary extent for the unclipped axis. Scroll semantics
remain excluded; mixed values requiring auto must be diagnosed explicitly until
their static semantics are supported. Prospective `overflow-axis-cases.json`
covers all nine visible/clip/hidden pairs at two radii (18 scenes).

Current status: the initial rounded overflow corpus passes both renderer profiles
with complete visual audits (81 views/profile),216 original/clone geometry checks,
27 native/WASM parity cases and108 profile-specific clip discriminators. Broader
L20 qualification remains open. Existing admission accepts `overflow: visible`, `clip` and static
`hidden`. The stacking lifecycle corpus establishes rectangular ancestor clip
behavior over original and cloned instances, but does not cover rounded clipping
or text/image edges.

`overflow-rounded-cases.json` contains27 cases: shape, text and embedded image
content; radii0/18/90px; and visible/clip/hidden. The host has a responsive60%
width, fixed height and padding. Each child crosses its host boundary. The90px
radius exercises overlap reduction on a nonsquare box. Capture240/390/768px
Chrome references, compile once at390px, and replay both renderer profiles.
Visible controls must demonstrate that clip/hidden actually remove content;
hidden and clip should be compared independently rather than assumed equivalent.

Remaining L20 work includes nested rounded ancestors, relative and absolute
descendants, text/decoration/image compositions, repeated original/clone resizing,
and explicit qualification of CSS-wide resets. Axis-specific overflow and clip
margin remain unsupported and require separate semantic/runtime investigation;
scrolling and user interaction remain excluded by the project scope. Borders and
per-corner/elliptical radii must compose with this policy when P01–P04 land.

Use unchanged geometry and pixel gates, inspect distinct outputs, retain failures,
and validate complete native/WASM output parity. The new fixtures are prospective
until their accepted/rejected behavior and renderer evidence have been collected.

Pinned Chrome153.0.8010.12 capture completed27 scenes/81 views in `output/playwright/html-to-riv/overflow-rounded-initial-oracle`. Frozen corrected v19 native/vector replays are running (sessions65376/72904). Receipt: `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 initial rounded overflow:27 scenes/81 Chrome comparisons pass both renderer profiles;216 original/clone repeated-size geometry checks and27 native/WASM parity cases pass. Native visual audit covers36/81 views (18 direct plus18 identical hidden controls), including rounded shape/image boundaries and text clipped through glyphs. Remaining45 native views and vector audit remain; no full L20 qualification claimed. The first resize test used an absolute-policy installer for a relative-only contract; corrected to the shipping probe relative installer, preserving the failed log. See `validation/overflow-investigation.md` and `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 initial visual audit complete in both profiles:81/81 comparisons each. Native54 direct+27 exact hidden-control transfers; vector18 direct+9 within-run transfers+54 audited exact-input/full-PNG cross-run transfers, with disjoint coverage checked. All108 profile-specific visible-versus-clip/hidden discriminators preserve geometry and change pixels. Sparse vector glyph/corner antialiasing differences remain within unchanged gates. Initial27-request parity and216 clone/resize geometry checks pass. L20 remains active for nested/positioned compositions, CSS-wide resets and axis/clip-margin investigation. Evidence: `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 nested expansion:24 scenes/72 comparisons pass each renderer profile;24 new public native/WASM parity cases and combined51-scene/408 original-clone resize checks pass. Native visual coverage is36/72 after inspecting absolute underlined text clip/initial and relative image unset across all widths. Remaining visual review and CSS-wide discriminators are open. Original br fixture rejected by the documented block-context rule; corrected display:block oracle and failed runs retained. Added reusable `validation/parity-fixtures.mjs`. Evidence: `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 nested audits complete:72 views/profile, native18 direct+54 exact image transfers; vector6 direct+18 within-run exact transfers+48 exact-input/full-PNG transfers. All36 profile-specific reset groups verify identical geometry, inherit=clip, initial=unset and changed pixels when the inner clip is released. Combined focused coverage is153/profile;51 public parity cases and408 clone/resize checks pass. Axis investigation captured18 prospective scenes/54 pinned Chrome views; all18 inputs are explicitly rejected by the current compiler. Runtime currently has only a two-axis rounded clip path; axis policy/transport and clip-margin support remain implementation work, not external blockers. See `validation/overflow-investigation.md` and `output/playwright/html-to-riv/overflow-l20-receipt.json`.

Axis implementation progress: `src/overflow.rs` now preserves specified x/y values and computes their coupling; current single-value admission uses this model without adding axis syntax yet. The nine visible/clip/hidden combinations and an opposite-axis override regression are covered. Existing hidden and clip remain distinct for future clip-margin behavior.

Renderer path identified: `crates/nuxie-renderer/src/hard_clip.rs` already obtains `current_state().overallClipPixelBounds` and the complete device matrix, creates a device-space clip path, and restores the matrix without discarding the installed clip. Axis clipping can use this same state access with convex half-plane intersection against the current clip bounds, retaining antialiased boundaries. This avoids an arbitrary finite replacement for the visible axis. It still needs a dedicated render API/recorded command, replay support, backend implementation, checked runtime occurrence policy and versioned compiler transport. The current hard exclusion operation is not an axis clipping implementation and must not be reused with its binary rounding.

L20 axis foundation implemented: compiler overflow values now retain specified x/y states and compute their coupling, preserving hidden versus clip and distinguishing computed auto. Existing admission is unchanged until runtime support lands. All333 compiler tests across63 result groups pass; native build passes. New native outputs for51 reviewed overflow inputs are byte-identical to previous Rive/map/requirements artifacts. WASM build56207 is in progress. Renderer current device clip bounds provide a bounded exact half-plane intersection path without an arbitrary visible-axis extent; render API/replay/backend/policy/transport work remains.

Axis value-model verification completed: WASM build passes and all51 overflow inputs pass native/WASM byte/map/requirements parity on the new compiler snapshot (`overflow-axis-model-compiler`). No public axis syntax has been admitted yet.

Renderer axis operation implemented behind the Metal renderer: `Renderer::clip_axis` defaults to explicit unsupported, recording preserves the axis and float edges, stream parsing/replay recognizes `clipAxis` and rejects unsupported backends, and the glyph adapter forwards it. The backend intersects its actual device clip bounds with the transformed local strip using convex half-plane clipping. It preserves fractional edges, restores the current matrix after installing the device path, and does not restrict the visible axis beyond the existing device clip. Three geometry tests cover fractional edges, rotation/reflection/shear, empty clips and invalid inputs. Metal-feature compilation and stream round-trip/rejection tests are running; this is not yet pixel-qualified or connected to public compiler admission.

Axis renderer verification: Metal-feature build and3 geometry tests pass; full render-stream suite7 tests passes, including precise axis round-trip, invalid-input rejection and unsupported-backend error. Actual pixel controls remain next.

Axis renderer frame wiring and pixel controls: the first stream run failed explicitly with UnsupportedOperation("clipAxis") because NativeMetalFrame lacked forwarding. Added forwarding in NativeMetalFrame and its canvas wrapper; the rebuilt frozen renderer completes144 Chrome153.0.8010.12 controls.108 pass;36 rotated/sheared x/y/both clips fail unchanged thresholds. Translation, fractional translation, scale, reflection and all unclipped controls pass. Rotated x/integer and unclipped rotation sheets directly inspected at all3 widths: native clip exposes excess teal while uncut transform and restored purple agree. This is renderer-only diagnostic evidence, not public axis syntax admission or responsive compiler qualification. Preserve both failed runs. Next: diagnose affine clip polygon/backend consumption before runtime/compiler policy integration. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-frame/failure-audit.json.

Axis renderer affine failure fixed: diagnostic output showed initial overallClipPixelBounds uses i32 sentinel extremes. Polygon intersections therefore reached billions of pixels, losing ordinary-edge precision on conversion to f32. Intersecting existing bounds with actual frame dimensions before constructing the strip fixes all36 failures. Fresh frozen renderer passes144/144 Chrome controls, with unchanged thresholds; Metal-feature geometry tests3/3 pass. Nine rotated integer x/y/both views directly inspected: clipped bounds and restored purple agree, sparse slanted-edge antialiasing remains.135 visual views remain unaudited. Original failure corpus/debug output retained. Public axis syntax remains rejected pending runtime policy, compiler/host transport, clone/resize and public parity qualification. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-finite-frame/replay.json and overflow-l20-receipt.json.

Axis renderer visual coverage now36/144: all rotated/sheared x/y/both integer/fractional views directly inspected and audited at three widths. Bounds, viewport crops and restored purple match Chrome; thin slanted-edge antialiasing differences remain within unchanged gates. New validation/audit-axis-renderer.py verifies432 distinct-axis pair comparisons across36 groups and288 restored-region controls against current PNG hashes. These controls prove axis differences and save/restore isolation; they do not replace remaining108 visual reviews or public compiler integration. Evidence: overflow-axis-renderer-finite-frame/{visual-inspection,control-audit}.json under output/playwright/html-to-riv.

Axis renderer visual audit now78/144 after directly inspecting translation and scale X/Y/both integer/fractional sheets at all widths. Placement, visible extents and restored purple agree with Chrome; fractional right/bottom edges retain thin coverage differences under unchanged thresholds. Coverage includes exact within-run full-image transfers recorded by the review tool, not additional direct inspection claims.66 views remain. Numeric144/144 and control audits432 axis distinctions/288 restored regions remain passing. Public compiler axis admission is still pending runtime integration. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-finite-frame/visual-inspection.json.

Axis renderer controls complete:144/144 Chrome comparisons pass, with audited visual coverage (120 directly inspected views and 24 exact full-image transfers). Final fractional-translation/reflection clips and all unclipped controls agree in extent/cropping/restore; thin fractional edge coverage differences remain within unchanged gates.432 axis-discrimination and288 restored-region checks pass. This qualifies the direct Metal renderer diagnostic matrix only. Runtime/public compiler axis syntax remains pending. Shared integration must cover both LayoutComponent.draw_proxy and begin_css_ancestor_clip, which currently read base.clip and clip a world rounded path. Live dimensions, transform restoration, clone policy and plain-container proxy inclusion must all be preserved. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-finite-frame/{replay,visual-inspection,control-audit}.json.

Axis runtime integration foundation: ordinary draw_proxy and deferred begin_css_ancestor_clip now share begin_layout_clip, preserving caller-owned save/restore and ClipSaved bookkeeping. Positioned-paint regression5/5 passes, including864 original/clone dynamic clip/resize draw frames. Initial cargo package-name typo failed before tests and is preserved; corrected nuxie-html-to-riv run passed. No new overflow syntax admitted. Next: occurrence policy and transform-aware strip dispatch, with proxy/path invalidation, clone handling and checked compiler/host transport. Existing direct-renderer144-view qualification remains separate from this runtime integration. Evidence: output/playwright/html-to-riv/overflow-shared-clip-runtime-tests-corrected.log.

Transform-aware axis renderer API added: clip_axis_transformed composes an additional local matrix for clipping while restoring the exact prior drawing matrix, avoiding inverse-transform roundoff. Recording/replay accepts optional finite clipAxis matrix; adapter and Metal frame/canvas forwarding included. Expanded render-stream7/7 passes. Fresh local-transform mode completes144/144 Chrome pixel controls; complete authored HTML/CSS and browser/native PNG byte identity transfers all144 reviews from the audited finite-frame run. Control audit432 distinctions/288 restored regions passes. No public compiler axis syntax admitted. Runtime occurrence policy and clone/resize/host integration remain next. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-transformed/{replay,control-audit,visual-identity-transfer}.json.

Live runtime axis policy implemented: LayoutComponent stores optional CssOverflowAxis (horizontal/vertical), clones it, keeps a drawable proxy through toggles, and computes each strip from current layout width/height and world transform. The shared ordinary/deferred clipping helper dispatches clip_axis_transformed; clearing restores the imported two-axis clip flag. An unsupported renderer fails explicitly rather than drawing without the requested clip. Positioned-paint6/6 passes, including32 new original/clone frames with axis switches, repeated240/390/768/240 sizes, clear/reinstall, deferred clips and sibling restore isolation; prior864 dynamic two-axis frames remain covered. This is runtime stream/lifecycle evidence, not public CSS or runtime pixel qualification. Atomic checked installer, versioned compiler/host transport, public admission/parity and actual runtime pixel corpus remain. Evidence: output/playwright/html-to-riv/overflow-axis-live-runtime-tests.log.

Axis overflow checked scene installer added: Artboard.set_css_overflow_axes_occurrence validates the root, all object IDs/types and duplicates before mutations, clears omitted overrides back to imported clipping, releases the artboard borrow during per-layout invalidation, and rebuilds draw order. Positioned-paint7/7 passes. New regression proves invalid lists retain prior policy, clearing restores the imported clip flag and clipPath drawing, clones retain their own axis, and clone replacement leaves original cleared. Existing32 axis lifecycle and864 ordinary clip frames remain covered. Versioned compiler/host transport, public syntax and runtime pixel qualification remain open. Evidence: output/playwright/html-to-riv/overflow-axis-installer-runtime-tests.log.

Axis overflow contract foundation added: version20, layout-css-axis-overflow-v1 and strict layout_axis_overflow entries {object_id,axis:x|y}. Validation enforces version/capability coupling, unique non-root layout targets and optional stacking coexistence; empty entries are omitted from older outputs. Public Rust exports and TypeScript version union updated. Native probe maps requirements through the checked runtime installer. Focused2/2, full compiler337 tests across64 groups and TypeScript checks pass. Featured probe build25280 is running; initial mistaken example target failed before building and is preserved. Public CSS axis syntax/emission, WASM/parity and runtime pixel qualification remain pending. Evidence: output/playwright/html-to-riv/overflow-axis-contract-{tests,full-tests,types-final}.log.

Axis version20 featured probe build25280 completed successfully. Public CSS admission and runtime scene qualification remain pending.

Public axis CSS parsing/emission implemented. overflow accepts1/2 values and overflow-x/y one; visible/clip/hidden, normal precedence, CSS-wide keywords and supported custom substitutions covered. Inherit copies computed values. Final visible/hidden coupling rejects computed auto; clip/visible emits version20 axis requirements, both-axis pairs use existing clipping. Focused6 tests pass including prior clipping regressions; old axis rejection cases replaced with unsupported scrolling pairs. Initial new test diagnostic-vector access compile error preserved. Full compiler84510 is running. New support section explicitly marks public parity/runtime scene geometry/pixels pending, separate from direct renderer qualification. Evidence: output/playwright/html-to-riv/overflow-axis-public-tests-corrected.log.

Initial public full suite84510 failed on the intentionally obsolete hidden-clip custom-property rejection. Updated it to reject hidden-visible and assert hidden-clip equals hidden; full rerun40314 active, no terminal passing claim.

Public axis overflow initial corpus passes: corrected full compiler339 tests/65 groups, fresh native/WASM builds,14 public parity cases,42 Chrome geometry/pixel comparisons per renderer profile, and combined65-scene520 original/clone resize geometry checks. Four prospective visible/hidden scenes remain intentionally rejected for computed auto. Initial replay failed on missing copied oracle PNGs; exact42 reference files copied with hashes and fresh native-images output passes. Twelve one-axis native views directly inspected: responsive clip extents, visible-axis escape, viewport crop and square clip over rounded orange background match Chrome.30 native visual reviews and vector audit remain, followed by composition/host/full regression expansion and clip margins. Evidence: output/playwright/html-to-riv/overflow-axis-public-{native-images,vector,parity} and overflow-l20-receipt.json.

Public initial axis corpus visual review complete:42 native views (24 direct,18 exact within-run image transfers) and42 vector exact compiler-input/full-PNG transfers audited. New48-scene composition matrix covers shape/underlined multiline text/image, relative/absolute children, X/Y clips, painted/plain hosts and outer rounded clip/visible controls with positioned sibling overlap. All144 Chrome comparisons pass per renderer profile,48 new native/WASM parity cases pass, and combined113-scene904 original/clone repeated-size geometry checks pass. Composition visual review/discriminator audits remain pending; no full L20 qualification claimed. Evidence: output/playwright/html-to-riv/overflow-axis-composition-{native,vector,parity,oracle} and overflow-axis-composition-resize.log.

Axis composition review now42/144 native views (12 direct plus exact full-image transfers) after unpainted text/image X/Y sheets inspected. Text line/underline clipping, quadrant cropping, outer rounded bounds and green sibling overlap agree with Chrome. Outer-clip audit found original X cases mostly fit outer height, so they prove placement rather than nested clipping. Preserved them and added24 short-outer X variants. New72 comparisons/profile and24 public parity cases pass;72 paired scene groups verify144 browser/runtime image changes across the outer clip toggle. Combined137-scene1096 original/clone size checks pass. Short-outer visual review and remaining composition native/vector review still pending. Evidence: output/playwright/html-to-riv/overflow-axis-{composition,shortouter}-discriminators.json and overflow-axis-shortouter-{native,vector,parity}.


### Short outer axis clipping visual audit (2026-09-10)

The 72 Chrome/native comparisons in `overflow-axis-shortouter-native` now have complete audited visual coverage: 36 direct views and 36 exact full-image transfers within the run. Inspected shapes, underlined text and quadrant images at 240/390/768px. Outer clipping removes lower content while visible controls preserve it; rounded background paint remains independent of the straight x-axis clip. The vector replay has 48 audited exact compiler-input/full-PNG transfers; 24 changed text views still require direct inspection. Numeric gates pass in both profiles; this does not complete L20. Evidence: `output/playwright/html-to-riv/overflow-l20-receipt.json` and the per-run visual receipts.


### Short outer vector review completed (2026-09-10)

All 72 short-outer vector comparisons now have audited visual coverage: 12 directly inspected views, 12 exact within-run transfers and 48 exact compiler-input/full-image cross-run transfers, with no overlapping counts. Glyph-edge differences are visible in the difference sheets and pass the unchanged numeric thresholds; clipping boundaries, line placement, underlines and sibling/background layering agree with Chrome. The new `validation/audit-combined-review.py` revalidates both underlying receipts and records their set union in `overflow-axis-shortouter-vector/visual-coverage.json`. Main composition review, host compatibility expansion, clip-margin and full regression work remain open; L20 is not yet qualified.


### Main composition shape review (2026-09-10)

Native composition review now covers 90/144 views (33 direct, 57 exact within-run transfers), with the receipt independently audited. All shape variants are covered. Y-axis clips preserve horizontal escape until the outer clip trims it; x-axis clips retain vertical extent and square edges independently of rounded orange backgrounds. Green sibling occlusion/exposure and responsive orange right-cap placement agree with Chrome. Tall x cases remain placement controls, with actual vertical outer-clipping discrimination supplied by the completed shortouter corpus. Text/image composition reviews and remaining L20 qualification work are still open.


### Main composition native review complete (2026-09-10)

All144 main composition native views now have audited visual coverage (63 direct, 81 exact within-run transfers). Text reviews confirm clipping of line beginnings under X and top/bottom lines under Y, preserved underlines and correct sibling/background layering. Image quadrant boundaries and clipped horizontal bands agree with Chrome at240/390/768. Vector replay has96 audited exact compiler-input/full-PNG transfers;48 changed text views require direct review. No thresholds changed. Evidence: per-run visual-inspection.json and visual-transfer.json, summarized in overflow-l20-receipt.json. L20 remains active pending vector review, host compatibility expansion, clip-margin and full regression integration.


### Main composition vector review complete (2026-09-10)

All144 vector composition views now have complete audited coverage: 21 directly inspected, 27 exact within-run transfers and96 exact compiler-input/full-image cross-run transfers, with no overlapping counts. Review confirms glyph clipping, underlines, background corners and positioned sibling layering against Chrome at240/390/768. Glyph-edge raster differences remain visible but satisfy unchanged thresholds. The combined audit revalidated both underlying receipts. Initial42, composition144 and shortouter72 axis views per profile now have complete visual coverage. This finishes the current focused visual corpus, not L20: host compatibility expansion, clip-margin and full regression integration remain.


Axis host regression discovered (2026-09-10): new public host test reproduces missing axis clip for an unpainted, unpositioned container after rebuilding the featured probe. Earlier focused pixel passes do not qualify this configuration. Preserved request, Rive, requirements and stream: `output/playwright/html-to-riv/overflow-axis-plain-host-reproducer/`. Runtime drawable lifecycle investigation is active; do not claim general axis clipping qualified. Logs: overflow-axis-host-initial.log and overflow-axis-host-focused.log.


### Plain axis proxy lifecycle fix (2026-09-10)

The checked installer now creates missing drawable proxies for plain layouts after import. It places nested proxies using complete owner ancestry, preserving subtree boundaries. The first insertion attempt opened clips after child paint; that failing trace is retained in overflow-axis-proxy-nested-debug.log. Corrected runtime tests pass8/8, including32 new original/clone frames with repeated install, clear, reinstall and resizing; existing positioned/stacking lifecycle tests also pass. Rebuilt native host tests pass18/18, including missing axis capability and malformed manifest rejection before stream output. Eight prospective plain/nested/painted x/y fixtures are in validation/overflow-axis-plain-cases.json. Pixel/geometry qualification and parity against a newly frozen toolchain remain pending; prior corpus receipts refer to their original binaries. Logs and source hashes are in overflow-l20-receipt.json.


### Plain-container fix pixel qualification (2026-09-10)

Frozen overflow-axis-proxy-toolchain passes24/24 Chrome geometry/pixel comparisons in each profile and8/8 public native/WASM parity cases. All24 native views were directly inspected; all24 vector views transfer with exact compiler-input/full-PNG identity and an independent audit. Single/nested x/y clips match expected red extents and preserve the blue sibling outside the clipped subtree. Painted x cases retain a thin fractional background-edge difference at390 within unchanged tolerances. Public original/clone resize regression now passes1160 updates across145 scenes. Initial resize run failed only its obsolete1096 count assertion, preserved in overflow-axis-plain-resize.log; corrected count passes in overflow-axis-plain-resize-final.log. Prior focused corpora and full regression still need rerunning with the fixed runtime. L20 remains active.


### Fixed-runtime regression started (2026-09-10)

Full Rust compiler suite passes340 tests across65 result groups (overflow-axis-proxy-full-tests.log). Composition reruns pass144/144 per profile; all144 native views transfer with exact compiler-input/full-image identity from the reviewed baseline. Vector identity audit remains pending. Full native gallery is running with frozen overflow-axis-proxy-toolchain in isolated overflow-axis-proxy-full-artifacts and overflow-axis-proxy-full-native directories (session18728). Initial/shortouter profile reruns are sequentially running in session14767. These processes are not yet completion evidence.


### Fixed-runtime focused regression complete (2026-09-10)

Initial42, composition144 and shortouter72 reruns pass in both profiles with the proxy fix:258 comparisons per profile. Every compiler input and full browser/runtime PNG pair is identical to its previously reviewed baseline. All six transfer receipts were independently audited. The transfer helper now supports --combined-source, revalidating direct and cross-run evidence before transferring a combined baseline; this does not create new visual inspection claims. Full6271-check native gallery remains live in session18728, with completion and gallery audit still pending.
