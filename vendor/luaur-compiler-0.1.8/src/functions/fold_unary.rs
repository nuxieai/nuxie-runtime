use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;
use luaur_ast::records::ast_expr_unary::AstExprUnaryOp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub fn fold_unary(result: &mut Constant, op: AstExprUnaryOp, arg: &Constant) {
    match op {
        AstExprUnaryOp::Not => {
            if arg.r#type != Type::Type_Unknown {
                result.r#type = Type::Type_Boolean;
                unsafe {
                    result.data.value_boolean = !arg.is_truthful();
                }
            }
        }
        AstExprUnaryOp::Minus => {
            if arg.r#type == Type::Type_Number {
                result.r#type = Type::Type_Number;
                unsafe {
                    result.data.value_number = -arg.data.value_number;
                }
            } else if arg.r#type == Type::Type_Vector {
                result.r#type = Type::Type_Vector;
                unsafe {
                    result.data.value_vector[0] = -arg.data.value_vector[0];
                    result.data.value_vector[1] = -arg.data.value_vector[1];
                    result.data.value_vector[2] = -arg.data.value_vector[2];
                    result.data.value_vector[3] = -arg.data.value_vector[3];
                }
            }
        }
        AstExprUnaryOp::Len => {
            if arg.r#type == Type::Type_String {
                result.r#type = Type::Type_Number;
                unsafe {
                    result.data.value_number = arg.string_length as f64;
                }
            }
        }
        _ => {
            LUAU_ASSERT!(false);
        }
    }
}
