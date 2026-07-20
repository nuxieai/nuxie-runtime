//! Faithful port of Luau's `Variant<Ts...>` — a `std::variant`-like tagged union.
//! Reference: `luau/Common/include/Luau/Variant.h`. Oracle:
//! `luau/tests/Variant.test.cpp` + `/tmp/variant_proto.rs` (DefaultCtor, Create,
//! Emplace, NonPOD copy, Equality, Visit — all pass).
//!
//! Rust has no variadic generics, so the C++ variadic `Variant<Ts...>` becomes a
//! fixed-arity enum family `Variant1<T0> .. Variant7<..>` (the arities Luau
//! actually instantiates; max is 7). A Rust `enum` *is* a tagged union, so this
//! is safe (no type-erased storage, no fn-pointer dispatch tables) and idiomatic.
//!
//! Mechanical mapping for callers (e.g. the eventual Analysis port):
//! - `Variant<A, B, C>`            -> `Variant3<A, B, C>`
//! - `Variant<A,B> x = a;`         -> `Variant2::V0(a)` (no blanket `From<Ti>` —
//!   Rust coherence forbids it since `T0` could equal `T1`; construct the variant
//!   directly at the position the type occupies)
//! - `v.get_if<B>()` (B is pos 1)  -> `v.get_if_1()` / `v.get_if_1_mut()`
//! - `v.emplace<B>(args)`          -> `v = Variant2::V1(B::from(args))`
//! - `v.index()`                   -> `v.index()`
//! - `visit(overloaded{...}, v)`   -> `match v { Variant3::V0(x) => …, … }`
//!
//! `==` (C++ `operator==`) and `Default` (C++ `Variant()` -> first alternative)
//! come from the derives + the generated first-alternative `Default` impl;
//! `valueless_by_exception()` is always `false`.

/// Generates one `VariantN` enum plus its `index`/`get_if_*`/`Default` API.
macro_rules! define_variant {
    (
        $name:ident < $t0:ident $(, $t:ident)* >
        = $v0:ident($g0:ident, $g0m:ident)
        $(, $idx:literal : $v:ident < $ty:ident > ($g:ident, $gm:ident) )*
    ) => {
        #[allow(clippy::large_enum_variant)]
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum $name<$t0 $(, $t)*> {
            $v0($t0),
            $( $v($ty), )*
        }

        impl<$t0 $(, $t)*> $name<$t0 $(, $t)*> {
            /// `index()` / `typeId` — the active alternative's position.
            pub fn index(&self) -> usize {
                match self {
                    Self::$v0(_) => 0,
                    $( Self::$v(_) => $idx, )*
                }
            }

            /// Always `false` (this port has no valueless state). Matches the C++
            /// `valueless_by_exception`.
            pub fn valueless_by_exception(&self) -> bool {
                false
            }

            pub fn $g0(&self) -> ::core::option::Option<&$t0> {
                match self {
                    Self::$v0(x) => ::core::option::Option::Some(x),
                    #[allow(unreachable_patterns)]
                    _ => ::core::option::Option::None,
                }
            }
            pub fn $g0m(&mut self) -> ::core::option::Option<&mut $t0> {
                match self {
                    Self::$v0(x) => ::core::option::Option::Some(x),
                    #[allow(unreachable_patterns)]
                    _ => ::core::option::Option::None,
                }
            }
            $(
                pub fn $g(&self) -> ::core::option::Option<&$ty> {
                    match self {
                        Self::$v(x) => ::core::option::Option::Some(x),
                        #[allow(unreachable_patterns)]
                        _ => ::core::option::Option::None,
                    }
                }
                pub fn $gm(&mut self) -> ::core::option::Option<&mut $ty> {
                    match self {
                        Self::$v(x) => ::core::option::Option::Some(x),
                        #[allow(unreachable_patterns)]
                        _ => ::core::option::Option::None,
                    }
                }
            )*
        }

        // C++ `Variant()` default-constructs the first alternative.
        impl<$t0: ::core::default::Default $(, $t)*> ::core::default::Default for $name<$t0 $(, $t)*> {
            fn default() -> Self {
                Self::$v0(<$t0 as ::core::default::Default>::default())
            }
        }
    };
}

define_variant!(Variant1<T0> = V0(get_if_0, get_if_0_mut));
define_variant!(
    Variant2<T0, T1> = V0(get_if_0, get_if_0_mut),
    1: V1<T1>(get_if_1, get_if_1_mut)
);
define_variant!(
    Variant3<T0, T1, T2> = V0(get_if_0, get_if_0_mut),
    1: V1<T1>(get_if_1, get_if_1_mut),
    2: V2<T2>(get_if_2, get_if_2_mut)
);
define_variant!(
    Variant4<T0, T1, T2, T3> = V0(get_if_0, get_if_0_mut),
    1: V1<T1>(get_if_1, get_if_1_mut),
    2: V2<T2>(get_if_2, get_if_2_mut),
    3: V3<T3>(get_if_3, get_if_3_mut)
);
define_variant!(
    Variant5<T0, T1, T2, T3, T4> = V0(get_if_0, get_if_0_mut),
    1: V1<T1>(get_if_1, get_if_1_mut),
    2: V2<T2>(get_if_2, get_if_2_mut),
    3: V3<T3>(get_if_3, get_if_3_mut),
    4: V4<T4>(get_if_4, get_if_4_mut)
);
define_variant!(
    Variant6<T0, T1, T2, T3, T4, T5> = V0(get_if_0, get_if_0_mut),
    1: V1<T1>(get_if_1, get_if_1_mut),
    2: V2<T2>(get_if_2, get_if_2_mut),
    3: V3<T3>(get_if_3, get_if_3_mut),
    4: V4<T4>(get_if_4, get_if_4_mut),
    5: V5<T5>(get_if_5, get_if_5_mut)
);
define_variant!(
    Variant7<T0, T1, T2, T3, T4, T5, T6> = V0(get_if_0, get_if_0_mut),
    1: V1<T1>(get_if_1, get_if_1_mut),
    2: V2<T2>(get_if_2, get_if_2_mut),
    3: V3<T3>(get_if_3, get_if_3_mut),
    4: V4<T4>(get_if_4, get_if_4_mut),
    5: V5<T5>(get_if_5, get_if_5_mut),
    6: V6<T6>(get_if_6, get_if_6_mut)
);

#[cfg(test)]
mod tests {
    use super::{Variant2, Variant3};
    use alloc::string::{String, ToString};

    // Mirrors luau/tests/Variant.test.cpp (DefaultCtor / Create / Emplace /
    // NonPOD / Equality / Visit).
    #[test]
    fn variant_behavior() {
        // DefaultCtor: first alternative, default value.
        let v: Variant2<i32, String> = Variant2::default();
        assert_eq!(v.get_if_0(), Some(&0));
        assert!(v.get_if_1().is_none());
        assert_eq!(v.index(), 0);
        assert!(!v.valueless_by_exception());

        // Create + get_if by position.
        let v1: Variant2<i32, String> = Variant2::V1("hi".to_string());
        assert_eq!(v1.get_if_1().map(String::as_str), Some("hi"));
        assert_eq!(v1.index(), 1);

        // Emplace == reassign; NonPOD copy via Clone.
        let mut m: Variant2<i32, String> = Variant2::V0(5);
        m = Variant2::V1("x".to_string());
        let mc = m.clone();
        assert_eq!(m, mc);

        // Equality: same variant+value; default == V0(0).
        let a: Variant2<i32, String> = Variant2::V0(0);
        assert_eq!(a, Variant2::<i32, String>::default());
        assert_ne!(v1, Variant2::V1("me".to_string()));
        assert_ne!(v1, Variant2::V0(1));

        // Visit -> match (arity 3).
        let t: Variant3<i32, bool, String> = Variant3::V2("z".to_string());
        let rendered = match &t {
            Variant3::V0(n) => n.to_string(),
            Variant3::V1(b) => b.to_string(),
            Variant3::V2(s) => s.clone(),
        };
        assert_eq!(rendered, "z");
    }
}
