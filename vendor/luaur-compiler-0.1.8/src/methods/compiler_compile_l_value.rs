use crate::enums::kind::Kind;
use crate::functions::sref_compiler::sref_ast_name;
use crate::records::compiler::Compiler;
use crate::records::l_value::LValue;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_ast::records::ast_expr_index_expr::AstExprIndexExpr;
use luaur_ast::records::ast_expr_index_name::AstExprIndexName;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::rtti;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_l_value(&mut self, node: *mut AstExpr, rs: &mut RegScope) -> LValue {
        self.set_debug_line_ast_node(node as *mut _);
        unsafe {
            let expr = rtti::ast_node_as::<AstExprLocal>(node as *mut _);
            if !expr.is_null() {
                if luaur_common::FFlag::LuauExportValueSyntax.get() && (*(*expr).local).is_exported
                {
                    return LValue {
                        kind: Kind::Kind_IndexName,
                        reg: self.get_export_table_reg(node as *mut _),
                        upval: 0,
                        index: 0,
                        number: 0,
                        name: sref_ast_name((*(*expr).local).name),
                        location: (*node).base.location,
                    };
                }
                let reg = self.get_expr_local_reg(node);
                if reg >= 0 {
                    LValue {
                        kind: Kind::Kind_Local,
                        reg: reg as u8,
                        upval: 0,
                        index: 0,
                        number: 0,
                        name: Default::default(),
                        location: (*node).base.location,
                    }
                } else {
                    LUAU_ASSERT!((*expr).upvalue);
                    LValue {
                        kind: Kind::Kind_Upvalue,
                        reg: 0,
                        upval: self.get_upval((*expr).local),
                        index: 0,
                        number: 0,
                        name: Default::default(),
                        location: (*node).base.location,
                    }
                }
            } else {
                let expr = rtti::ast_node_as::<AstExprGlobal>(node as *mut _);
                if !expr.is_null() {
                    LValue {
                        kind: Kind::Kind_Global,
                        reg: 0,
                        upval: 0,
                        index: 0,
                        number: 0,
                        name: sref_ast_name((*expr).name),
                        location: (*node).base.location,
                    }
                } else {
                    let expr = rtti::ast_node_as::<AstExprIndexName>(node as *mut _);
                    if !expr.is_null() {
                        LValue {
                            kind: Kind::Kind_IndexName,
                            reg: self.compile_expr_auto((*expr).expr, rs),
                            upval: 0,
                            index: 0,
                            number: 0,
                            name: sref_ast_name((*expr).index),
                            location: (*node).base.location,
                        }
                    } else {
                        let expr = rtti::ast_node_as::<AstExprIndexExpr>(node as *mut _);
                        if !expr.is_null() {
                            let reg = self.compile_expr_auto((*expr).expr, rs);
                            self.compile_l_value_index(reg, (*expr).index, rs)
                        } else {
                            LUAU_ASSERT!(false);
                            LValue {
                                kind: Kind::Kind_Local,
                                reg: 0,
                                upval: 0,
                                index: 0,
                                number: 0,
                                name: Default::default(),
                                location: (*node).base.location,
                            }
                        }
                    }
                }
            }
        }
    }
}
