use crate::records::builtin::Builtin;
use crate::records::compile_options::CompileOptions;
use luaur_common::FFlag;

pub fn get_builtin_function_id(builtin: &Builtin, options: &CompileOptions) -> i32 {
    use luaur_common::enums::luau_builtin_function::LuauBuiltinFunction::*;

    if builtin.is_global("assert") {
        return LBF_ASSERT as i32;
    }

    if builtin.is_global("type") {
        return LBF_TYPE as i32;
    }

    if builtin.is_global("typeof") {
        return LBF_TYPEOF as i32;
    }

    if builtin.is_global("rawset") {
        return LBF_RAWSET as i32;
    }
    if builtin.is_global("rawget") {
        return LBF_RAWGET as i32;
    }
    if builtin.is_global("rawequal") {
        return LBF_RAWEQUAL as i32;
    }
    if builtin.is_global("rawlen") {
        return LBF_RAWLEN as i32;
    }

    if builtin.is_global("unpack") {
        return LBF_TABLE_UNPACK as i32;
    }

    if builtin.is_global("select") {
        return LBF_SELECT_VARARG as i32;
    }

    if builtin.is_global("getmetatable") {
        return LBF_GETMETATABLE as i32;
    }
    if builtin.is_global("setmetatable") {
        return LBF_SETMETATABLE as i32;
    }

    if builtin.is_global("tonumber") {
        return LBF_TONUMBER as i32;
    }
    if builtin.is_global("tostring") {
        return LBF_TOSTRING as i32;
    }

    if builtin.object.operator_eq_c_char(c"math".as_ptr()) {
        if builtin.method.operator_eq_c_char(c"abs".as_ptr()) {
            return LBF_MATH_ABS as i32;
        }
        if builtin.method.operator_eq_c_char(c"acos".as_ptr()) {
            return LBF_MATH_ACOS as i32;
        }
        if builtin.method.operator_eq_c_char(c"asin".as_ptr()) {
            return LBF_MATH_ASIN as i32;
        }
        if builtin.method.operator_eq_c_char(c"atan2".as_ptr()) {
            return LBF_MATH_ATAN2 as i32;
        }
        if builtin.method.operator_eq_c_char(c"atan".as_ptr()) {
            return LBF_MATH_ATAN as i32;
        }
        if builtin.method.operator_eq_c_char(c"ceil".as_ptr()) {
            return LBF_MATH_CEIL as i32;
        }
        if builtin.method.operator_eq_c_char(c"cosh".as_ptr()) {
            return LBF_MATH_COSH as i32;
        }
        if builtin.method.operator_eq_c_char(c"cos".as_ptr()) {
            return LBF_MATH_COS as i32;
        }
        if builtin.method.operator_eq_c_char(c"deg".as_ptr()) {
            return LBF_MATH_DEG as i32;
        }
        if builtin.method.operator_eq_c_char(c"exp".as_ptr()) {
            return LBF_MATH_EXP as i32;
        }
        if builtin.method.operator_eq_c_char(c"floor".as_ptr()) {
            return LBF_MATH_FLOOR as i32;
        }
        if builtin.method.operator_eq_c_char(c"fmod".as_ptr()) {
            return LBF_MATH_FMOD as i32;
        }
        if builtin.method.operator_eq_c_char(c"frexp".as_ptr()) {
            return LBF_MATH_FREXP as i32;
        }
        if builtin.method.operator_eq_c_char(c"ldexp".as_ptr()) {
            return LBF_MATH_LDEXP as i32;
        }
        if builtin.method.operator_eq_c_char(c"log10".as_ptr()) {
            return LBF_MATH_LOG10 as i32;
        }
        if builtin.method.operator_eq_c_char(c"log".as_ptr()) {
            return LBF_MATH_LOG as i32;
        }
        if builtin.method.operator_eq_c_char(c"max".as_ptr()) {
            return LBF_MATH_MAX as i32;
        }
        if builtin.method.operator_eq_c_char(c"min".as_ptr()) {
            return LBF_MATH_MIN as i32;
        }
        if builtin.method.operator_eq_c_char(c"modf".as_ptr()) {
            return LBF_MATH_MODF as i32;
        }
        if builtin.method.operator_eq_c_char(c"pow".as_ptr()) {
            return LBF_MATH_POW as i32;
        }
        if builtin.method.operator_eq_c_char(c"rad".as_ptr()) {
            return LBF_MATH_RAD as i32;
        }
        if builtin.method.operator_eq_c_char(c"sinh".as_ptr()) {
            return LBF_MATH_SINH as i32;
        }
        if builtin.method.operator_eq_c_char(c"sin".as_ptr()) {
            return LBF_MATH_SIN as i32;
        }
        if builtin.method.operator_eq_c_char(c"sqrt".as_ptr()) {
            return LBF_MATH_SQRT as i32;
        }
        if builtin.method.operator_eq_c_char(c"tanh".as_ptr()) {
            return LBF_MATH_TANH as i32;
        }
        if builtin.method.operator_eq_c_char(c"tan".as_ptr()) {
            return LBF_MATH_TAN as i32;
        }
        if builtin.method.operator_eq_c_char(c"clamp".as_ptr()) {
            return LBF_MATH_CLAMP as i32;
        }
        if builtin.method.operator_eq_c_char(c"sign".as_ptr()) {
            return LBF_MATH_SIGN as i32;
        }
        if builtin.method.operator_eq_c_char(c"round".as_ptr()) {
            return LBF_MATH_ROUND as i32;
        }
        if builtin.method.operator_eq_c_char(c"lerp".as_ptr()) {
            return LBF_MATH_LERP as i32;
        }
        if builtin.method.operator_eq_c_char(c"isnan".as_ptr()) {
            return LBF_MATH_ISNAN as i32;
        }
        if builtin.method.operator_eq_c_char(c"isinf".as_ptr()) {
            return LBF_MATH_ISINF as i32;
        }
        if builtin.method.operator_eq_c_char(c"isfinite".as_ptr()) {
            return LBF_MATH_ISFINITE as i32;
        }
    }

    if builtin.object.operator_eq_c_char(c"bit32".as_ptr()) {
        if builtin.method.operator_eq_c_char(c"arshift".as_ptr()) {
            return LBF_BIT32_ARSHIFT as i32;
        }
        if builtin.method.operator_eq_c_char(c"band".as_ptr()) {
            return LBF_BIT32_BAND as i32;
        }
        if builtin.method.operator_eq_c_char(c"bnot".as_ptr()) {
            return LBF_BIT32_BNOT as i32;
        }
        if builtin.method.operator_eq_c_char(c"bor".as_ptr()) {
            return LBF_BIT32_BOR as i32;
        }
        if builtin.method.operator_eq_c_char(c"bxor".as_ptr()) {
            return LBF_BIT32_BXOR as i32;
        }
        if builtin.method.operator_eq_c_char(c"btest".as_ptr()) {
            return LBF_BIT32_BTEST as i32;
        }
        if builtin.method.operator_eq_c_char(c"extract".as_ptr()) {
            return LBF_BIT32_EXTRACT as i32;
        }
        if builtin.method.operator_eq_c_char(c"lrotate".as_ptr()) {
            return LBF_BIT32_LROTATE as i32;
        }
        if builtin.method.operator_eq_c_char(c"lshift".as_ptr()) {
            return LBF_BIT32_LSHIFT as i32;
        }
        if builtin.method.operator_eq_c_char(c"replace".as_ptr()) {
            return LBF_BIT32_REPLACE as i32;
        }
        if builtin.method.operator_eq_c_char(c"rrotate".as_ptr()) {
            return LBF_BIT32_RROTATE as i32;
        }
        if builtin.method.operator_eq_c_char(c"rshift".as_ptr()) {
            return LBF_BIT32_RSHIFT as i32;
        }
        if builtin.method.operator_eq_c_char(c"countlz".as_ptr()) {
            return LBF_BIT32_COUNTLZ as i32;
        }
        if builtin.method.operator_eq_c_char(c"countrz".as_ptr()) {
            return LBF_BIT32_COUNTRZ as i32;
        }
        if builtin.method.operator_eq_c_char(c"byteswap".as_ptr()) {
            return LBF_BIT32_BYTESWAP as i32;
        }
    }

    if builtin.object.operator_eq_c_char(c"string".as_ptr()) {
        if builtin.method.operator_eq_c_char(c"byte".as_ptr()) {
            return LBF_STRING_BYTE as i32;
        }
        if builtin.method.operator_eq_c_char(c"char".as_ptr()) {
            return LBF_STRING_CHAR as i32;
        }
        if builtin.method.operator_eq_c_char(c"len".as_ptr()) {
            return LBF_STRING_LEN as i32;
        }
        if builtin.method.operator_eq_c_char(c"sub".as_ptr()) {
            return LBF_STRING_SUB as i32;
        }
    }

    if builtin.object.operator_eq_c_char(c"table".as_ptr()) {
        if builtin.method.operator_eq_c_char(c"insert".as_ptr()) {
            return LBF_TABLE_INSERT as i32;
        }
        if builtin.method.operator_eq_c_char(c"unpack".as_ptr()) {
            return LBF_TABLE_UNPACK as i32;
        }
    }

    if builtin.object.operator_eq_c_char(c"buffer".as_ptr()) {
        if builtin.method.operator_eq_c_char(c"readi8".as_ptr()) {
            return LBF_BUFFER_READI8 as i32;
        }
        if builtin.method.operator_eq_c_char(c"readu8".as_ptr()) {
            return LBF_BUFFER_READU8 as i32;
        }
        if builtin.method.operator_eq_c_char(c"writei8".as_ptr())
            || builtin.method.operator_eq_c_char(c"writeu8".as_ptr())
        {
            return LBF_BUFFER_WRITEU8 as i32;
        }
        if builtin.method.operator_eq_c_char(c"readi16".as_ptr()) {
            return LBF_BUFFER_READI16 as i32;
        }
        if builtin.method.operator_eq_c_char(c"readu16".as_ptr()) {
            return LBF_BUFFER_READU16 as i32;
        }
        if builtin.method.operator_eq_c_char(c"writei16".as_ptr())
            || builtin.method.operator_eq_c_char(c"writeu16".as_ptr())
        {
            return LBF_BUFFER_WRITEU16 as i32;
        }
        if builtin.method.operator_eq_c_char(c"readi32".as_ptr()) {
            return LBF_BUFFER_READI32 as i32;
        }
        if builtin.method.operator_eq_c_char(c"readu32".as_ptr()) {
            return LBF_BUFFER_READU32 as i32;
        }
        if builtin.method.operator_eq_c_char(c"writei32".as_ptr())
            || builtin.method.operator_eq_c_char(c"writeu32".as_ptr())
        {
            return LBF_BUFFER_WRITEU32 as i32;
        }
        if builtin.method.operator_eq_c_char(c"readf32".as_ptr()) {
            return LBF_BUFFER_READF32 as i32;
        }
        if builtin.method.operator_eq_c_char(c"writef32".as_ptr()) {
            return LBF_BUFFER_WRITEF32 as i32;
        }
        if builtin.method.operator_eq_c_char(c"readf64".as_ptr()) {
            return LBF_BUFFER_READF64 as i32;
        }
        if builtin.method.operator_eq_c_char(c"writef64".as_ptr()) {
            return LBF_BUFFER_WRITEF64 as i32;
        }
        if FFlag::LuauIntegerFastcalls.get()
            && builtin.method.operator_eq_c_char(c"readinteger".as_ptr())
        {
            return LBF_BUFFER_READINTEGER as i32;
        }
        if FFlag::LuauIntegerFastcalls.get()
            && builtin.method.operator_eq_c_char(c"writeinteger".as_ptr())
        {
            return LBF_BUFFER_WRITEINTEGER as i32;
        }
    }

    if builtin.object.operator_eq_c_char(c"vector".as_ptr()) {
        if builtin.method.operator_eq_c_char(c"create".as_ptr()) {
            return LBF_VECTOR as i32;
        }
        if builtin.method.operator_eq_c_char(c"magnitude".as_ptr()) {
            return LBF_VECTOR_MAGNITUDE as i32;
        }
        if builtin.method.operator_eq_c_char(c"normalize".as_ptr()) {
            return LBF_VECTOR_NORMALIZE as i32;
        }
        if builtin.method.operator_eq_c_char(c"cross".as_ptr()) {
            return LBF_VECTOR_CROSS as i32;
        }
        if builtin.method.operator_eq_c_char(c"dot".as_ptr()) {
            return LBF_VECTOR_DOT as i32;
        }
        if builtin.method.operator_eq_c_char(c"floor".as_ptr()) {
            return LBF_VECTOR_FLOOR as i32;
        }
        if builtin.method.operator_eq_c_char(c"ceil".as_ptr()) {
            return LBF_VECTOR_CEIL as i32;
        }
        if builtin.method.operator_eq_c_char(c"abs".as_ptr()) {
            return LBF_VECTOR_ABS as i32;
        }
        if builtin.method.operator_eq_c_char(c"sign".as_ptr()) {
            return LBF_VECTOR_SIGN as i32;
        }
        if builtin.method.operator_eq_c_char(c"clamp".as_ptr()) {
            return LBF_VECTOR_CLAMP as i32;
        }
        if builtin.method.operator_eq_c_char(c"min".as_ptr()) {
            return LBF_VECTOR_MIN as i32;
        }
        if builtin.method.operator_eq_c_char(c"max".as_ptr()) {
            return LBF_VECTOR_MAX as i32;
        }
        if builtin.method.operator_eq_c_char(c"lerp".as_ptr()) {
            return LBF_VECTOR_LERP as i32;
        }
    }

    if FFlag::LuauIntegerFastcalls.get() && builtin.object.operator_eq_c_char(c"integer".as_ptr()) {
        if builtin.method.operator_eq_c_char(c"add".as_ptr()) {
            return LBF_INTEGER_ADD as i32;
        }
        if builtin.method.operator_eq_c_char(c"sub".as_ptr()) {
            return LBF_INTEGER_SUB as i32;
        }
        if builtin.method.operator_eq_c_char(c"mod".as_ptr()) {
            return LBF_INTEGER_MOD as i32;
        }
        if builtin.method.operator_eq_c_char(c"mul".as_ptr()) {
            return LBF_INTEGER_MUL as i32;
        }
        if builtin.method.operator_eq_c_char(c"div".as_ptr()) {
            return LBF_INTEGER_DIV as i32;
        }
        if builtin.method.operator_eq_c_char(c"idiv".as_ptr()) {
            return LBF_INTEGER_IDIV as i32;
        }
        if builtin.method.operator_eq_c_char(c"udiv".as_ptr()) {
            return LBF_INTEGER_UDIV as i32;
        }
        if builtin.method.operator_eq_c_char(c"rem".as_ptr()) {
            return LBF_INTEGER_REM as i32;
        }
        if builtin.method.operator_eq_c_char(c"urem".as_ptr()) {
            return LBF_INTEGER_UREM as i32;
        }
        if builtin.method.operator_eq_c_char(c"min".as_ptr()) {
            return LBF_INTEGER_MIN as i32;
        }
        if builtin.method.operator_eq_c_char(c"max".as_ptr()) {
            return LBF_INTEGER_MAX as i32;
        }
        if builtin.method.operator_eq_c_char(c"neg".as_ptr()) {
            return LBF_INTEGER_NEG as i32;
        }
        if builtin.method.operator_eq_c_char(c"create".as_ptr()) {
            return LBF_INTEGER_CREATE as i32;
        }
        if builtin.method.operator_eq_c_char(c"clamp".as_ptr()) {
            return LBF_INTEGER_CLAMP as i32;
        }
        if builtin.method.operator_eq_c_char(c"band".as_ptr()) {
            return LBF_INTEGER_BAND as i32;
        }
        if builtin.method.operator_eq_c_char(c"bor".as_ptr()) {
            return LBF_INTEGER_BOR as i32;
        }
        if builtin.method.operator_eq_c_char(c"bxor".as_ptr()) {
            return LBF_INTEGER_BXOR as i32;
        }
        if builtin.method.operator_eq_c_char(c"bnot".as_ptr()) {
            return LBF_INTEGER_BNOT as i32;
        }
        if builtin.method.operator_eq_c_char(c"btest".as_ptr()) {
            return LBF_INTEGER_BTEST as i32;
        }
        if builtin.method.operator_eq_c_char(c"bswap".as_ptr()) {
            return LBF_INTEGER_BSWAP as i32;
        }
        if builtin.method.operator_eq_c_char(c"lt".as_ptr()) {
            return LBF_INTEGER_LT as i32;
        }
        if builtin.method.operator_eq_c_char(c"le".as_ptr()) {
            return LBF_INTEGER_LE as i32;
        }
        if builtin.method.operator_eq_c_char(c"ult".as_ptr()) {
            return LBF_INTEGER_ULT as i32;
        }
        if builtin.method.operator_eq_c_char(c"ule".as_ptr()) {
            return LBF_INTEGER_ULE as i32;
        }
        if builtin.method.operator_eq_c_char(c"gt".as_ptr()) {
            return LBF_INTEGER_GT as i32;
        }
        if builtin.method.operator_eq_c_char(c"ge".as_ptr()) {
            return LBF_INTEGER_GE as i32;
        }
        if builtin.method.operator_eq_c_char(c"ugt".as_ptr()) {
            return LBF_INTEGER_UGT as i32;
        }
        if builtin.method.operator_eq_c_char(c"uge".as_ptr()) {
            return LBF_INTEGER_UGE as i32;
        }
        if builtin.method.operator_eq_c_char(c"lshift".as_ptr()) {
            return LBF_INTEGER_LSHIFT as i32;
        }
        if builtin.method.operator_eq_c_char(c"rshift".as_ptr()) {
            return LBF_INTEGER_RSHIFT as i32;
        }
        if builtin.method.operator_eq_c_char(c"arshift".as_ptr()) {
            return LBF_INTEGER_ARSHIFT as i32;
        }
        if builtin.method.operator_eq_c_char(c"lrotate".as_ptr()) {
            return LBF_INTEGER_LROTATE as i32;
        }
        if builtin.method.operator_eq_c_char(c"rrotate".as_ptr()) {
            return LBF_INTEGER_RROTATE as i32;
        }
        if builtin.method.operator_eq_c_char(c"countrz".as_ptr()) {
            return LBF_INTEGER_COUNTRZ as i32;
        }
        if builtin.method.operator_eq_c_char(c"countlz".as_ptr()) {
            return LBF_INTEGER_COUNTLZ as i32;
        }
        if builtin.method.operator_eq_c_char(c"extract".as_ptr()) {
            return LBF_INTEGER_EXTRACT as i32;
        }
        if builtin.method.operator_eq_c_char(c"tonumber".as_ptr()) {
            return LBF_INTEGER_TONUMBER as i32;
        }
    }

    if !options.vector_ctor.is_null() {
        let vector_ctor =
            unsafe { core::ffi::CStr::from_ptr(options.vector_ctor).to_string_lossy() };
        if !options.vector_lib.is_null() {
            let vector_lib =
                unsafe { core::ffi::CStr::from_ptr(options.vector_lib).to_string_lossy() };
            if builtin.is_method(&vector_lib, &vector_ctor) {
                return LBF_VECTOR as i32;
            }
        } else {
            if builtin.is_global(&vector_ctor) {
                return LBF_VECTOR as i32;
            }
        }
    }

    -1
}
