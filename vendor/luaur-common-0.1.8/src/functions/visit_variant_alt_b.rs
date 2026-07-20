//! @interface-stub
/// C++ `template<class Visitor, typename... Ts> auto visit(Visitor&& vis,
/// Variant<Ts...>& var)` (Variant.h:265, the mutable overload). The Rust
/// Variant port is a fixed-arity enum family (Variant1..Variant7); callers
/// are translated to a `match` over the enum instead of calling this. This
/// node exists as the pinned overload contract.
pub fn visit<Visitor, V, R>(_vis: Visitor, _var: &mut V) -> R {
    unreachable!("C++ Variant visit() mutable template overload; Rust uses match over the Variant enum — no call site")
}

#[allow(unused_imports)]
pub use visit as visit_mut;
