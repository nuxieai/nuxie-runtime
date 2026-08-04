use crate::records::ast_class_method::AstClassMethod;
use crate::records::ast_class_property::AstClassProperty;
use crate::records::ast_stat_class::AstStatClass;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::{ast_expr_visit, ast_type_visit, AstVisitable};
use luaur_common::records::variant::Variant2;

impl AstVisitable for AstStatClass {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        luaur_common::LUAU_ASSERT!(luaur_common::FFlag::DebugLuauUserDefinedClasses.get());

        if visitor.visit_stat_class(self as *const Self as *mut core::ffi::c_void) {
            for member in self.members.iter() {
                match member {
                    Variant2::V0(prop) => {
                        let prop: &AstClassProperty = prop;
                        if !prop.ty.is_null() {
                            unsafe {
                                ast_type_visit(prop.ty, visitor);
                            }
                        }
                    }
                    Variant2::V1(method) => {
                        let method: &AstClassMethod = method;
                        unsafe {
                            ast_expr_visit(method.function as *mut _, visitor);
                        }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_class_visit(this: &AstStatClass, visitor: &mut dyn AstVisitor) {
    this.visit(visitor);
}
