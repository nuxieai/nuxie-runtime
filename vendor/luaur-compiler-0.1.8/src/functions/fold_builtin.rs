use crate::enums::type_constant_folding::Type;
use crate::functions::bit_32::bit32;
use crate::functions::cbool::cbool;
use crate::functions::cnum::cnum;
use crate::functions::cstring_builtin_folding::cstring_c_char;
use crate::functions::cstring_builtin_folding_alt_b::cstring_c_char_usize;
use crate::functions::ctype::ctype;
use crate::functions::ctypeof::ctypeof;
use crate::functions::cvar::cvar;
use crate::functions::cvector::cvector;
use crate::records::constant::Constant;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_common::enums::luau_builtin_function::LuauBuiltinFunction::{self, *};

// C++ BuiltinFolding.cpp:17 `const double kRadDeg = kPi / 180.0;` (~0.0174533).
// The previous value was the inverse (180/pi), which swapped math.deg/math.rad folding.
const K_RAD_DEG: f64 = 3.14159265358979323846_f64 / 180.0;
const K_STRING_CHAR_FOLD_LIMIT: usize = 128;

#[allow(non_snake_case)]
pub fn fold_builtin(
    string_table: &mut AstNameTable,
    bfid: i32,
    args: *const Constant,
    count: usize,
) -> Constant {
    // `slice::from_raw_parts(null, 0)` is UB; builtins with no args pass a null
    // `args` pointer with count 0, so produce an empty slice in that case.
    let args = if count == 0 {
        &[][..]
    } else {
        unsafe { core::slice::from_raw_parts(args, count) }
    };

    match bfid {
        bfid if bfid == LBF_MATH_ABS as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.abs());
            }
        }

        bfid if bfid == LBF_MATH_ACOS as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.acos());
            }
        }

        bfid if bfid == LBF_MATH_ASIN as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.asin());
            }
        }

        bfid if bfid == LBF_MATH_ATAN2 as i32 => {
            if count == 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
            {
                return cnum(
                    unsafe { args[0].data.value_number }
                        .atan2(unsafe { args[1].data.value_number }),
                );
            }
        }

        bfid if bfid == LBF_MATH_ATAN as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.atan());
            }
        }

        bfid if bfid == LBF_MATH_CEIL as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.ceil());
            }
        }

        bfid if bfid == LBF_MATH_COSH as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.cosh());
            }
        }

        bfid if bfid == LBF_MATH_COS as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.cos());
            }
        }

        bfid if bfid == LBF_MATH_DEG as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number } / K_RAD_DEG);
            }
        }

        bfid if bfid == LBF_MATH_EXP as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.exp());
            }
        }

        bfid if bfid == LBF_MATH_FLOOR as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.floor());
            }
        }

        bfid if bfid == LBF_MATH_FMOD as i32 => {
            if count == 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
            {
                return cnum(
                    unsafe { args[0].data.value_number } % unsafe { args[1].data.value_number },
                );
            }
        }

        bfid if bfid == LBF_MATH_LDEXP as i32 => {
            if count == 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
            {
                let n = unsafe { args[0].data.value_number };
                let exp = unsafe { args[1].data.value_number } as i32;
                return cnum(n * (2.0f64.powi(exp)));
            }
        }

        bfid if bfid == LBF_MATH_LOG10 as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.log10());
            }
        }

        bfid if bfid == LBF_MATH_LOG as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.ln());
            } else if count == 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
            {
                let base = unsafe { args[1].data.value_number };
                if base == 2.0 {
                    return cnum(unsafe { args[0].data.value_number }.log2());
                } else if base == 10.0 {
                    return cnum(unsafe { args[0].data.value_number }.log10());
                } else {
                    return cnum(unsafe { args[0].data.value_number }.ln() / base.ln());
                }
            }
        }

        bfid if bfid == LBF_MATH_MAX as i32 => {
            if count >= 1 && args[0].r#type == Type::Type_Number {
                let mut r = unsafe { args[0].data.value_number };
                for i in 1..count {
                    if args[i].r#type != Type::Type_Number {
                        return cvar();
                    }
                    let a = unsafe { args[i].data.value_number };
                    r = if a > r { a } else { r };
                }
                return cnum(r);
            }
        }

        bfid if bfid == LBF_MATH_MIN as i32 => {
            if count >= 1 && args[0].r#type == Type::Type_Number {
                let mut r = unsafe { args[0].data.value_number };
                for i in 1..count {
                    if args[i].r#type != Type::Type_Number {
                        return cvar();
                    }
                    let a = unsafe { args[i].data.value_number };
                    r = if a < r { a } else { r };
                }
                return cnum(r);
            }
        }

        bfid if bfid == LBF_MATH_POW as i32 => {
            if count == 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
            {
                return cnum(
                    unsafe { args[0].data.value_number }.powf(unsafe { args[1].data.value_number }),
                );
            }
        }

        bfid if bfid == LBF_MATH_RAD as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number } * K_RAD_DEG);
            }
        }

        bfid if bfid == LBF_MATH_SINH as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.sinh());
            }
        }

        bfid if bfid == LBF_MATH_SIN as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.sin());
            }
        }

        bfid if bfid == LBF_MATH_SQRT as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.sqrt());
            }
        }

        bfid if bfid == LBF_MATH_TANH as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.tanh());
            }
        }

        bfid if bfid == LBF_MATH_TAN as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.tan());
            }
        }

        bfid if bfid == LBF_BIT32_ARSHIFT as i32 => {
            if count == 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
            {
                let u = bit32(unsafe { args[0].data.value_number });
                let s = unsafe { args[1].data.value_number } as i32;
                if (s as u32) < 32 {
                    return cnum((u as i32 >> s) as u32 as f64);
                }
            }
        }

        bfid if bfid == LBF_BIT32_BAND as i32 => {
            if count >= 1 && args[0].r#type == Type::Type_Number {
                let mut r = bit32(unsafe { args[0].data.value_number });
                for i in 1..count {
                    if args[i].r#type != Type::Type_Number {
                        return cvar();
                    }
                    r &= bit32(unsafe { args[i].data.value_number });
                }
                return cnum(r as f64);
            }
        }

        bfid if bfid == LBF_BIT32_BNOT as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum((!bit32(unsafe { args[0].data.value_number })) as f64);
            }
        }

        bfid if bfid == LBF_BIT32_BOR as i32 => {
            if count >= 1 && args[0].r#type == Type::Type_Number {
                let mut r = bit32(unsafe { args[0].data.value_number });
                for i in 1..count {
                    if args[i].r#type != Type::Type_Number {
                        return cvar();
                    }
                    r |= bit32(unsafe { args[i].data.value_number });
                }
                return cnum(r as f64);
            }
        }

        bfid if bfid == LBF_BIT32_BXOR as i32 => {
            if count >= 1 && args[0].r#type == Type::Type_Number {
                let mut r = bit32(unsafe { args[0].data.value_number });
                for i in 1..count {
                    if args[i].r#type != Type::Type_Number {
                        return cvar();
                    }
                    r ^= bit32(unsafe { args[i].data.value_number });
                }
                return cnum(r as f64);
            }
        }

        bfid if bfid == LBF_BIT32_BTEST as i32 => {
            if count >= 1 && args[0].r#type == Type::Type_Number {
                let mut r = bit32(unsafe { args[0].data.value_number });
                for i in 1..count {
                    if args[i].r#type != Type::Type_Number {
                        return cvar();
                    }
                    r &= bit32(unsafe { args[i].data.value_number });
                }
                return cbool(r != 0);
            }
        }

        bfid if bfid == LBF_BIT32_EXTRACT as i32 => {
            if count >= 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
                && (count == 2 || args[2].r#type == Type::Type_Number)
            {
                let u = bit32(unsafe { args[0].data.value_number });
                let f = unsafe { args[1].data.value_number } as i32;
                let w = if count == 2 {
                    1
                } else {
                    (unsafe { args[2].data.value_number }) as i32
                };
                if f >= 0 && w > 0 && f + w <= 32 {
                    let m = !(0xfffffffeu32 << (w - 1));
                    return cnum(((u >> f) & m) as f64);
                }
            }
        }

        bfid if bfid == LBF_BIT32_LROTATE as i32 => {
            if count == 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
            {
                let u = bit32(unsafe { args[0].data.value_number });
                let s = unsafe { args[1].data.value_number } as i32;
                return cnum(u.rotate_left((s & 31) as u32) as f64);
            }
        }

        bfid if bfid == LBF_BIT32_LSHIFT as i32 => {
            if count == 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
            {
                let u = bit32(unsafe { args[0].data.value_number });
                let s = unsafe { args[1].data.value_number } as i32;
                if (s as u32) < 32 {
                    return cnum((u << s) as f64);
                }
            }
        }

        bfid if bfid == LBF_BIT32_REPLACE as i32 => {
            if count >= 3
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
                && args[2].r#type == Type::Type_Number
                && (count == 3 || args[3].r#type == Type::Type_Number)
            {
                let n = bit32(unsafe { args[0].data.value_number });
                let v = bit32(unsafe { args[1].data.value_number });
                let f = unsafe { args[2].data.value_number } as i32;
                let w = if count == 3 {
                    1
                } else {
                    (unsafe { args[3].data.value_number }) as i32
                };
                if f >= 0 && w > 0 && f + w <= 32 {
                    let m = !(0xfffffffeu32 << (w - 1));
                    return cnum(((n & !(m << f)) | ((v & m) << f)) as f64);
                }
            }
        }

        bfid if bfid == LBF_BIT32_RROTATE as i32 => {
            if count == 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
            {
                let u = bit32(unsafe { args[0].data.value_number });
                let s = unsafe { args[1].data.value_number } as i32;
                return cnum(u.rotate_right((s & 31) as u32) as f64);
            }
        }

        bfid if bfid == LBF_BIT32_RSHIFT as i32 => {
            if count == 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
            {
                let u = bit32(unsafe { args[0].data.value_number });
                let s = unsafe { args[1].data.value_number } as i32;
                if (s as u32) < 32 {
                    return cnum((u >> s) as f64);
                }
            }
        }

        bfid if bfid == LBF_TYPE as i32 => {
            if count == 1 && args[0].r#type != Type::Type_Unknown {
                return ctype(&args[0]);
            }
        }

        bfid if bfid == LBF_STRING_BYTE as i32 => {
            if count == 1 && args[0].r#type == Type::Type_String {
                if args[0].string_length > 0 {
                    let s = unsafe { args[0].data.value_string };
                    return cnum(unsafe { *s as u8 } as f64);
                }
            } else if count == 2
                && args[0].r#type == Type::Type_String
                && args[1].r#type == Type::Type_Number
            {
                let i = unsafe { args[1].data.value_number } as i32;
                if i > 0 && (i as u32) <= args[0].string_length {
                    let s = unsafe { args[0].data.value_string };
                    return cnum(unsafe { *s.add((i - 1) as usize) as u8 } as f64);
                }
            }
        }

        bfid if bfid == LBF_STRING_CHAR as i32 => {
            if count < K_STRING_CHAR_FOLD_LIMIT {
                let mut buf = [0i8; K_STRING_CHAR_FOLD_LIMIT];
                for i in 0..count {
                    if args[i].r#type != Type::Type_Number {
                        return cvar();
                    }
                    let ch = unsafe { args[i].data.value_number } as i32;
                    if (ch as u8 as i32) != ch {
                        return cvar();
                    }
                    buf[i] = ch as i8;
                }
                if count == 0 {
                    return cstring_c_char(c"".as_ptr());
                }
                let name = string_table.get_or_add(buf.as_ptr(), count);
                return cstring_c_char_usize(name.value, count);
            }
        }

        bfid if bfid == LBF_STRING_LEN as i32 => {
            if count == 1 && args[0].r#type == Type::Type_String {
                return cnum(args[0].string_length as f64);
            }
        }

        bfid if bfid == LBF_TYPEOF as i32 => {
            if count == 1 && args[0].r#type != Type::Type_Unknown {
                return ctypeof(&args[0]);
            }
        }

        bfid if bfid == LBF_STRING_SUB as i32 => {
            if count >= 2
                && args[0].r#type == Type::Type_String
                && args[1].r#type == Type::Type_Number
            {
                if count >= 3 && args[2].r#type != Type::Type_Number {
                    return cvar();
                }
                let str_ptr = unsafe { args[0].data.value_string };
                let len = args[0].string_length;
                let mut start = unsafe { args[1].data.value_number } as i32;
                let mut end = if count >= 3 {
                    (unsafe { args[2].data.value_number }) as i32
                } else {
                    len as i32
                };

                if start < 0 {
                    start += len as i32 + 1;
                }
                if end < 0 {
                    end += len as i32 + 1;
                }
                if end < 1 {
                    return cstring_c_char(c"".as_ptr());
                }
                start = if start < 1 { 1 } else { start };
                end = if end > len as i32 { len as i32 } else { end };

                if start <= end {
                    let name = string_table.get_or_add(
                        unsafe { str_ptr.add((start - 1) as usize) },
                        (end - start + 1) as usize,
                    );
                    return cstring_c_char_usize(name.value, (end - start + 1) as usize);
                }
                return cstring_c_char(c"".as_ptr());
            }
        }

        bfid if bfid == LBF_MATH_CLAMP as i32 => {
            if count == 3
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
                && args[2].r#type == Type::Type_Number
            {
                let min = unsafe { args[1].data.value_number };
                let max = unsafe { args[2].data.value_number };
                if min <= max {
                    let mut v = unsafe { args[0].data.value_number };
                    v = if v < min { min } else { v };
                    v = if v > max { max } else { v };
                    return cnum(v);
                }
            }
        }

        bfid if bfid == LBF_MATH_SIGN as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                let v = unsafe { args[0].data.value_number };
                return cnum(if v > 0.0 {
                    1.0
                } else if v < 0.0 {
                    -1.0
                } else {
                    0.0
                });
            }
        }

        bfid if bfid == LBF_MATH_ROUND as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cnum(unsafe { args[0].data.value_number }.round());
            }
        }

        bfid if bfid == LBF_VECTOR as i32 => {
            if count >= 2
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
            {
                if count == 2 {
                    return cvector(
                        unsafe { args[0].data.value_number },
                        unsafe { args[1].data.value_number },
                        0.0,
                        0.0,
                    );
                } else if count == 3 && args[2].r#type == Type::Type_Number {
                    return cvector(
                        unsafe { args[0].data.value_number },
                        unsafe { args[1].data.value_number },
                        unsafe { args[2].data.value_number },
                        0.0,
                    );
                } else if count == 4
                    && args[2].r#type == Type::Type_Number
                    && args[3].r#type == Type::Type_Number
                {
                    return cvector(
                        unsafe { args[0].data.value_number },
                        unsafe { args[1].data.value_number },
                        unsafe { args[2].data.value_number },
                        unsafe { args[3].data.value_number },
                    );
                }
            }
        }

        bfid if bfid == LBF_MATH_LERP as i32 => {
            if count == 3
                && args[0].r#type == Type::Type_Number
                && args[1].r#type == Type::Type_Number
                && args[2].r#type == Type::Type_Number
            {
                let a = unsafe { args[0].data.value_number };
                let b = unsafe { args[1].data.value_number };
                let t = unsafe { args[2].data.value_number };
                let v = if t == 1.0 { b } else { a + (b - a) * t };
                return cnum(v);
            }
        }

        bfid if bfid == LBF_MATH_ISNAN as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cbool(unsafe { args[0].data.value_number }.is_nan());
            }
        }

        bfid if bfid == LBF_MATH_ISINF as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cbool(unsafe { args[0].data.value_number }.is_infinite());
            }
        }

        bfid if bfid == LBF_MATH_ISFINITE as i32 => {
            if count == 1 && args[0].r#type == Type::Type_Number {
                return cbool(unsafe { args[0].data.value_number }.is_finite());
            }
        }

        _ => {}
    }

    cvar()
}
