//! AST visitor dispatch — the faithful Rust form of Luau's virtual
//! `AstNode::visit(AstVisitor*)` traversal (`Ast/src/Ast.cpp`).
//!
//! In C++ each concrete node overrides `visit(AstVisitor*)`: it calls the typed
//! `visitor->visit(this)` (compile-time overload on `this`'s static type) and,
//! if that returns `true`, recurses into its children with `child->visit(v)` —
//! a *virtual* call dispatched on the child's dynamic type.
//!
//! Rust has no vtable here (nodes are thin `*mut AstExpr` etc. in the arena), so
//! the per-node override becomes `impl AstVisitable for X`, and the virtual
//! recursion becomes a `class_index` match in the `*_visit` dispatch functions
//! below — the central analog of the C++ vtable. A node's `visit` body calls
//! `crate::visit::ast_expr_visit(self.child, v)` for each child pointer (and
//! loops over `AstArray` children), never `child.visit(v)` directly, because the
//! static type of `self.child` is only the base.

use crate::records::ast_visitor::AstVisitor;

// `block->visit(visitor)` where the static type is already `AstStatBlock` —
// C++ calls the override directly (no virtual dispatch needed), so route to
// the node's own `visit` impl rather than the class-index dispatcher.
pub use crate::methods::ast_stat_block_visit::ast_stat_block_visit;

/// C++ `AstX::visit(AstVisitor*)` override. Implemented once per concrete node
/// by that node's `visit` method item. `&self` is enough — `visit` never mutates
/// the node (it only feeds `self` to the visitor and recurses).
pub trait AstVisitable {
    fn visit(&self, visitor: &mut dyn AstVisitor);
}

/// `expr->visit(visitor)` where `expr` is a base `*mut AstExpr` — dispatch to the
/// concrete override by RTTI class index.
///
/// # Safety
/// `expr` must be null or point to a live `AstExpr`-prefixed node.
pub unsafe fn ast_expr_visit(
    expr: *mut crate::records::ast_expr::AstExpr,
    visitor: &mut dyn AstVisitor,
) {
    dispatch_node((expr as *mut crate::records::ast_node::AstNode), visitor);
}

/// `stat->visit(visitor)` for a base `*mut AstStat`.
///
/// # Safety
/// `stat` must be null or point to a live `AstStat`-prefixed node.
pub unsafe fn ast_stat_visit(
    stat: *mut crate::records::ast_stat::AstStat,
    visitor: &mut dyn AstVisitor,
) {
    dispatch_node((stat as *mut crate::records::ast_node::AstNode), visitor);
}

/// `ty->visit(visitor)` for a base `*mut AstType`.
///
/// # Safety
/// `ty` must be null or point to a live `AstType`-prefixed node.
pub unsafe fn ast_type_visit(
    ty: *mut crate::records::ast_type::AstType,
    visitor: &mut dyn AstVisitor,
) {
    dispatch_node((ty as *mut crate::records::ast_node::AstNode), visitor);
}

/// `pack->visit(visitor)` for a base `*mut AstTypePack`.
///
/// # Safety
/// `pack` must be null or point to a live `AstTypePack`-prefixed node.
pub unsafe fn ast_type_pack_visit(
    pack: *mut crate::records::ast_type_pack::AstTypePack,
    visitor: &mut dyn AstVisitor,
) {
    dispatch_node((pack as *mut crate::records::ast_node::AstNode), visitor);
}

/// `node->visit(visitor)` for any base `*mut AstNode`.
///
/// # Safety
/// `node` must be null or point to a live AST node.
pub unsafe fn ast_node_visit(
    node: *mut crate::records::ast_node::AstNode,
    visitor: &mut dyn AstVisitor,
) {
    dispatch_node(node, visitor);
}

// The central class-index dispatcher — the analog of the C++ vtable. One arm
// per concrete node type; each downcast is sound for the same reason as
// `ast_node_as` (standard-layout, base at offset 0).
//
// # Safety
// `node` must be null or point to a live AST node.
pub unsafe fn dispatch_node(
    node: *mut crate::records::ast_node::AstNode,
    visitor: &mut dyn AstVisitor,
) {
    use crate::rtti::AstNodeClass;
    use crate::visit::AstVisitable;
    if node.is_null() {
        return;
    }
    match (*node).class_index {
        x if x == crate::records::ast_attr::AstAttr::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_attr::AstAttr)).visit(visitor)
        }
        x if x == crate::records::ast_expr_binary::AstExprBinary::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_binary::AstExprBinary)).visit(visitor)
        }
        x if x == crate::records::ast_expr_call::AstExprCall::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_call::AstExprCall)).visit(visitor)
        }
        x if x == crate::records::ast_expr_constant_bool::AstExprConstantBool::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_constant_bool::AstExprConstantBool)).visit(visitor)
        }
        x if x == crate::records::ast_expr_constant_integer::AstExprConstantInteger::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_constant_integer::AstExprConstantInteger)).visit(visitor)
        }
        x if x == crate::records::ast_expr_constant_nil::AstExprConstantNil::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_constant_nil::AstExprConstantNil)).visit(visitor)
        }
        x if x == crate::records::ast_expr_constant_number::AstExprConstantNumber::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_constant_number::AstExprConstantNumber)).visit(visitor)
        }
        x if x == crate::records::ast_expr_constant_string::AstExprConstantString::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_constant_string::AstExprConstantString)).visit(visitor)
        }
        x if x == crate::records::ast_expr_error::AstExprError::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_error::AstExprError)).visit(visitor)
        }
        x if x == crate::records::ast_expr_function::AstExprFunction::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_function::AstExprFunction)).visit(visitor)
        }
        x if x == crate::records::ast_expr_global::AstExprGlobal::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_global::AstExprGlobal)).visit(visitor)
        }
        x if x == crate::records::ast_expr_group::AstExprGroup::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_group::AstExprGroup)).visit(visitor)
        }
        x if x == crate::records::ast_expr_if_else::AstExprIfElse::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_if_else::AstExprIfElse)).visit(visitor)
        }
        x if x == crate::records::ast_expr_index_expr::AstExprIndexExpr::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_index_expr::AstExprIndexExpr)).visit(visitor)
        }
        x if x == crate::records::ast_expr_index_name::AstExprIndexName::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_index_name::AstExprIndexName)).visit(visitor)
        }
        x if x == crate::records::ast_expr_instantiate::AstExprInstantiate::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_instantiate::AstExprInstantiate)).visit(visitor)
        }
        x if x == crate::records::ast_expr_interp_string::AstExprInterpString::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_interp_string::AstExprInterpString)).visit(visitor)
        }
        x if x == crate::records::ast_expr_local::AstExprLocal::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_local::AstExprLocal)).visit(visitor)
        }
        x if x == crate::records::ast_expr_table::AstExprTable::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_table::AstExprTable)).visit(visitor)
        }
        x if x == crate::records::ast_expr_type_assertion::AstExprTypeAssertion::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_type_assertion::AstExprTypeAssertion)).visit(visitor)
        }
        x if x == crate::records::ast_expr_unary::AstExprUnary::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_unary::AstExprUnary)).visit(visitor)
        }
        x if x == crate::records::ast_expr_varargs::AstExprVarargs::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_expr_varargs::AstExprVarargs)).visit(visitor)
        }
        x if x == crate::records::ast_generic_type::AstGenericType::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_generic_type::AstGenericType)).visit(visitor)
        }
        x if x == crate::records::ast_generic_type_pack::AstGenericTypePack::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_generic_type_pack::AstGenericTypePack)).visit(visitor)
        }
        x if x == crate::records::ast_stat_assign::AstStatAssign::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_assign::AstStatAssign)).visit(visitor)
        }
        x if x == crate::records::ast_stat_block::AstStatBlock::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_block::AstStatBlock)).visit(visitor)
        }
        x if x == crate::records::ast_stat_break::AstStatBreak::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_break::AstStatBreak)).visit(visitor)
        }
        x if x == crate::records::ast_stat_class::AstStatClass::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_class::AstStatClass)).visit(visitor)
        }
        x if x == crate::records::ast_stat_compound_assign::AstStatCompoundAssign::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_compound_assign::AstStatCompoundAssign)).visit(visitor)
        }
        x if x == crate::records::ast_stat_continue::AstStatContinue::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_continue::AstStatContinue)).visit(visitor)
        }
        x if x == crate::records::ast_stat_declare_extern_type::AstStatDeclareExternType::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_declare_extern_type::AstStatDeclareExternType)).visit(visitor)
        }
        x if x == crate::records::ast_stat_declare_function::AstStatDeclareFunction::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_declare_function::AstStatDeclareFunction)).visit(visitor)
        }
        x if x == crate::records::ast_stat_declare_global::AstStatDeclareGlobal::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_declare_global::AstStatDeclareGlobal)).visit(visitor)
        }
        x if x == crate::records::ast_stat_error::AstStatError::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_error::AstStatError)).visit(visitor)
        }
        x if x == crate::records::ast_stat_expr::AstStatExpr::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_expr::AstStatExpr)).visit(visitor)
        }
        x if x == crate::records::ast_stat_for::AstStatFor::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_for::AstStatFor)).visit(visitor)
        }
        x if x == crate::records::ast_stat_for_in::AstStatForIn::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_for_in::AstStatForIn)).visit(visitor)
        }
        x if x == crate::records::ast_stat_function::AstStatFunction::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_function::AstStatFunction)).visit(visitor)
        }
        x if x == crate::records::ast_stat_if::AstStatIf::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_if::AstStatIf)).visit(visitor)
        }
        x if x == crate::records::ast_stat_local::AstStatLocal::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_local::AstStatLocal)).visit(visitor)
        }
        x if x == crate::records::ast_stat_local_function::AstStatLocalFunction::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_local_function::AstStatLocalFunction)).visit(visitor)
        }
        x if x == crate::records::ast_stat_repeat::AstStatRepeat::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_repeat::AstStatRepeat)).visit(visitor)
        }
        x if x == crate::records::ast_stat_return::AstStatReturn::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_return::AstStatReturn)).visit(visitor)
        }
        x if x == crate::records::ast_stat_type_alias::AstStatTypeAlias::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_type_alias::AstStatTypeAlias)).visit(visitor)
        }
        x if x == crate::records::ast_stat_type_function::AstStatTypeFunction::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_type_function::AstStatTypeFunction)).visit(visitor)
        }
        x if x == crate::records::ast_stat_while::AstStatWhile::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_stat_while::AstStatWhile)).visit(visitor)
        }
        x if x == crate::records::ast_type_error::AstTypeError::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_error::AstTypeError)).visit(visitor)
        }
        x if x == crate::records::ast_type_function::AstTypeFunction::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_function::AstTypeFunction)).visit(visitor)
        }
        x if x == crate::records::ast_type_group::AstTypeGroup::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_group::AstTypeGroup)).visit(visitor)
        }
        x if x == crate::records::ast_type_intersection::AstTypeIntersection::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_intersection::AstTypeIntersection)).visit(visitor)
        }
        x if x == crate::records::ast_type_optional::AstTypeOptional::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_optional::AstTypeOptional)).visit(visitor)
        }
        x if x == crate::records::ast_type_pack_explicit::AstTypePackExplicit::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_pack_explicit::AstTypePackExplicit)).visit(visitor)
        }
        x if x == crate::records::ast_type_pack_generic::AstTypePackGeneric::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_pack_generic::AstTypePackGeneric)).visit(visitor)
        }
        x if x == crate::records::ast_type_pack_variadic::AstTypePackVariadic::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_pack_variadic::AstTypePackVariadic)).visit(visitor)
        }
        x if x == crate::records::ast_type_reference::AstTypeReference::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_reference::AstTypeReference)).visit(visitor)
        }
        x if x == crate::records::ast_type_singleton_bool::AstTypeSingletonBool::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_singleton_bool::AstTypeSingletonBool)).visit(visitor)
        }
        x if x == crate::records::ast_type_singleton_string::AstTypeSingletonString::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_singleton_string::AstTypeSingletonString)).visit(visitor)
        }
        x if x == crate::records::ast_type_table::AstTypeTable::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_table::AstTypeTable)).visit(visitor)
        }
        x if x == crate::records::ast_type_typeof::AstTypeTypeof::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_typeof::AstTypeTypeof)).visit(visitor)
        }
        x if x == crate::records::ast_type_union::AstTypeUnion::CLASS_INDEX => {
            (&*(node as *const crate::records::ast_type_union::AstTypeUnion)).visit(visitor)
        }
        _ => {
            // C++ cannot reach here: every concrete AstNode subclass overrides
            // visit. An unknown class index means arena corruption.
            panic!("dispatch_node: unknown AST class index {}", (*node).class_index);
        }
    }
}
