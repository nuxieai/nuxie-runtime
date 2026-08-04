use crate::records::builtin_info::BuiltinInfo;
use luaur_common::enums::luau_builtin_function::LuauBuiltinFunction;
use luaur_common::macros::luau_unreachable::LUAU_UNREACHABLE;

pub(crate) fn get_builtin_info(bfid: i32) -> BuiltinInfo {
    #[allow(non_upper_case_globals)]
    match unsafe { core::mem::transmute::<u8, LuauBuiltinFunction>(bfid as u8) } {
        LuauBuiltinFunction::LBF_NONE => BuiltinInfo {
            params: -1,
            results: -1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_ASSERT => BuiltinInfo {
            params: -1,
            results: -1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_MATH_ABS
        | LuauBuiltinFunction::LBF_MATH_ACOS
        | LuauBuiltinFunction::LBF_MATH_ASIN => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_MATH_ATAN2 => BuiltinInfo {
            params: 2,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_MATH_ATAN
        | LuauBuiltinFunction::LBF_MATH_CEIL
        | LuauBuiltinFunction::LBF_MATH_COSH
        | LuauBuiltinFunction::LBF_MATH_COS
        | LuauBuiltinFunction::LBF_MATH_DEG
        | LuauBuiltinFunction::LBF_MATH_EXP
        | LuauBuiltinFunction::LBF_MATH_FLOOR => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_MATH_FMOD => BuiltinInfo {
            params: 2,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_MATH_FREXP => BuiltinInfo {
            params: 1,
            results: 2,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_MATH_LDEXP => BuiltinInfo {
            params: 2,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_MATH_LOG10 => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_MATH_LOG => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_MATH_MAX | LuauBuiltinFunction::LBF_MATH_MIN => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_MATH_MODF => BuiltinInfo {
            params: 1,
            results: 2,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_MATH_POW => BuiltinInfo {
            params: 2,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_MATH_RAD
        | LuauBuiltinFunction::LBF_MATH_SINH
        | LuauBuiltinFunction::LBF_MATH_SIN
        | LuauBuiltinFunction::LBF_MATH_SQRT
        | LuauBuiltinFunction::LBF_MATH_TANH
        | LuauBuiltinFunction::LBF_MATH_TAN => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_BIT32_ARSHIFT => BuiltinInfo {
            params: 2,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_BIT32_BAND => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_BIT32_BNOT => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_BIT32_BOR
        | LuauBuiltinFunction::LBF_BIT32_BXOR
        | LuauBuiltinFunction::LBF_BIT32_BTEST => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_BIT32_EXTRACT => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_BIT32_LROTATE | LuauBuiltinFunction::LBF_BIT32_LSHIFT => {
            BuiltinInfo {
                params: 2,
                results: 1,
                flags: BuiltinInfo::Flag_NoneSafe,
            }
        }

        LuauBuiltinFunction::LBF_BIT32_REPLACE => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_BIT32_RROTATE | LuauBuiltinFunction::LBF_BIT32_RSHIFT => {
            BuiltinInfo {
                params: 2,
                results: 1,
                flags: BuiltinInfo::Flag_NoneSafe,
            }
        }

        LuauBuiltinFunction::LBF_TYPE => BuiltinInfo {
            params: 1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_STRING_BYTE => BuiltinInfo {
            params: -1,
            results: -1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_STRING_CHAR => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_STRING_LEN => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_TYPEOF => BuiltinInfo {
            params: 1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_STRING_SUB => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_MATH_CLAMP => BuiltinInfo {
            params: 3,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_MATH_SIGN | LuauBuiltinFunction::LBF_MATH_ROUND => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_RAWSET => BuiltinInfo {
            params: 3,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_RAWGET | LuauBuiltinFunction::LBF_RAWEQUAL => BuiltinInfo {
            params: 2,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_TABLE_INSERT => BuiltinInfo {
            params: -1,
            results: 0,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_TABLE_UNPACK => BuiltinInfo {
            params: -1,
            results: -1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_VECTOR => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_BIT32_COUNTLZ | LuauBuiltinFunction::LBF_BIT32_COUNTRZ => {
            BuiltinInfo {
                params: 1,
                results: 1,
                flags: BuiltinInfo::Flag_NoneSafe,
            }
        }

        LuauBuiltinFunction::LBF_SELECT_VARARG => BuiltinInfo {
            params: -1,
            results: -1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_RAWLEN => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_BIT32_EXTRACTK => BuiltinInfo {
            params: 3,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_GETMETATABLE => BuiltinInfo {
            params: 1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_SETMETATABLE => BuiltinInfo {
            params: 2,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_TONUMBER => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_TOSTRING => BuiltinInfo {
            params: 1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_BIT32_BYTESWAP => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_BUFFER_READI8
        | LuauBuiltinFunction::LBF_BUFFER_READU8
        | LuauBuiltinFunction::LBF_BUFFER_READI16
        | LuauBuiltinFunction::LBF_BUFFER_READU16
        | LuauBuiltinFunction::LBF_BUFFER_READI32
        | LuauBuiltinFunction::LBF_BUFFER_READU32
        | LuauBuiltinFunction::LBF_BUFFER_READF32
        | LuauBuiltinFunction::LBF_BUFFER_READF64 => BuiltinInfo {
            params: 2,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_BUFFER_WRITEU8
        | LuauBuiltinFunction::LBF_BUFFER_WRITEU16
        | LuauBuiltinFunction::LBF_BUFFER_WRITEU32
        | LuauBuiltinFunction::LBF_BUFFER_WRITEF32
        | LuauBuiltinFunction::LBF_BUFFER_WRITEF64
        | LuauBuiltinFunction::LBF_BUFFER_WRITEINTEGER => BuiltinInfo {
            params: 3,
            results: 0,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_BUFFER_READINTEGER => BuiltinInfo {
            params: 2,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_VECTOR_MAGNITUDE | LuauBuiltinFunction::LBF_VECTOR_NORMALIZE => {
            BuiltinInfo {
                params: 1,
                results: 1,
                flags: BuiltinInfo::Flag_NoneSafe,
            }
        }
        LuauBuiltinFunction::LBF_VECTOR_CROSS | LuauBuiltinFunction::LBF_VECTOR_DOT => {
            BuiltinInfo {
                params: 2,
                results: 1,
                flags: BuiltinInfo::Flag_NoneSafe,
            }
        }
        LuauBuiltinFunction::LBF_VECTOR_FLOOR
        | LuauBuiltinFunction::LBF_VECTOR_CEIL
        | LuauBuiltinFunction::LBF_VECTOR_ABS
        | LuauBuiltinFunction::LBF_VECTOR_SIGN => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },
        LuauBuiltinFunction::LBF_VECTOR_CLAMP => BuiltinInfo {
            params: 3,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },
        LuauBuiltinFunction::LBF_VECTOR_MIN | LuauBuiltinFunction::LBF_VECTOR_MAX => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },
        LuauBuiltinFunction::LBF_VECTOR_LERP => BuiltinInfo {
            params: 3,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_MATH_LERP => BuiltinInfo {
            params: 3,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },
        LuauBuiltinFunction::LBF_MATH_ISNAN => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },
        LuauBuiltinFunction::LBF_MATH_ISINF => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },
        LuauBuiltinFunction::LBF_MATH_ISFINITE => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_INTEGER_BAND
        | LuauBuiltinFunction::LBF_INTEGER_BOR
        | LuauBuiltinFunction::LBF_INTEGER_BXOR
        | LuauBuiltinFunction::LBF_INTEGER_BTEST
        | LuauBuiltinFunction::LBF_INTEGER_MIN
        | LuauBuiltinFunction::LBF_INTEGER_MAX => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_INTEGER_EXTRACT => BuiltinInfo {
            params: -1,
            results: 1,
            flags: 0,
        },

        LuauBuiltinFunction::LBF_INTEGER_BNOT
        | LuauBuiltinFunction::LBF_INTEGER_BSWAP
        | LuauBuiltinFunction::LBF_INTEGER_NEG
        | LuauBuiltinFunction::LBF_INTEGER_COUNTLZ
        | LuauBuiltinFunction::LBF_INTEGER_COUNTRZ
        | LuauBuiltinFunction::LBF_INTEGER_TONUMBER
        | LuauBuiltinFunction::LBF_INTEGER_CREATE => BuiltinInfo {
            params: 1,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_INTEGER_CLAMP => BuiltinInfo {
            params: 3,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        LuauBuiltinFunction::LBF_INTEGER_ADD
        | LuauBuiltinFunction::LBF_INTEGER_SUB
        | LuauBuiltinFunction::LBF_INTEGER_DIV
        | LuauBuiltinFunction::LBF_INTEGER_REM
        | LuauBuiltinFunction::LBF_INTEGER_UDIV
        | LuauBuiltinFunction::LBF_INTEGER_UREM
        | LuauBuiltinFunction::LBF_INTEGER_MOD
        | LuauBuiltinFunction::LBF_INTEGER_MUL
        | LuauBuiltinFunction::LBF_INTEGER_IDIV
        | LuauBuiltinFunction::LBF_INTEGER_LT
        | LuauBuiltinFunction::LBF_INTEGER_LE
        | LuauBuiltinFunction::LBF_INTEGER_ULT
        | LuauBuiltinFunction::LBF_INTEGER_ULE
        | LuauBuiltinFunction::LBF_INTEGER_GT
        | LuauBuiltinFunction::LBF_INTEGER_GE
        | LuauBuiltinFunction::LBF_INTEGER_UGT
        | LuauBuiltinFunction::LBF_INTEGER_UGE
        | LuauBuiltinFunction::LBF_INTEGER_LSHIFT
        | LuauBuiltinFunction::LBF_INTEGER_RSHIFT
        | LuauBuiltinFunction::LBF_INTEGER_ARSHIFT
        | LuauBuiltinFunction::LBF_INTEGER_LROTATE
        | LuauBuiltinFunction::LBF_INTEGER_RROTATE => BuiltinInfo {
            params: 2,
            results: 1,
            flags: BuiltinInfo::Flag_NoneSafe,
        },

        _ => {
            LUAU_UNREACHABLE!();
        }
    }
}
