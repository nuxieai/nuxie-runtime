# Wave C4 core-utility candidate

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Sources and pinned blob SHA-256 values:

- `tests/unit_tests/runtime/refcnt_test.cpp` (2 cases): `1612a98321f0f996a22acbc2d6925a0a7934599b83f200432f6450a8ec30f205`
- `tests/unit_tests/runtime/simple_array_test.cpp` (13 cases): `2a0c0a077955a502b5c05f9652bb213e5969efe1f951474f7c8dfce6737102cc`
- `tests/unit_tests/runtime/simd_test.cpp` (23 cases): `bb35a0f245e5de15b04c56fb757993d889946efe07b61ca99bbc495566d37a9c`
- `tests/unit_tests/runtime/span_test.cpp` (3 cases): `1fad526746d695c7cbf791b14b0d16f99a7cf8344241c47a13d58514f962819a`
- `tests/unit_tests/runtime/type_conversions_test.cpp` (11 cases): `ea4ce3dbae66d83151cfacc5200a4fc3b9645fa95b145b681bf0a19139f3aac8`

## Candidate verdict

Candidate for fresh independent review: **31 executable passes, zero expected-red, 21 honest pending, 52 exact identities**.

- 4 direct passes: RefCnt case 1, SimpleArray case 1, SIMD `fast_acos`, and Span case 3.
- 27 `cxx-language-only` adapted passes: RefCnt case 2, 14 primitive/fixed-array SIMD cases, Span case 2, and all 11 type-conversion cases.
- 21 pending: SimpleArray cases 2-13, SIMD cases 4-8, 10, 15, and 22, and Span case 1.

Every pass is a distinct discoverable Rust test. No denominator case is backed by an aggregate test, a test-local production algorithm, a proxy owner, raw-source text, or a shared parameter loop. The candidate changes no production behavior. The three module declarations and all new owner tests are test-only; the integration tests are also non-test artifacts only.

## Owner and adaptation adjudication

- RefCnt executes against the retained intrusive `RefCnt`/`rcp` owner. Case 2 explicitly declares the unavoidable C++ inheritance and implicit converting-constructor syntax adaptation while preserving every pointer identity, field, move, and count assertion.
- SimpleArray and Span tests are placed inside the retained owner module. Only complete, directly observable streams are claimed. The remaining SimpleArray rows stay pending because the pinned streams require allocator counters, constructor forms, nested-owner behavior, or null data-pointer observables that the retained Vec-backed owner does not expose. Capacity or `Vec::as_ptr()` substitutions were rejected as proxies.
- The 14 adapted SIMD rows contain only direct primitive or fixed-array expressions for the pinned C++ portability wrapper's complete lane contract. Rows with swizzle assignment, IEEE generic-width templates, specialized min/max/clamp/reduce/div255/mix behavior, or randomness remain pending. `fast_acos` runs owner-locally against the retained production formula and preserves the full ten-iteration Newton stream.
- Type conversions use Rust's language-native `checked_mul` and `overflowing_mul` authority. Every operand, integer width, success flag, defined output, alias-shaped action, and assertion order is preserved without recreating `checkedMul` in test code.

## Validation

- Focused non-incremental suites: 31 passed, zero failed, zero ignored (14 SIMD adaptation tests, 11 type-conversion tests, and six renderer owner-local tests under `renderer-vulkan`).
- Strict Wave C4 source/identity/name/line/status/locator audit: 52/52 accepted; direct 4, adapted 27, pending 21; pass 31, unverified 21.
- No expected-red is claimed. Five provisional SimpleArray failures were deliberately removed when the complete pinned assertion streams proved to require unavailable allocator/null-pointer observables; they are not counted as denominator evidence.
- Repository correspondence checker: passed for 157 files and 1,404 pinned `TEST_CASE` declarations.
- Correspondence checker unit suite: 24/24 passed.
- Default non-test release LLVM IR builds passed for `nuxie-binary`, `nuxie-runtime`, and `nuxie-renderer`; current IR contains no `wave_c4_` test symbol or expected-red string.
- Pinned source hashes, JSON parse, exact Rust locators, scoped formatting, and `git diff --check` passed.

This is candidate evidence only and does not self-accept Wave C4.
