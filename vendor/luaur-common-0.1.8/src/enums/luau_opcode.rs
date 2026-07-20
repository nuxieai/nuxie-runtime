#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum LuauOpcode {
    LOP_NOP,
    LOP_BREAK,
    LOP_LOADNIL,
    LOP_LOADB,
    LOP_LOADN,
    LOP_LOADK,
    LOP_MOVE,
    LOP_GETGLOBAL,
    LOP_SETGLOBAL,
    LOP_GETUPVAL,
    LOP_SETUPVAL,
    LOP_CLOSEUPVALS,
    LOP_GETIMPORT,
    LOP_GETTABLE,
    LOP_SETTABLE,
    LOP_GETTABLEKS,
    LOP_SETTABLEKS,
    LOP_GETTABLEN,
    LOP_SETTABLEN,
    LOP_NEWCLOSURE,
    LOP_NAMECALL,
    LOP_CALL,
    LOP_RETURN,
    LOP_JUMP,
    LOP_JUMPBACK,
    LOP_JUMPIF,
    LOP_JUMPIFNOT,
    LOP_JUMPIFEQ,
    LOP_JUMPIFLE,
    LOP_JUMPIFLT,
    LOP_JUMPIFNOTEQ,
    LOP_JUMPIFNOTLE,
    LOP_JUMPIFNOTLT,
    LOP_ADD,
    LOP_SUB,
    LOP_MUL,
    LOP_DIV,
    LOP_MOD,
    LOP_POW,
    LOP_ADDK,
    LOP_SUBK,
    LOP_MULK,
    LOP_DIVK,
    LOP_MODK,
    LOP_POWK,
    LOP_AND,
    LOP_OR,
    LOP_ANDK,
    LOP_ORK,
    LOP_CONCAT,
    LOP_NOT,
    LOP_MINUS,
    LOP_LENGTH,
    LOP_NEWTABLE,
    LOP_DUPTABLE,
    LOP_SETLIST,
    LOP_FORNPREP,
    LOP_FORNLOOP,
    LOP_FORGLOOP,
    LOP_FORGPREP_INEXT,
    LOP_FASTCALL3,
    LOP_FORGPREP_NEXT,
    LOP_NATIVECALL,
    LOP_GETVARARGS,
    LOP_DUPCLOSURE,
    LOP_PREPVARARGS,
    LOP_LOADKX,
    LOP_JUMPX,
    LOP_FASTCALL,
    LOP_COVERAGE,
    LOP_CAPTURE,
    LOP_SUBRK,
    LOP_DIVRK,
    LOP_FASTCALL1,
    LOP_FASTCALL2,
    LOP_FASTCALL2K,
    LOP_FORGPREP,
    LOP_JUMPXEQKNIL,
    LOP_JUMPXEQKB,
    LOP_JUMPXEQKN,
    LOP_JUMPXEQKS,
    LOP_IDIV,
    LOP_IDIVK,
    LOP_GETUDATAKS,
    LOP_SETUDATAKS,
    LOP_NAMECALLUDATA,
    LOP_NEWCLASSMEMBER,
    LOP_CALLFB,
    LOP_CMPPROTO,
    LOP__COUNT,
}

impl LuauOpcode {
    pub const LOP_NOP: LuauOpcode = LuauOpcode::LOP_NOP;
    pub const LOP_BREAK: LuauOpcode = LuauOpcode::LOP_BREAK;
    pub const LOP_LOADNIL: LuauOpcode = LuauOpcode::LOP_LOADNIL;
    pub const LOP_LOADB: LuauOpcode = LuauOpcode::LOP_LOADB;
    pub const LOP_LOADN: LuauOpcode = LuauOpcode::LOP_LOADN;
    pub const LOP_LOADK: LuauOpcode = LuauOpcode::LOP_LOADK;
    pub const LOP_MOVE: LuauOpcode = LuauOpcode::LOP_MOVE;
    pub const LOP_GETGLOBAL: LuauOpcode = LuauOpcode::LOP_GETGLOBAL;
    pub const LOP_SETGLOBAL: LuauOpcode = LuauOpcode::LOP_SETGLOBAL;
    pub const LOP_GETUPVAL: LuauOpcode = LuauOpcode::LOP_GETUPVAL;
    pub const LOP_SETUPVAL: LuauOpcode = LuauOpcode::LOP_SETUPVAL;
    pub const LOP_CLOSEUPVALS: LuauOpcode = LuauOpcode::LOP_CLOSEUPVALS;
    pub const LOP_GETIMPORT: LuauOpcode = LuauOpcode::LOP_GETIMPORT;
    pub const LOP_GETTABLE: LuauOpcode = LuauOpcode::LOP_GETTABLE;
    pub const LOP_SETTABLE: LuauOpcode = LuauOpcode::LOP_SETTABLE;
    pub const LOP_GETTABLEKS: LuauOpcode = LuauOpcode::LOP_GETTABLEKS;
    pub const LOP_SETTABLEKS: LuauOpcode = LuauOpcode::LOP_SETTABLEKS;
    pub const LOP_GETTABLEN: LuauOpcode = LuauOpcode::LOP_GETTABLEN;
    pub const LOP_SETTABLEN: LuauOpcode = LuauOpcode::LOP_SETTABLEN;
    pub const LOP_NEWCLOSURE: LuauOpcode = LuauOpcode::LOP_NEWCLOSURE;
    pub const LOP_NAMECALL: LuauOpcode = LuauOpcode::LOP_NAMECALL;
    pub const LOP_CALL: LuauOpcode = LuauOpcode::LOP_CALL;
    pub const LOP_RETURN: LuauOpcode = LuauOpcode::LOP_RETURN;
    pub const LOP_JUMP: LuauOpcode = LuauOpcode::LOP_JUMP;
    pub const LOP_JUMPBACK: LuauOpcode = LuauOpcode::LOP_JUMPBACK;
    pub const LOP_JUMPIF: LuauOpcode = LuauOpcode::LOP_JUMPIF;
    pub const LOP_JUMPIFNOT: LuauOpcode = LuauOpcode::LOP_JUMPIFNOT;
    pub const LOP_JUMPIFEQ: LuauOpcode = LuauOpcode::LOP_JUMPIFEQ;
    pub const LOP_JUMPIFLE: LuauOpcode = LuauOpcode::LOP_JUMPIFLE;
    pub const LOP_JUMPIFLT: LuauOpcode = LuauOpcode::LOP_JUMPIFLT;
    pub const LOP_JUMPIFNOTEQ: LuauOpcode = LuauOpcode::LOP_JUMPIFNOTEQ;
    pub const LOP_JUMPIFNOTLE: LuauOpcode = LuauOpcode::LOP_JUMPIFNOTLE;
    pub const LOP_JUMPIFNOTLT: LuauOpcode = LuauOpcode::LOP_JUMPIFNOTLT;
    pub const LOP_ADD: LuauOpcode = LuauOpcode::LOP_ADD;
    pub const LOP_SUB: LuauOpcode = LuauOpcode::LOP_SUB;
    pub const LOP_MUL: LuauOpcode = LuauOpcode::LOP_MUL;
    pub const LOP_DIV: LuauOpcode = LuauOpcode::LOP_DIV;
    pub const LOP_MOD: LuauOpcode = LuauOpcode::LOP_MOD;
    pub const LOP_POW: LuauOpcode = LuauOpcode::LOP_POW;
    pub const LOP_ADDK: LuauOpcode = LuauOpcode::LOP_ADDK;
    pub const LOP_SUBK: LuauOpcode = LuauOpcode::LOP_SUBK;
    pub const LOP_MULK: LuauOpcode = LuauOpcode::LOP_MULK;
    pub const LOP_DIVK: LuauOpcode = LuauOpcode::LOP_DIVK;
    pub const LOP_MODK: LuauOpcode = LuauOpcode::LOP_MODK;
    pub const LOP_POWK: LuauOpcode = LuauOpcode::LOP_POWK;
    pub const LOP_AND: LuauOpcode = LuauOpcode::LOP_AND;
    pub const LOP_OR: LuauOpcode = LuauOpcode::LOP_OR;
    pub const LOP_ANDK: LuauOpcode = LuauOpcode::LOP_ANDK;
    pub const LOP_ORK: LuauOpcode = LuauOpcode::LOP_ORK;
    pub const LOP_CONCAT: LuauOpcode = LuauOpcode::LOP_CONCAT;
    pub const LOP_NOT: LuauOpcode = LuauOpcode::LOP_NOT;
    pub const LOP_MINUS: LuauOpcode = LuauOpcode::LOP_MINUS;
    pub const LOP_LENGTH: LuauOpcode = LuauOpcode::LOP_LENGTH;
    pub const LOP_NEWTABLE: LuauOpcode = LuauOpcode::LOP_NEWTABLE;
    pub const LOP_DUPTABLE: LuauOpcode = LuauOpcode::LOP_DUPTABLE;
    pub const LOP_SETLIST: LuauOpcode = LuauOpcode::LOP_SETLIST;
    pub const LOP_FORNPREP: LuauOpcode = LuauOpcode::LOP_FORNPREP;
    pub const LOP_FORNLOOP: LuauOpcode = LuauOpcode::LOP_FORNLOOP;
    pub const LOP_FORGLOOP: LuauOpcode = LuauOpcode::LOP_FORGLOOP;
    pub const LOP_FORGPREP_INEXT: LuauOpcode = LuauOpcode::LOP_FORGPREP_INEXT;
    pub const LOP_FASTCALL3: LuauOpcode = LuauOpcode::LOP_FASTCALL3;
    pub const LOP_FORGPREP_NEXT: LuauOpcode = LuauOpcode::LOP_FORGPREP_NEXT;
    pub const LOP_NATIVECALL: LuauOpcode = LuauOpcode::LOP_NATIVECALL;
    pub const LOP_GETVARARGS: LuauOpcode = LuauOpcode::LOP_GETVARARGS;
    pub const LOP_DUPCLOSURE: LuauOpcode = LuauOpcode::LOP_DUPCLOSURE;
    pub const LOP_PREPVARARGS: LuauOpcode = LuauOpcode::LOP_PREPVARARGS;
    pub const LOP_LOADKX: LuauOpcode = LuauOpcode::LOP_LOADKX;
    pub const LOP_JUMPX: LuauOpcode = LuauOpcode::LOP_JUMPX;
    pub const LOP_FASTCALL: LuauOpcode = LuauOpcode::LOP_FASTCALL;
    pub const LOP_COVERAGE: LuauOpcode = LuauOpcode::LOP_COVERAGE;
    pub const LOP_CAPTURE: LuauOpcode = LuauOpcode::LOP_CAPTURE;
    pub const LOP_SUBRK: LuauOpcode = LuauOpcode::LOP_SUBRK;
    pub const LOP_DIVRK: LuauOpcode = LuauOpcode::LOP_DIVRK;
    pub const LOP_FASTCALL1: LuauOpcode = LuauOpcode::LOP_FASTCALL1;
    pub const LOP_FASTCALL2: LuauOpcode = LuauOpcode::LOP_FASTCALL2;
    pub const LOP_FASTCALL2K: LuauOpcode = LuauOpcode::LOP_FASTCALL2K;
    pub const LOP_FORGPREP: LuauOpcode = LuauOpcode::LOP_FORGPREP;
    pub const LOP_JUMPXEQKNIL: LuauOpcode = LuauOpcode::LOP_JUMPXEQKNIL;
    pub const LOP_JUMPXEQKB: LuauOpcode = LuauOpcode::LOP_JUMPXEQKB;
    pub const LOP_JUMPXEQKN: LuauOpcode = LuauOpcode::LOP_JUMPXEQKN;
    pub const LOP_JUMPXEQKS: LuauOpcode = LuauOpcode::LOP_JUMPXEQKS;
    pub const LOP_IDIV: LuauOpcode = LuauOpcode::LOP_IDIV;
    pub const LOP_IDIVK: LuauOpcode = LuauOpcode::LOP_IDIVK;
    pub const LOP_GETUDATAKS: LuauOpcode = LuauOpcode::LOP_GETUDATAKS;
    pub const LOP_SETUDATAKS: LuauOpcode = LuauOpcode::LOP_SETUDATAKS;
    pub const LOP_NAMECALLUDATA: LuauOpcode = LuauOpcode::LOP_NAMECALLUDATA;
    pub const LOP_NEWCLASSMEMBER: LuauOpcode = LuauOpcode::LOP_NEWCLASSMEMBER;
    pub const LOP_CALLFB: LuauOpcode = LuauOpcode::LOP_CALLFB;
    pub const LOP_CMPPROTO: LuauOpcode = LuauOpcode::LOP_CMPPROTO;
    pub const LOP__COUNT: LuauOpcode = LuauOpcode::LOP__COUNT;
}

impl From<u8> for LuauOpcode {
    /// C++ casts the instruction's op byte straight to `LuauOpcode`
    /// (`LuauOpcode(LUAU_INSN_OP(insn))`). Valid bytecode only carries
    /// in-range opcodes; `repr(u8)` makes the layout identical.
    fn from(v: u8) -> Self {
        unsafe { core::mem::transmute(v) }
    }
}
