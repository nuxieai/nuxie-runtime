//! `AstVisitor::visit(<node>*)` — the base visitor's default override.
//!
//! Provided by the corresponding `AstVisitor` trait default method in
//! `crate::records::ast_visitor`, where the whole virtual-dispatch delegation
//! chain (`visit_expr` -> `visit_node`, etc.) lives as one coherent unit. This
//! per-node graph item carries no separate definition (a standalone `impl dyn
//! AstVisitor` would just duplicate the trait default and collide).
