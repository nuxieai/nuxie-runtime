//! C++ `template<class T, typename... Ts> T* get_if(Variant<Ts...>* var)` —
//! the mutable free-function overload (Variant.h:220). Callers use the const,
//! trait-based `get_if` (`functions/get_if_variant.rs`); this mutable overload
//! has no Rust call site. This node exists as the pinned overload contract.
pub fn get_if() {
    unreachable!("C++ Variant mutable free get_if(Variant*) overload; Rust uses the const trait-based get_if — no call site")
}
