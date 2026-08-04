use alloc::vec::Vec;
use core::ffi::c_char;
use luaur_ast::records::ast_expr_binary::AstExprBinaryOp;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

use crate::enums::type_constant_folding::Type;
use crate::functions::constants_equal::constants_equal;
use crate::records::constant::Constant;

pub fn fold_binary(
    result: &mut Constant,
    op: AstExprBinaryOp,
    la: &Constant,
    ra: &Constant,
    string_table: &mut AstNameTable,
) {
    const K_CONSTANT_FOLD_STRING_LIMIT: u32 = 4096;

    match op {
        AstExprBinaryOp::Add => {
            if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Number {
                result.r#type = Type::Type_Number;
                unsafe {
                    result.data.value_number = la.data.value_number + ra.data.value_number;
                }
            } else if la.r#type == Type::Type_Vector && ra.r#type == Type::Type_Vector {
                result.r#type = Type::Type_Vector;
                unsafe {
                    for i in 0..4 {
                        result.data.value_vector[i] =
                            la.data.value_vector[i] + ra.data.value_vector[i];
                    }
                }
            }
        }
        AstExprBinaryOp::Sub => {
            if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Number {
                result.r#type = Type::Type_Number;
                unsafe {
                    result.data.value_number = la.data.value_number - ra.data.value_number;
                }
            } else if la.r#type == Type::Type_Vector && ra.r#type == Type::Type_Vector {
                result.r#type = Type::Type_Vector;
                unsafe {
                    for i in 0..4 {
                        result.data.value_vector[i] =
                            la.data.value_vector[i] - ra.data.value_vector[i];
                    }
                }
            }
        }
        AstExprBinaryOp::Mul => {
            if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Number {
                result.r#type = Type::Type_Number;
                unsafe {
                    result.data.value_number = la.data.value_number * ra.data.value_number;
                }
            } else if la.r#type == Type::Type_Vector && ra.r#type == Type::Type_Vector {
                unsafe {
                    let had_w = la.data.value_vector[3] != 0.0 || ra.data.value_vector[3] != 0.0;
                    let result_w = la.data.value_vector[3] * ra.data.value_vector[3];
                    if result_w == 0.0 || had_w {
                        result.r#type = Type::Type_Vector;
                        for i in 0..3 {
                            result.data.value_vector[i] =
                                la.data.value_vector[i] * ra.data.value_vector[i];
                        }
                        result.data.value_vector[3] = result_w;
                    }
                }
            } else if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Vector {
                unsafe {
                    let had_w = ra.data.value_vector[3] != 0.0;
                    let result_w = (la.data.value_number as f32) * ra.data.value_vector[3];
                    if result_w == 0.0 || had_w {
                        result.r#type = Type::Type_Vector;
                        for i in 0..3 {
                            result.data.value_vector[i] =
                                (la.data.value_number as f32) * ra.data.value_vector[i];
                        }
                        result.data.value_vector[3] = result_w;
                    }
                }
            } else if la.r#type == Type::Type_Vector && ra.r#type == Type::Type_Number {
                unsafe {
                    let had_w = la.data.value_vector[3] != 0.0;
                    let result_w = la.data.value_vector[3] * (ra.data.value_number as f32);
                    if result_w == 0.0 || had_w {
                        result.r#type = Type::Type_Vector;
                        for i in 0..3 {
                            result.data.value_vector[i] =
                                la.data.value_vector[i] * (ra.data.value_number as f32);
                        }
                        result.data.value_vector[3] = result_w;
                    }
                }
            }
        }
        AstExprBinaryOp::Div => {
            if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Number {
                result.r#type = Type::Type_Number;
                unsafe {
                    result.data.value_number = la.data.value_number / ra.data.value_number;
                }
            } else if la.r#type == Type::Type_Vector && ra.r#type == Type::Type_Vector {
                unsafe {
                    let had_w = la.data.value_vector[3] != 0.0 || ra.data.value_vector[3] != 0.0;
                    let result_w = la.data.value_vector[3] / ra.data.value_vector[3];
                    if result_w == 0.0 || had_w {
                        result.r#type = Type::Type_Vector;
                        for i in 0..3 {
                            result.data.value_vector[i] =
                                la.data.value_vector[i] / ra.data.value_vector[i];
                        }
                        result.data.value_vector[3] = result_w;
                    }
                }
            } else if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Vector {
                unsafe {
                    let had_w = ra.data.value_vector[3] != 0.0;
                    let result_w = (la.data.value_number as f32) / ra.data.value_vector[3];
                    if result_w == 0.0 || had_w {
                        result.r#type = Type::Type_Vector;
                        for i in 0..3 {
                            result.data.value_vector[i] =
                                (la.data.value_number as f32) / ra.data.value_vector[i];
                        }
                        result.data.value_vector[3] = result_w;
                    }
                }
            } else if la.r#type == Type::Type_Vector && ra.r#type == Type::Type_Number {
                unsafe {
                    let had_w = la.data.value_vector[3] != 0.0;
                    let result_w = la.data.value_vector[3] / (ra.data.value_number as f32);
                    if result_w == 0.0 || had_w {
                        result.r#type = Type::Type_Vector;
                        for i in 0..3 {
                            result.data.value_vector[i] =
                                la.data.value_vector[i] / (ra.data.value_number as f32);
                        }
                        result.data.value_vector[3] = result_w;
                    }
                }
            }
        }
        AstExprBinaryOp::FloorDiv => {
            if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Number {
                result.r#type = Type::Type_Number;
                unsafe {
                    result.data.value_number =
                        (la.data.value_number / ra.data.value_number).floor();
                }
            } else if la.r#type == Type::Type_Vector && ra.r#type == Type::Type_Vector {
                unsafe {
                    let had_w = la.data.value_vector[3] != 0.0 || ra.data.value_vector[3] != 0.0;
                    let result_w = (la.data.value_vector[3] / ra.data.value_vector[3]).floor();
                    if result_w == 0.0 || had_w {
                        result.r#type = Type::Type_Vector;
                        for i in 0..3 {
                            result.data.value_vector[i] =
                                (la.data.value_vector[i] / ra.data.value_vector[i]).floor();
                        }
                        result.data.value_vector[3] = result_w;
                    }
                }
            } else if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Vector {
                unsafe {
                    let had_w = ra.data.value_vector[3] != 0.0;
                    let result_w =
                        ((la.data.value_number as f32) / ra.data.value_vector[3]).floor();
                    if result_w == 0.0 || had_w {
                        result.r#type = Type::Type_Vector;
                        for i in 0..3 {
                            result.data.value_vector[i] =
                                ((la.data.value_number as f32) / ra.data.value_vector[i]).floor();
                        }
                        result.data.value_vector[3] = result_w;
                    }
                }
            } else if la.r#type == Type::Type_Vector && ra.r#type == Type::Type_Number {
                unsafe {
                    let had_w = la.data.value_vector[3] != 0.0;
                    let result_w =
                        (la.data.value_vector[3] / (ra.data.value_number as f32)).floor();
                    if result_w == 0.0 || had_w {
                        result.r#type = Type::Type_Vector;
                        for i in 0..3 {
                            result.data.value_vector[i] =
                                (la.data.value_vector[i] / (ra.data.value_number as f32)).floor();
                        }
                        result.data.value_vector[3] = result_w;
                    }
                }
            }
        }
        AstExprBinaryOp::Mod => {
            if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Number {
                result.r#type = Type::Type_Number;
                unsafe {
                    result.data.value_number = la.data.value_number
                        - (la.data.value_number / ra.data.value_number).floor()
                            * ra.data.value_number;
                }
            }
        }
        AstExprBinaryOp::Pow => {
            if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Number {
                result.r#type = Type::Type_Number;
                unsafe {
                    result.data.value_number = la.data.value_number.powf(ra.data.value_number);
                }
            }
        }
        AstExprBinaryOp::Concat => {
            if la.r#type == Type::Type_String
                && ra.r#type == Type::Type_String
                && (la.string_length + ra.string_length) <= K_CONSTANT_FOLD_STRING_LIMIT
            {
                result.r#type = Type::Type_String;
                result.string_length = la.string_length + ra.string_length;
                if la.string_length == 0 {
                    unsafe {
                        result.data.value_string = ra.data.value_string;
                    }
                } else if ra.string_length == 0 {
                    unsafe {
                        result.data.value_string = la.data.value_string;
                    }
                } else {
                    let mut tmp = Vec::with_capacity(result.string_length as usize);
                    let la_slice = la.get_string();
                    let ra_slice = ra.get_string();
                    tmp.extend_from_slice(unsafe {
                        core::slice::from_raw_parts(
                            la_slice.as_slice().as_ptr() as *const u8,
                            la_slice.len(),
                        )
                    });
                    tmp.extend_from_slice(unsafe {
                        core::slice::from_raw_parts(
                            ra_slice.as_slice().as_ptr() as *const u8,
                            ra_slice.len(),
                        )
                    });
                    let name = string_table.get_or_add(tmp.as_ptr() as *const c_char, tmp.len());
                    unsafe {
                        result.data.value_string = name.value;
                    }
                }
            }
        }
        AstExprBinaryOp::CompareNe => {
            if la.r#type != Type::Type_Unknown && ra.r#type != Type::Type_Unknown {
                result.r#type = Type::Type_Boolean;
                unsafe {
                    result.data.value_boolean = !constants_equal(la, ra);
                }
            }
        }
        AstExprBinaryOp::CompareEq => {
            if la.r#type != Type::Type_Unknown && ra.r#type != Type::Type_Unknown {
                result.r#type = Type::Type_Boolean;
                unsafe {
                    result.data.value_boolean = constants_equal(la, ra);
                }
            }
        }
        AstExprBinaryOp::CompareLt => {
            if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Number {
                result.r#type = Type::Type_Boolean;
                unsafe {
                    result.data.value_boolean = la.data.value_number < ra.data.value_number;
                }
            }
        }
        AstExprBinaryOp::CompareLe => {
            if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Number {
                result.r#type = Type::Type_Boolean;
                unsafe {
                    result.data.value_boolean = la.data.value_number <= ra.data.value_number;
                }
            }
        }
        AstExprBinaryOp::CompareGt => {
            if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Number {
                result.r#type = Type::Type_Boolean;
                unsafe {
                    result.data.value_boolean = la.data.value_number > ra.data.value_number;
                }
            }
        }
        AstExprBinaryOp::CompareGe => {
            if la.r#type == Type::Type_Number && ra.r#type == Type::Type_Number {
                result.r#type = Type::Type_Boolean;
                unsafe {
                    result.data.value_boolean = la.data.value_number >= ra.data.value_number;
                }
            }
        }
        AstExprBinaryOp::And => {
            if la.r#type != Type::Type_Unknown {
                *result = if la.is_truthful() { *ra } else { *la };
            }
        }
        AstExprBinaryOp::Or => {
            if la.r#type != Type::Type_Unknown {
                *result = if la.is_truthful() { *la } else { *ra };
            }
        }
        _ => {
            LUAU_ASSERT!(false);
        }
    }
}
