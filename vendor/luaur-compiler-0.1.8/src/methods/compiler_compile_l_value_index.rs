use crate::enums::kind::Kind;
use crate::enums::type_constant_folding::Type;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use crate::records::l_value::LValue;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn compile_l_value_index(
        &mut self,
        reg: u8,
        index: *mut AstExpr,
        rs: &mut RegScope,
    ) -> LValue {
        let cv = self.get_constant(index);
        if cv.r#type == Type::Type_Number && {
            let value_number = unsafe { cv.data.value_number };
            value_number >= 1.0
                && value_number <= 256.0
                && (value_number as i32) as f64 == value_number
        } {
            let value_number = unsafe { cv.data.value_number };
            LValue {
                kind: Kind::Kind_IndexNumber,
                reg,
                upval: 0,
                index: 0,
                number: (value_number as i32 - 1) as u8,
                name: Default::default(),
                location: unsafe { (*index).base.location },
            }
        } else if cv.r#type == Type::Type_String {
            LValue {
                kind: Kind::Kind_IndexName,
                reg,
                upval: 0,
                index: 0,
                number: 0,
                // C++ `sref(cv.getString())` preserves the byte length. Routing through
                // AstName::ast_name_c_char treated the data as a NUL-terminated C-string and
                // truncated keys containing embedded NULs (e.g. "a\0" -> "a"), wrongly
                // deduping them with "a".
                name: crate::functions::sref_compiler_alt_c::sref_ast_array_c_char(cv.get_string()),
                location: unsafe { (*index).base.location },
            }
        } else {
            LValue {
                kind: Kind::Kind_IndexExpr,
                reg,
                upval: 0,
                index: self.compile_expr_auto(index, rs),
                number: 0,
                name: Default::default(),
                location: unsafe { (*index).base.location },
            }
        }
    }
}
