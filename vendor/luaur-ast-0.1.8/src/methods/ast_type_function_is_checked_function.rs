//! `AstTypeFunction::isCheckedFunction` (`Ast/src/Ast.cpp:1218`).
//! Hand-ported (mutually false-blocked against the identically named
//! `AstStatDeclareFunction::isCheckedFunction` via a bare-name method edge).

use crate::records::ast_attr::AstAttrType;
use crate::records::ast_type_function::AstTypeFunction;

impl AstTypeFunction {
    pub fn is_checked_function(&self) -> bool {
        for &attr in self.attributes.iter() {
            if !attr.is_null() && unsafe { (*attr).r#type } == AstAttrType::Checked {
                return true;
            }
        }
        false
    }
}
