use crate::records::ast_visitor::AstVisitor;

/// In Rust, a virtual destructor is represented by the `Drop` trait or is implicitly
/// handled by the trait object's vtable. Since `AstVisitor` is a trait, we do not
/// need an explicit destructor method. This impl block is provided for compatibility
/// with the translation graph.
impl dyn AstVisitor {
    // No-op: Rust handles trait object destruction via Drop.
}
