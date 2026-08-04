# Rung 3 no-op rows

Range: `86d2a9dc..f1f121dc` (official Luau 0.726 → 0.727).

| row | disposition | verification |
|---|---|---|
| A18 `Parser::parseAttributedFunction` | No port required. | The C++ hunk only reformats one `reportExprError` call. The existing Rust twin, `vendor/luaur-ast-0.1.8/src/methods/parser_parse_attributed_function.rs`, already performs the same call and has no behavioral delta. The inventory's `NONE-untranslated` path claim is stale: the twin exists after rung 2. |
| C01 `DenseHashSet` pointer-key constructor | No port required. | Rust has no overload/SFINAE split. `DenseHashSet::new(empty_key)` already accepts pointer keys, and its existing `Default` implementation obtains the null pointer through `DenseDefault` for `*mut T`/`*const T`, covering the C++ defaulted `nullptr` convenience. |
| C02 `DenseHashSet` non-pointer constructor | No port required. | The C++ change only constrains overload resolution with `std::enable_if_t`. Rust's single generic `DenseHashSet::new(empty_key)` is type-safe for pointer and non-pointer keys without an overload ambiguity. |
| C03 `DenseHashMap` pointer-key constructor | No port required. | `DenseHashMap::new(empty_key)` already accepts raw-pointer keys and all translated construction sites supply the empty-key sentinel explicitly. The new C++ default argument is overload/API convenience; no translated Rust call depends on a zero-argument map constructor. |
| C04 `DenseHashMap` non-pointer constructor | No port required. | The C++ SFINAE constraint only prevents constructor ambiguity. Rust's single generic constructor already expresses the behavior without overload resolution. |

Verified files:

- `vendor/luaur-common-0.1.8/src/records/dense_hash_set.rs`
- `vendor/luaur-common-0.1.8/src/records/dense_hash_map.rs`
- `vendor/luaur-common-0.1.8/src/records/dense_hash_table.rs`
- `vendor/luaur-ast-0.1.8/src/methods/parser_parse_attributed_function.rs`
