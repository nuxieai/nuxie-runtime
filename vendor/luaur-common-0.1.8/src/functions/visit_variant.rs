//! @interface-stub
/// C++ `template<class Visitor, typename... Ts> auto visit(Visitor&& vis,
/// const Variant<Ts...>& var)` (Variant.h:237). The Rust Variant port is a
/// fixed-arity enum family (Variant1..Variant7); callers are translated to a
/// `match` over the enum instead of calling this. This node exists as the
/// pinned overload contract.
pub fn visit<Visitor, V, R>(_vis: Visitor, _var: &V) -> R {
    unreachable!(
        "C++ Variant visit() template; Rust uses match over the Variant enum — no call site"
    )
}
