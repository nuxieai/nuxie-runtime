# Wave C3 micro final independent rereview

Status: **REJECTED; TWO UNPINNED ASSERTIONS MUST BE REMOVED**

Original candidate: `fae9e184300a8b0fd49ea75787c35de3f81fa296`

Owner/proxy rejection: `82a8d8b39`

Evidence-census correction: `6886bc0ecfe497e988a87a122bd97be11306423b`

Schema rejection: `c2ddbc3d3`

Ledger-schema correction reviewed: `b40f87e119ca1198ee998b9c7e80ac043f1c410a`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Verdict

The ledger-schema correction is mechanically correct, and the prior
owner/proxy rejection remains fully corrected. The frozen declaration is the
exact 23-case denominator: lite RTTI 1, malformed import 2, math 18, and Node
2. It records nine structured adapted passes, one direct pass, and 13 strict
pending/unverified rows. All ten declared executable tests pass.

The combined candidate is nevertheless rejected under the campaign's literal
assertion-stream rule. Two of the ten executable bodies add a Rust assertion
that has no corresponding assertion in its pinned C++ case. The current exact
semantic result is therefore eight accepted executable rows, two rejected
executable rows, and 13 honest pending rows. No expected-red is involved.

## Blocking assertion-stream defects

### Lite RTTI case 1

`crates/nuxie-runtime/tests/upstream_language_wave_c3.rs:91` adds:

```rust
assert_ne!(TypeId::of::<RttiF>(), TypeId::of::<RttiG>());
```

It appears after the adapted constructor/type/field checks and before the
`Rc<dyn Any>` construction and the two shared-owner downcasts. The pinned
`lite_rtti_test.cpp` body declares `F`, `G`, and `H`, creates the `G`-backed
`rcp<F>`, casts it to `G` and `H`, and asserts only that the `G` cast is
non-null and the `H` cast is null. It never independently asserts that the
type ids of `F` and `G` differ.

The extra inequality is plausible and green, but it adds a parity requirement
that the pinned case does not make. Delete only this assertion. Preserve the
`F`/`G`/`H` declarations, `Rc` construction, both downcasts, and the pinned
success/failure assertions in their current order.

### Malformed-import case 1

`crates/nuxie/tests/upstream_malformed_import_wave_c3.rs:32` adds:

```rust
assert!(full_file.is_some(), "the full final prefix imports");
```

It appears after the every-prefix import loop. Pinned malformed-import case 1
does not require the full prefix to import successfully. For every prefix,
including the full prefix, it branches on the actual `ImportResult` and asserts
only the result/file ownership invariant: success has a non-null file and any
other result has a null file. Full-file success is asserted separately by
pinned case 2.

Rust's `Result<File>` adaptation makes the case-1 result/file mismatch
unrepresentable, but it does not authorize adding a mandatory-success
assertion. Delete only the post-loop assertion. Preserve the exact prefix
range, import calls, match/drop lifecycle, and case-2 full-file/artboard
assertion unchanged.

## Frozen topology and authority

- The executable identities remain lite RTTI #1, malformed import #1-#2, and
  math #1, #2, #9-#12, and #14.
- The 13 pending identities remain math #3-#8, #13, #15-#18, and Node #1-#2.
  Every pending row is `unverified`, has empty evidence, and has no note,
  adaptation, locator, placeholder, or synthetic red.
- The rejected Wave C3 `MixedInteger` comparison implementation, round-up
  closure/test, duplicated `count_ones` fallback evidence, and Node arena proxy
  tests remain absent. No replacement proxy or production implementation was
  introduced.
- Math #14 alone is direct and calls the production `positive_mod` owner. The
  other nine declared executable rows carry case-specific structured
  `rust-safety` or `cxx-language-only` adaptation metadata.
- Apart from the two unpinned assertions above, the retained executable bodies
  preserve their pinned assertions, literals, action order, and adjudicated
  owner/adaptation authority.

## Gates

- Focused non-incremental sweep: seven math, one lite-RTTI, and two
  malformed-import tests passed; zero failed or ignored.
- Strict Wave C3 shard: 23 identities and ten evidence locators resolved; nine
  adapted, one direct, 13 pending; ten pass and 13 unverified.
- Strict schema shape: all nine adapted rows have non-empty structured
  adaptation metadata; all 13 pending rows contain only their identity,
  unverified disposition, and empty evidence.
- Repository correspondence checker: 157 files / 1,404 pinned cases, green.
- Correspondence-checker unit suite: 24/24 green.
- Pinned checkout and all four source SHA-256 identities: green; ledger JSON
  parsing and candidate/correction diff checks are green.
- Rejected helper/test symbols and Node proxy evidence are absent from the Wave
  C3 source scope.
- Default release `nuxie-runtime` LLVM IR contains no Wave C3 test, rejected
  helper, round-up, fallback, or Node-proxy test symbol.

Every relied-on Cargo invocation disabled incremental compilation. This
review changes no candidate test, ledger row, fixture, production code, or
runtime behavior.
