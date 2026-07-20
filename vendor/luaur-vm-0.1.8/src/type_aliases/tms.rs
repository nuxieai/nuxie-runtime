#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TMS {
    TM_INDEX,
    TM_NEWINDEX,
    TM_MODE,
    TM_NAMECALL,
    TM_CALL,
    TM_ITER,
    TM_LEN,

    TM_EQ, // last tag method with `fast' access

    TM_ADD,
    TM_SUB,
    TM_MUL,
    TM_DIV,
    TM_IDIV,
    TM_MOD,
    TM_POW,
    TM_UNM,

    TM_LT,
    TM_LE,
    TM_CONCAT,
    TM_TYPE,
    TM_METATABLE,

    TM_N, // number of elements in the enum
}

#[allow(non_camel_case_types)]
pub type tms = TMS;
