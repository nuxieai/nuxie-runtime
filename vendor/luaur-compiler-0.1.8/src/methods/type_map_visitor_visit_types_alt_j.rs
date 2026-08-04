use crate::functions::is_matching_global_member::is_matching_global_member;
use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_ast::records::ast_expr_index_name::AstExprIndexName;
use luaur_ast::records::ast_type::AstType;
use luaur_ast::records::ast_type_table::AstTypeTable;
use luaur_ast::rtti::{ast_node_as, ast_node_is};
use luaur_common::enums::luau_bytecode_type::{
    LuauBytecodeType, LBC_TYPE_ANY, LBC_TYPE_BOOLEAN, LBC_TYPE_INTEGER, LBC_TYPE_NUMBER,
    LBC_TYPE_STRING, LBC_TYPE_VECTOR,
};

impl TypeMapVisitor<'_> {
    pub fn visit_ast_expr_index_name(&mut self, node: *mut AstExprIndexName) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let node_ref = &*node;

            luaur_ast::visit::ast_expr_visit(node_ref.expr, self);

            if let Some(&type_ptr) = self.resolved_exprs.find(&node_ref.expr) {
                let table_ty_ptr = ast_node_as::<AstTypeTable>(
                    type_ptr as *mut luaur_ast::records::ast_node::AstNode,
                );

                if !table_ty_ptr.is_null() {
                    let table_ty = &*table_ty_ptr;
                    for i in 0..table_ty.props.size {
                        let prop = &*table_ty.props.data.add(i);
                        if prop.name.value == node_ref.index.value {
                            self.record_resolved_type_ast_expr_ast_type(
                                node as *mut _,
                                prop.r#type,
                            );
                            return false;
                        }
                    }
                }
            }

            if let Some(&type_bc) = self.expr_types.find(&node_ref.expr) {
                if type_bc == LBC_TYPE_VECTOR {
                    let index_ptr = node_ref.index.value;

                    let is_xyz = core::ffi::CStr::from_ptr(index_ptr).to_bytes() == b"X"
                        || core::ffi::CStr::from_ptr(index_ptr).to_bytes() == b"Y"
                        || core::ffi::CStr::from_ptr(index_ptr).to_bytes() == b"Z"
                        || core::ffi::CStr::from_ptr(index_ptr).to_bytes() == b"x"
                        || core::ffi::CStr::from_ptr(index_ptr).to_bytes() == b"y"
                        || core::ffi::CStr::from_ptr(index_ptr).to_bytes() == b"z";

                    if is_xyz {
                        self.record_resolved_type_ast_expr_ast_type(
                            node as *mut _,
                            &self.builtin_types.number_type as *const _ as *const AstType,
                        );
                        return false;
                    }
                }
            }

            if is_matching_global_member(self.globals, node, c"vector".as_ptr(), c"zero".as_ptr())
                || is_matching_global_member(
                    self.globals,
                    node,
                    c"vector".as_ptr(),
                    c"one".as_ptr(),
                )
            {
                self.record_resolved_type_ast_expr_ast_type(
                    node as *mut _,
                    &self.builtin_types.vector_type as *const _ as *const AstType,
                );
                return false;
            }

            if let Some(library_member_type_cb) = self.library_member_type_cb {
                let object_ptr = ast_node_as::<AstExprGlobal>(
                    node_ref.expr as *mut luaur_ast::records::ast_node::AstNode,
                );
                if !object_ptr.is_null() {
                    let object = &*object_ptr;
                    let raw_ty = library_member_type_cb(object.name.value, node_ref.index.value);
                    let ty = LuauBytecodeType(raw_ty as u16);

                    if ty != LBC_TYPE_ANY {
                        match ty {
                            LBC_TYPE_BOOLEAN => {
                                self.resolved_exprs.try_insert(
                                    node as *mut AstExpr,
                                    &self.builtin_types.boolean_type as *const _ as *const AstType,
                                );
                            }
                            LBC_TYPE_NUMBER => {
                                self.resolved_exprs.try_insert(
                                    node as *mut AstExpr,
                                    &self.builtin_types.number_type as *const _ as *const AstType,
                                );
                            }
                            LBC_TYPE_INTEGER => {
                                self.resolved_exprs.try_insert(
                                    node as *mut AstExpr,
                                    &self.builtin_types.integer_type as *const _ as *const AstType,
                                );
                            }
                            LBC_TYPE_STRING => {
                                self.resolved_exprs.try_insert(
                                    node as *mut AstExpr,
                                    &self.builtin_types.string_type as *const _ as *const AstType,
                                );
                            }
                            LBC_TYPE_VECTOR => {
                                self.resolved_exprs.try_insert(
                                    node as *mut AstExpr,
                                    &self.builtin_types.vector_type as *const _ as *const AstType,
                                );
                            }
                            _ => {}
                        }

                        self.expr_types.try_insert(node as *mut AstExpr, ty);
                        return false;
                    }
                }
            }

            false
        }
    }
}
