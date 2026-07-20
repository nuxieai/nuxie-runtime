#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LuauBuiltinFunction {
    LBF_NONE = 0,

    // assert()
    LBF_ASSERT,

    // math.
    LBF_MATH_ABS,
    LBF_MATH_ACOS,
    LBF_MATH_ASIN,
    LBF_MATH_ATAN2,
    LBF_MATH_ATAN,
    LBF_MATH_CEIL,
    LBF_MATH_COSH,
    LBF_MATH_COS,
    LBF_MATH_DEG,
    LBF_MATH_EXP,
    LBF_MATH_FLOOR,
    LBF_MATH_FMOD,
    LBF_MATH_FREXP,
    LBF_MATH_LDEXP,
    LBF_MATH_LOG10,
    LBF_MATH_LOG,
    LBF_MATH_MAX,
    LBF_MATH_MIN,
    LBF_MATH_MODF,
    LBF_MATH_POW,
    LBF_MATH_RAD,
    LBF_MATH_SINH,
    LBF_MATH_SIN,
    LBF_MATH_SQRT,
    LBF_MATH_TANH,
    LBF_MATH_TAN,

    // bit32.
    LBF_BIT32_ARSHIFT,
    LBF_BIT32_BAND,
    LBF_BIT32_BNOT,
    LBF_BIT32_BOR,
    LBF_BIT32_BXOR,
    LBF_BIT32_BTEST,
    LBF_BIT32_EXTRACT,
    LBF_BIT32_LROTATE,
    LBF_BIT32_LSHIFT,
    LBF_BIT32_REPLACE,
    LBF_BIT32_RROTATE,
    LBF_BIT32_RSHIFT,

    // type()
    LBF_TYPE,

    // string.
    LBF_STRING_BYTE,
    LBF_STRING_CHAR,
    LBF_STRING_LEN,

    // typeof()
    LBF_TYPEOF,

    // string.
    LBF_STRING_SUB,

    // math.
    LBF_MATH_CLAMP,
    LBF_MATH_SIGN,
    LBF_MATH_ROUND,

    // raw*
    LBF_RAWSET,
    LBF_RAWGET,
    LBF_RAWEQUAL,

    // table.
    LBF_TABLE_INSERT,
    LBF_TABLE_UNPACK,

    // vector ctor
    LBF_VECTOR,

    // bit32.count
    LBF_BIT32_COUNTLZ,
    LBF_BIT32_COUNTRZ,

    // select(_, ...)
    LBF_SELECT_VARARG,

    // rawlen
    LBF_RAWLEN,

    // bit32.extract(_, k, k)
    LBF_BIT32_EXTRACTK,

    // get/setmetatable
    LBF_GETMETATABLE,
    LBF_SETMETATABLE,

    // tonumber/tostring
    LBF_TONUMBER,
    LBF_TOSTRING,

    // bit32.byteswap(n)
    LBF_BIT32_BYTESWAP,

    // buffer.
    LBF_BUFFER_READI8,
    LBF_BUFFER_READU8,
    LBF_BUFFER_WRITEU8,
    LBF_BUFFER_READI16,
    LBF_BUFFER_READU16,
    LBF_BUFFER_WRITEU16,
    LBF_BUFFER_READI32,
    LBF_BUFFER_READU32,
    LBF_BUFFER_WRITEU32,
    LBF_BUFFER_READF32,
    LBF_BUFFER_WRITEF32,
    LBF_BUFFER_READF64,
    LBF_BUFFER_WRITEF64,

    // vector.
    LBF_VECTOR_MAGNITUDE,
    LBF_VECTOR_NORMALIZE,
    LBF_VECTOR_CROSS,
    LBF_VECTOR_DOT,
    LBF_VECTOR_FLOOR,
    LBF_VECTOR_CEIL,
    LBF_VECTOR_ABS,
    LBF_VECTOR_SIGN,
    LBF_VECTOR_CLAMP,
    LBF_VECTOR_MIN,
    LBF_VECTOR_MAX,

    // math.lerp
    LBF_MATH_LERP,

    // vector.lerp
    LBF_VECTOR_LERP,

    // math.
    LBF_MATH_ISNAN,
    LBF_MATH_ISINF,
    LBF_MATH_ISFINITE,

    // integer
    LBF_INTEGER_CREATE,
    LBF_INTEGER_TONUMBER,
    LBF_INTEGER_NEG,
    LBF_INTEGER_ADD,
    LBF_INTEGER_SUB,
    LBF_INTEGER_MUL,
    LBF_INTEGER_DIV,
    LBF_INTEGER_MIN,
    LBF_INTEGER_MAX,
    LBF_INTEGER_REM,
    LBF_INTEGER_IDIV,
    LBF_INTEGER_UDIV,
    LBF_INTEGER_UREM,
    LBF_INTEGER_MOD,
    LBF_INTEGER_CLAMP,
    LBF_INTEGER_BAND,
    LBF_INTEGER_BOR,
    LBF_INTEGER_BNOT,
    LBF_INTEGER_BXOR,
    LBF_INTEGER_LT,
    LBF_INTEGER_LE,
    LBF_INTEGER_ULT,
    LBF_INTEGER_ULE,
    LBF_INTEGER_GT,
    LBF_INTEGER_GE,
    LBF_INTEGER_UGT,
    LBF_INTEGER_UGE,
    LBF_INTEGER_LSHIFT,
    LBF_INTEGER_RSHIFT,
    LBF_INTEGER_ARSHIFT,
    LBF_INTEGER_LROTATE,
    LBF_INTEGER_RROTATE,
    LBF_INTEGER_EXTRACT,
    LBF_INTEGER_BTEST,
    LBF_INTEGER_COUNTRZ,
    LBF_INTEGER_COUNTLZ,
    LBF_INTEGER_BSWAP,

    // buffer.readinteger / buffer.writeinteger (int64_t)
    LBF_BUFFER_READINTEGER,
    LBF_BUFFER_WRITEINTEGER,
}

impl LuauBuiltinFunction {
    pub const LBF_NONE: Self = Self::LBF_NONE;
    pub const LBF_ASSERT: Self = Self::LBF_ASSERT;
    pub const LBF_MATH_ABS: Self = Self::LBF_MATH_ABS;
    pub const LBF_MATH_ACOS: Self = Self::LBF_MATH_ACOS;
    pub const LBF_MATH_ASIN: Self = Self::LBF_MATH_ASIN;
    pub const LBF_MATH_ATAN2: Self = Self::LBF_MATH_ATAN2;
    pub const LBF_MATH_ATAN: Self = Self::LBF_MATH_ATAN;
    pub const LBF_MATH_CEIL: Self = Self::LBF_MATH_CEIL;
    pub const LBF_MATH_COSH: Self = Self::LBF_MATH_COSH;
    pub const LBF_MATH_COS: Self = Self::LBF_MATH_COS;
    pub const LBF_MATH_DEG: Self = Self::LBF_MATH_DEG;
    pub const LBF_MATH_EXP: Self = Self::LBF_MATH_EXP;
    pub const LBF_MATH_FLOOR: Self = Self::LBF_MATH_FLOOR;
    pub const LBF_MATH_FMOD: Self = Self::LBF_MATH_FMOD;
    pub const LBF_MATH_FREXP: Self = Self::LBF_MATH_FREXP;
    pub const LBF_MATH_LDEXP: Self = Self::LBF_MATH_LDEXP;
    pub const LBF_MATH_LOG10: Self = Self::LBF_MATH_LOG10;
    pub const LBF_MATH_LOG: Self = Self::LBF_MATH_LOG;
    pub const LBF_MATH_MAX: Self = Self::LBF_MATH_MAX;
    pub const LBF_MATH_MIN: Self = Self::LBF_MATH_MIN;
    pub const LBF_MATH_MODF: Self = Self::LBF_MATH_MODF;
    pub const LBF_MATH_POW: Self = Self::LBF_MATH_POW;
    pub const LBF_MATH_RAD: Self = Self::LBF_MATH_RAD;
    pub const LBF_MATH_SINH: Self = Self::LBF_MATH_SINH;
    pub const LBF_MATH_SIN: Self = Self::LBF_MATH_SIN;
    pub const LBF_MATH_SQRT: Self = Self::LBF_MATH_SQRT;
    pub const LBF_MATH_TANH: Self = Self::LBF_MATH_TANH;
    pub const LBF_MATH_TAN: Self = Self::LBF_MATH_TAN;
    pub const LBF_BIT32_ARSHIFT: Self = Self::LBF_BIT32_ARSHIFT;
    pub const LBF_BIT32_BAND: Self = Self::LBF_BIT32_BAND;
    pub const LBF_BIT32_BNOT: Self = Self::LBF_BIT32_BNOT;
    pub const LBF_BIT32_BOR: Self = Self::LBF_BIT32_BOR;
    pub const LBF_BIT32_BXOR: Self = Self::LBF_BIT32_BXOR;
    pub const LBF_BIT32_BTEST: Self = Self::LBF_BIT32_BTEST;
    pub const LBF_BIT32_EXTRACT: Self = Self::LBF_BIT32_EXTRACT;
    pub const LBF_BIT32_LROTATE: Self = Self::LBF_BIT32_LROTATE;
    pub const LBF_BIT32_LSHIFT: Self = Self::LBF_BIT32_LSHIFT;
    pub const LBF_BIT32_REPLACE: Self = Self::LBF_BIT32_REPLACE;
    pub const LBF_BIT32_RROTATE: Self = Self::LBF_BIT32_RROTATE;
    pub const LBF_BIT32_RSHIFT: Self = Self::LBF_BIT32_RSHIFT;
    pub const LBF_TYPE: Self = Self::LBF_TYPE;
    pub const LBF_STRING_BYTE: Self = Self::LBF_STRING_BYTE;
    pub const LBF_STRING_CHAR: Self = Self::LBF_STRING_CHAR;
    pub const LBF_STRING_LEN: Self = Self::LBF_STRING_LEN;
    pub const LBF_TYPEOF: Self = Self::LBF_TYPEOF;
    pub const LBF_STRING_SUB: Self = Self::LBF_STRING_SUB;
    pub const LBF_MATH_CLAMP: Self = Self::LBF_MATH_CLAMP;
    pub const LBF_MATH_SIGN: Self = Self::LBF_MATH_SIGN;
    pub const LBF_MATH_ROUND: Self = Self::LBF_MATH_ROUND;
    pub const LBF_RAWSET: Self = Self::LBF_RAWSET;
    pub const LBF_RAWGET: Self = Self::LBF_RAWGET;
    pub const LBF_RAWEQUAL: Self = Self::LBF_RAWEQUAL;
    pub const LBF_TABLE_INSERT: Self = Self::LBF_TABLE_INSERT;
    pub const LBF_TABLE_UNPACK: Self = Self::LBF_TABLE_UNPACK;
    pub const LBF_VECTOR: Self = Self::LBF_VECTOR;
    pub const LBF_BIT32_COUNTLZ: Self = Self::LBF_BIT32_COUNTLZ;
    pub const LBF_BIT32_COUNTRZ: Self = Self::LBF_BIT32_COUNTRZ;
    pub const LBF_SELECT_VARARG: Self = Self::LBF_SELECT_VARARG;
    pub const LBF_RAWLEN: Self = Self::LBF_RAWLEN;
    pub const LBF_BIT32_EXTRACTK: Self = Self::LBF_BIT32_EXTRACTK;
    pub const LBF_GETMETATABLE: Self = Self::LBF_GETMETATABLE;
    pub const LBF_SETMETATABLE: Self = Self::LBF_SETMETATABLE;
    pub const LBF_TONUMBER: Self = Self::LBF_TONUMBER;
    pub const LBF_TOSTRING: Self = Self::LBF_TOSTRING;
    pub const LBF_BIT32_BYTESWAP: Self = Self::LBF_BIT32_BYTESWAP;
    pub const LBF_BUFFER_READI8: Self = Self::LBF_BUFFER_READI8;
    pub const LBF_BUFFER_READU8: Self = Self::LBF_BUFFER_READU8;
    pub const LBF_BUFFER_WRITEU8: Self = Self::LBF_BUFFER_WRITEU8;
    pub const LBF_BUFFER_READI16: Self = Self::LBF_BUFFER_READI16;
    pub const LBF_BUFFER_READU16: Self = Self::LBF_BUFFER_READU16;
    pub const LBF_BUFFER_WRITEU16: Self = Self::LBF_BUFFER_WRITEU16;
    pub const LBF_BUFFER_READI32: Self = Self::LBF_BUFFER_READI32;
    pub const LBF_BUFFER_READU32: Self = Self::LBF_BUFFER_READU32;
    pub const LBF_BUFFER_WRITEU32: Self = Self::LBF_BUFFER_WRITEU32;
    pub const LBF_BUFFER_READF32: Self = Self::LBF_BUFFER_READF32;
    pub const LBF_BUFFER_WRITEF32: Self = Self::LBF_BUFFER_WRITEF32;
    pub const LBF_BUFFER_READF64: Self = Self::LBF_BUFFER_READF64;
    pub const LBF_BUFFER_WRITEF64: Self = Self::LBF_BUFFER_WRITEF64;
    pub const LBF_VECTOR_MAGNITUDE: Self = Self::LBF_VECTOR_MAGNITUDE;
    pub const LBF_VECTOR_NORMALIZE: Self = Self::LBF_VECTOR_NORMALIZE;
    pub const LBF_VECTOR_CROSS: Self = Self::LBF_VECTOR_CROSS;
    pub const LBF_VECTOR_DOT: Self = Self::LBF_VECTOR_DOT;
    pub const LBF_VECTOR_FLOOR: Self = Self::LBF_VECTOR_FLOOR;
    pub const LBF_VECTOR_CEIL: Self = Self::LBF_VECTOR_CEIL;
    pub const LBF_VECTOR_ABS: Self = Self::LBF_VECTOR_ABS;
    pub const LBF_VECTOR_SIGN: Self = Self::LBF_VECTOR_SIGN;
    pub const LBF_VECTOR_CLAMP: Self = Self::LBF_VECTOR_CLAMP;
    pub const LBF_VECTOR_MIN: Self = Self::LBF_VECTOR_MIN;
    pub const LBF_VECTOR_MAX: Self = Self::LBF_VECTOR_MAX;
    pub const LBF_MATH_LERP: Self = Self::LBF_MATH_LERP;
    pub const LBF_VECTOR_LERP: Self = Self::LBF_VECTOR_LERP;
    pub const LBF_MATH_ISNAN: Self = Self::LBF_MATH_ISNAN;
    pub const LBF_MATH_ISINF: Self = Self::LBF_MATH_ISINF;
    pub const LBF_MATH_ISFINITE: Self = Self::LBF_MATH_ISFINITE;
    pub const LBF_INTEGER_CREATE: Self = Self::LBF_INTEGER_CREATE;
    pub const LBF_INTEGER_TONUMBER: Self = Self::LBF_INTEGER_TONUMBER;
    pub const LBF_INTEGER_NEG: Self = Self::LBF_INTEGER_NEG;
    pub const LBF_INTEGER_ADD: Self = Self::LBF_INTEGER_ADD;
    pub const LBF_INTEGER_SUB: Self = Self::LBF_INTEGER_SUB;
    pub const LBF_INTEGER_MUL: Self = Self::LBF_INTEGER_MUL;
    pub const LBF_INTEGER_DIV: Self = Self::LBF_INTEGER_DIV;
    pub const LBF_INTEGER_MIN: Self = Self::LBF_INTEGER_MIN;
    pub const LBF_INTEGER_MAX: Self = Self::LBF_INTEGER_MAX;
    pub const LBF_INTEGER_REM: Self = Self::LBF_INTEGER_REM;
    pub const LBF_INTEGER_IDIV: Self = Self::LBF_INTEGER_IDIV;
    pub const LBF_INTEGER_UDIV: Self = Self::LBF_INTEGER_UDIV;
    pub const LBF_INTEGER_UREM: Self = Self::LBF_INTEGER_UREM;
    pub const LBF_INTEGER_MOD: Self = Self::LBF_INTEGER_MOD;
    pub const LBF_INTEGER_CLAMP: Self = Self::LBF_INTEGER_CLAMP;
    pub const LBF_INTEGER_BAND: Self = Self::LBF_INTEGER_BAND;
    pub const LBF_INTEGER_BOR: Self = Self::LBF_INTEGER_BOR;
    pub const LBF_INTEGER_BNOT: Self = Self::LBF_INTEGER_BNOT;
    pub const LBF_INTEGER_BXOR: Self = Self::LBF_INTEGER_BXOR;
    pub const LBF_INTEGER_LT: Self = Self::LBF_INTEGER_LT;
    pub const LBF_INTEGER_LE: Self = Self::LBF_INTEGER_LE;
    pub const LBF_INTEGER_ULT: Self = Self::LBF_INTEGER_ULT;
    pub const LBF_INTEGER_ULE: Self = Self::LBF_INTEGER_ULE;
    pub const LBF_INTEGER_GT: Self = Self::LBF_INTEGER_GT;
    pub const LBF_INTEGER_GE: Self = Self::LBF_INTEGER_GE;
    pub const LBF_INTEGER_UGT: Self = Self::LBF_INTEGER_UGT;
    pub const LBF_INTEGER_UGE: Self = Self::LBF_INTEGER_UGE;
    pub const LBF_INTEGER_LSHIFT: Self = Self::LBF_INTEGER_LSHIFT;
    pub const LBF_INTEGER_RSHIFT: Self = Self::LBF_INTEGER_RSHIFT;
    pub const LBF_INTEGER_ARSHIFT: Self = Self::LBF_INTEGER_ARSHIFT;
    pub const LBF_INTEGER_LROTATE: Self = Self::LBF_INTEGER_LROTATE;
    pub const LBF_INTEGER_RROTATE: Self = Self::LBF_INTEGER_RROTATE;
    pub const LBF_INTEGER_EXTRACT: Self = Self::LBF_INTEGER_EXTRACT;
    pub const LBF_INTEGER_BTEST: Self = Self::LBF_INTEGER_BTEST;
    pub const LBF_INTEGER_COUNTRZ: Self = Self::LBF_INTEGER_COUNTRZ;
    pub const LBF_INTEGER_COUNTLZ: Self = Self::LBF_INTEGER_COUNTLZ;
    pub const LBF_INTEGER_BSWAP: Self = Self::LBF_INTEGER_BSWAP;
    pub const LBF_BUFFER_READINTEGER: Self = Self::LBF_BUFFER_READINTEGER;
    pub const LBF_BUFFER_WRITEINTEGER: Self = Self::LBF_BUFFER_WRITEINTEGER;
}
