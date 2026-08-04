//! `AstStatDeclareFunction::isCheckedFunction` (`Ast/src/Ast.cpp:1051`).
//! Hand-ported (the scheduler mutually false-blocks it against the identically
//! named `AstTypeFunction::isCheckedFunction` via a bare-name method edge).

use crate::records::ast_attr::AstAttrType;
use crate::records::ast_stat_declare_function::AstStatDeclareFunction;

impl AstStatDeclareFunction {
    pub fn is_checked_function(&self) -> bool {
        for &attr in self.attributes.iter() {
            if !attr.is_null() && unsafe { (*attr).r#type } == AstAttrType::Checked {
                return true;
            }
        }
        false
    }
}
