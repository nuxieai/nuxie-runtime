# Wave B5 independent semantic review

Reviewed candidate: `d12cabb53277613cb70fb98c570eefc040f193f1`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **REJECTED — 30/31 semantically accepted**

## Acceptance rule

All 31 rows were read against the pinned C++ fixture, action order, live owner,
and assertion semantics. Passing output alone was insufficient. An executable
expected-red had to preserve the exact setup and assertion prefix through the
first concrete missing or divergent runtime seam. Owner scans, assertion
reordering, and other narrowing proxies were rejected even when the eventual
failure matched a real renderer defect.

## Exact census

| upstream file | cases | accepted pass | accepted expected-red | rejected |
|---|---:|---:|---:|---:|
| `hittest_test.cpp` | 21 | 13 | 7 | 1 |
| `ik_constraint_test.cpp` | 1 | 1 | 0 | 0 |
| `ik_test.cpp` | 2 | 2 | 0 | 0 |
| `image_asset_test.cpp` | 2 | 2 | 0 | 0 |
| `image_decoders_test.cpp` | 5 | 3 | 2 | 0 |
| **total** | **31** | **21** | **9** | **1** |

The committed shard mechanically declares 21 pass and 10 expected-red. One of
those ten red rows is rejected as evidence, leaving 30 accepted rows. Of the
accepted rows, 28 are direct and two are `rust-safety` adaptations for stable
ImageAsset identity.

## Rejected row

`hittest_test.cpp#4`, **hit test on opaque nested artboard**, reaches the real
301-pixel boundary divergence, but its setup and assertion prefix are not an
exact port.

Pinned C++ requires `nestedAnimations()[0]` to be a `NestedStateMachine`,
retains that exact occurrence, advances the artboard and outer state machine,
and only then asserts the nested `bool-target` value is false. The Rust helper
instead searches all nested animations with `find_map` and accepts the first
state machine anywhere in the collection. It can therefore pass when index
zero is the wrong owner. The Rust test also asserts `bool-target == false`
before `update_components` and the outer state-machine advance, then omits the
pinned post-initialization nested-value assertion.

Those differences can hide both incorrect nested-animation ordering and an
incorrect initialization mutation. The later failure at
`second-gray-toggle` after `pointer_down(301, 50)` is genuine, but it occurs
after a narrowed owner selection and reordered assertion prefix. This row must
select nested animation zero exactly and assert the nested boolean after the
pinned initialization sequence before its boundary red can be accepted.

## Accepted semantic groups

### Remaining hittest cases

The six other direct hittest ports preserve their HitTester geometry or live
fixture/artboard/state-machine/input/event/animation owners and exact action
order. The `TESTING` early-out row proves the actual four-element hit-component
owner collection and stops at the first absent retained `earlyOutCount`
observable. Its failure is an actual missing owner seam, not a fabricated
counter.

All 14 Silver rows resolve the exact pinned provenance case and execute the
complete manifest action stream before comparing fresh Rust SRIV with the
frozen C++ stream. Eight streams match. The six expected-red streams fail at
their documented first operation or signed-zero difference after executing
the full action stream.

### IK

The constraint case preserves both named Bone owners, the Skin owner, the
first IKConstraint relation, and both graph-order assertions. The two IK cases
preserve the exact named shapes, bones, target, animation, both dependent
relations, target values, all four matrices, the upstream `0.0001` per-field
tolerance, and the complete 1,000-iteration two-pose loop.

### ImageAsset identity adaptations

Both adaptations replace only raw C++ pointer-address comparison with stable
retained FileAsset global identity. They preserve the authored Image-to-
ImageAsset relation, exact shared/distinct identity assertions, payload byte
sizes, update, decode, and draw behavior. The out-of-band case attaches the
exact `walle-370.png` and `eve-317.png` payloads to their authored asset IDs
before drawing. No graph-membership proxy replaces the identity question.

### Platform-conditional image decoders

`validate_encoded_image` performs a complete decode and validates the decoded
RGBA buffer; it is not the header-only preflight API. The valid PNG, JPEG, and
WebP rows preserve exact input lengths, dimensions, and decoded-buffer size
rules.

The bad JPEG row preserves the source-only non-Apple expectation as an
executable expected-red. Rust rejects it in the platform-independent admission
guard before a backend allocation, so the red does not fabricate a macOS
decoder result. The bad PNG body preserves both pinned branches: Apple expects
the oversized black bitmap and is red at the same admission guard on the
review host, while non-Apple requires the null result. The current macOS
classification is therefore accepted; the branch bodies themselves remain
source-exact.

## Mechanical and execution gates

- pinned upstream HEAD: exact;
- all 31 identities, ordinals, source lines, names, evidence lines, symbols,
  classifications, ignore attributes, and reasons: exact;
- shard census: 29 direct / two adapted, 21 pass / ten expected-red;
- all 21 passing rows executed successfully;
- all ten expected-red rows were forced individually; each selected one
  ignored test and failed inside its named body at the documented owner,
  decoder, or SRIV boundary;
- repository correspondence checker: 157 files and 1,404 pinned `TEST_CASE`s,
  green;
- correspondence checker unit suite: 24/24 green;
- non-test LLVM IR contains no Wave B5 test module or symbol;
- JSON parsing and scoped `git diff --check`: green.

Execution success does not promote the narrowed nested-artboard row. Wave B5
remains rejected until `hittest_test.cpp#4` restores the exact index-zero owner
selection and post-initialization nested boolean assertion.
