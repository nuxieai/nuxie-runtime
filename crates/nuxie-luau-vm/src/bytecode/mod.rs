//! Luau bytecode: the instruction encoding, the loaded prototype, the
//! loader (`load.rs`) and the verifier (`verify.rs`, plan decision D7).
//!
//! **Versions.** This VM admits bytecode versions 3 through 11 (type-info
//! versions 1 through 3): the device runtime's admission set (seam doc
//! section 4). The editor emits 6 and 7; this repository's compiler emits 11
//! with the product flag profile (call feedback, `CALLFB`) and 7 without it.
//! Versions 12 (cost model) and 13 (double-precision vector constants) and
//! the experimental 100 (user-defined classes) are rejected with
//! [`LoadError::UnsupportedVersion`](crate::LoadError): no producer the
//! product uses emits them.

pub(crate) mod load;
pub(crate) mod verify;

use crate::value::{ProtoId, StrId, Value};

pub const VERSION_MIN: u8 = 3;
pub const VERSION_MAX: u8 = 11;

/// Opcode numbers (C++ `LuauOpcode`, in declaration order).
pub mod op {
    pub const NOP: u8 = 0;
    pub const BREAK: u8 = 1;
    pub const LOADNIL: u8 = 2;
    pub const LOADB: u8 = 3;
    pub const LOADN: u8 = 4;
    pub const LOADK: u8 = 5;
    pub const MOVE: u8 = 6;
    pub const GETGLOBAL: u8 = 7;
    pub const SETGLOBAL: u8 = 8;
    pub const GETUPVAL: u8 = 9;
    pub const SETUPVAL: u8 = 10;
    pub const CLOSEUPVALS: u8 = 11;
    pub const GETIMPORT: u8 = 12;
    pub const GETTABLE: u8 = 13;
    pub const SETTABLE: u8 = 14;
    pub const GETTABLEKS: u8 = 15;
    pub const SETTABLEKS: u8 = 16;
    pub const GETTABLEN: u8 = 17;
    pub const SETTABLEN: u8 = 18;
    pub const NEWCLOSURE: u8 = 19;
    pub const NAMECALL: u8 = 20;
    pub const CALL: u8 = 21;
    pub const RETURN: u8 = 22;
    pub const JUMP: u8 = 23;
    pub const JUMPBACK: u8 = 24;
    pub const JUMPIF: u8 = 25;
    pub const JUMPIFNOT: u8 = 26;
    pub const JUMPIFEQ: u8 = 27;
    pub const JUMPIFLE: u8 = 28;
    pub const JUMPIFLT: u8 = 29;
    pub const JUMPIFNOTEQ: u8 = 30;
    pub const JUMPIFNOTLE: u8 = 31;
    pub const JUMPIFNOTLT: u8 = 32;
    pub const ADD: u8 = 33;
    pub const SUB: u8 = 34;
    pub const MUL: u8 = 35;
    pub const DIV: u8 = 36;
    pub const MOD: u8 = 37;
    pub const POW: u8 = 38;
    pub const ADDK: u8 = 39;
    pub const SUBK: u8 = 40;
    pub const MULK: u8 = 41;
    pub const DIVK: u8 = 42;
    pub const MODK: u8 = 43;
    pub const POWK: u8 = 44;
    pub const AND: u8 = 45;
    pub const OR: u8 = 46;
    pub const ANDK: u8 = 47;
    pub const ORK: u8 = 48;
    pub const CONCAT: u8 = 49;
    pub const NOT: u8 = 50;
    pub const MINUS: u8 = 51;
    pub const LENGTH: u8 = 52;
    pub const NEWTABLE: u8 = 53;
    pub const DUPTABLE: u8 = 54;
    pub const SETLIST: u8 = 55;
    pub const FORNPREP: u8 = 56;
    pub const FORNLOOP: u8 = 57;
    pub const FORGLOOP: u8 = 58;
    pub const FORGPREP_INEXT: u8 = 59;
    pub const FASTCALL3: u8 = 60;
    pub const FORGPREP_NEXT: u8 = 61;
    pub const NATIVECALL: u8 = 62;
    pub const GETVARARGS: u8 = 63;
    pub const DUPCLOSURE: u8 = 64;
    pub const PREPVARARGS: u8 = 65;
    pub const LOADKX: u8 = 66;
    pub const JUMPX: u8 = 67;
    pub const FASTCALL: u8 = 68;
    pub const COVERAGE: u8 = 69;
    pub const CAPTURE: u8 = 70;
    pub const SUBRK: u8 = 71;
    pub const DIVRK: u8 = 72;
    pub const FASTCALL1: u8 = 73;
    pub const FASTCALL2: u8 = 74;
    pub const FASTCALL2K: u8 = 75;
    pub const FORGPREP: u8 = 76;
    pub const JUMPXEQKNIL: u8 = 77;
    pub const JUMPXEQKB: u8 = 78;
    pub const JUMPXEQKN: u8 = 79;
    pub const JUMPXEQKS: u8 = 80;
    pub const IDIV: u8 = 81;
    pub const IDIVK: u8 = 82;
    pub const GETUDATAKS: u8 = 83;
    pub const SETUDATAKS: u8 = 84;
    pub const NAMECALLUDATA: u8 = 85;
    pub const NEWCLASSMEMBER: u8 = 86;
    pub const CALLFB: u8 = 87;
    pub const CMPPROTO: u8 = 88;
    pub const NEWCLASS: u8 = 89;
    /// The number of opcodes (C++ `LOP__COUNT`): every byte below it is one.
    pub const COUNT: u8 = 90;

    /// C++ enumerator names, indexed by opcode number.
    pub(super) const NAMES: [&str; COUNT as usize] = [
        "LOP_NOP",
        "LOP_BREAK",
        "LOP_LOADNIL",
        "LOP_LOADB",
        "LOP_LOADN",
        "LOP_LOADK",
        "LOP_MOVE",
        "LOP_GETGLOBAL",
        "LOP_SETGLOBAL",
        "LOP_GETUPVAL",
        "LOP_SETUPVAL",
        "LOP_CLOSEUPVALS",
        "LOP_GETIMPORT",
        "LOP_GETTABLE",
        "LOP_SETTABLE",
        "LOP_GETTABLEKS",
        "LOP_SETTABLEKS",
        "LOP_GETTABLEN",
        "LOP_SETTABLEN",
        "LOP_NEWCLOSURE",
        "LOP_NAMECALL",
        "LOP_CALL",
        "LOP_RETURN",
        "LOP_JUMP",
        "LOP_JUMPBACK",
        "LOP_JUMPIF",
        "LOP_JUMPIFNOT",
        "LOP_JUMPIFEQ",
        "LOP_JUMPIFLE",
        "LOP_JUMPIFLT",
        "LOP_JUMPIFNOTEQ",
        "LOP_JUMPIFNOTLE",
        "LOP_JUMPIFNOTLT",
        "LOP_ADD",
        "LOP_SUB",
        "LOP_MUL",
        "LOP_DIV",
        "LOP_MOD",
        "LOP_POW",
        "LOP_ADDK",
        "LOP_SUBK",
        "LOP_MULK",
        "LOP_DIVK",
        "LOP_MODK",
        "LOP_POWK",
        "LOP_AND",
        "LOP_OR",
        "LOP_ANDK",
        "LOP_ORK",
        "LOP_CONCAT",
        "LOP_NOT",
        "LOP_MINUS",
        "LOP_LENGTH",
        "LOP_NEWTABLE",
        "LOP_DUPTABLE",
        "LOP_SETLIST",
        "LOP_FORNPREP",
        "LOP_FORNLOOP",
        "LOP_FORGLOOP",
        "LOP_FORGPREP_INEXT",
        "LOP_FASTCALL3",
        "LOP_FORGPREP_NEXT",
        "LOP_NATIVECALL",
        "LOP_GETVARARGS",
        "LOP_DUPCLOSURE",
        "LOP_PREPVARARGS",
        "LOP_LOADKX",
        "LOP_JUMPX",
        "LOP_FASTCALL",
        "LOP_COVERAGE",
        "LOP_CAPTURE",
        "LOP_SUBRK",
        "LOP_DIVRK",
        "LOP_FASTCALL1",
        "LOP_FASTCALL2",
        "LOP_FASTCALL2K",
        "LOP_FORGPREP",
        "LOP_JUMPXEQKNIL",
        "LOP_JUMPXEQKB",
        "LOP_JUMPXEQKN",
        "LOP_JUMPXEQKS",
        "LOP_IDIV",
        "LOP_IDIVK",
        "LOP_GETUDATAKS",
        "LOP_SETUDATAKS",
        "LOP_NAMECALLUDATA",
        "LOP_NEWCLASSMEMBER",
        "LOP_CALLFB",
        "LOP_CMPPROTO",
        "LOP_NEWCLASS",
    ];

    /// C++ `getOpLength`: instructions followed by an AUX word.
    pub(crate) fn has_aux(op: u8) -> bool {
        matches!(
            op,
            GETGLOBAL
                | SETGLOBAL
                | GETIMPORT
                | GETTABLEKS
                | SETTABLEKS
                | NAMECALL
                | JUMPIFEQ
                | JUMPIFLE
                | JUMPIFLT
                | JUMPIFNOTEQ
                | JUMPIFNOTLE
                | JUMPIFNOTLT
                | NEWTABLE
                | SETLIST
                | FORGLOOP
                | LOADKX
                | FASTCALL2
                | FASTCALL2K
                | FASTCALL3
                | JUMPXEQKNIL
                | JUMPXEQKB
                | JUMPXEQKN
                | JUMPXEQKS
                | GETUDATAKS
                | SETUDATAKS
                | NAMECALLUDATA
                | NEWCLASSMEMBER
                | CALLFB
                | CMPPROTO
                | NEWCLASS
        )
    }
}

/// An opcode number below [`op::COUNT`]: the checked conversion from an
/// instruction's op byte (C++ casts the byte to `LuauOpcode` unchecked).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Opcode(u8);

impl Opcode {
    /// The opcode numbered `byte`, or `None` if no opcode has that number.
    pub fn new(byte: u8) -> Option<Self> {
        (byte < op::COUNT).then_some(Self(byte))
    }

    /// The opcode number (one of the [`op`] constants).
    pub fn number(self) -> u8 {
        self.0
    }

    /// Instruction words, 1 or 2 with an AUX word (C++ `getOpLength`).
    pub fn length(self) -> usize {
        if op::has_aux(self.0) { 2 } else { 1 }
    }

    /// The C++ enumerator name, such as `LOP_GETTABLEKS`.
    pub fn name(self) -> &'static str {
        op::NAMES.get(usize::from(self.0)).copied().unwrap_or_default()
    }
}

impl std::fmt::Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// Tags in the bytecode stream other than opcodes (C++ `LuauBytecodeTag`,
/// `LuauBytecodeType`, `LuauFeedbackType`): the values this VM's loader reads.
pub mod tag {
    /// Type-info encoding versions loaded (C++ `LBC_TYPE_VERSION_MIN/MAX`).
    pub const TYPE_VERSION_MIN: u8 = 1;
    pub const TYPE_VERSION_MAX: u8 = 3;

    /// Constant table entry kinds (C++ `LBC_CONSTANT_*`).
    pub const CONSTANT_NIL: u8 = 0;
    pub const CONSTANT_BOOLEAN: u8 = 1;
    pub const CONSTANT_NUMBER: u8 = 2;
    pub const CONSTANT_STRING: u8 = 3;
    pub const CONSTANT_IMPORT: u8 = 4;
    pub const CONSTANT_TABLE: u8 = 5;
    pub const CONSTANT_CLOSURE: u8 = 6;
    pub const CONSTANT_VECTOR: u8 = 7;
    pub const CONSTANT_TABLE_WITH_CONSTANTS: u8 = 8;
    pub const CONSTANT_INTEGER: u8 = 9;
    pub const CONSTANT_CLASS_SHAPE: u8 = 10;
    pub const CONSTANT_VECTORD: u8 = 11;

    /// The function type tag that opens a version-1 type-info blob
    /// (C++ `LBC_TYPE_FUNCTION`).
    pub const TYPE_FUNCTION: u8 = 5;

    /// The one feedback slot kind of version 11 (C++ `LFT_CALLTARGET`).
    pub const FEEDBACK_CALL_TARGET: u8 = 0;
}

/// Instruction field accessors (C++ `LUAU_INSN_*`).
#[inline]
pub(crate) fn insn_op(insn: u32) -> u8 {
    (insn & 0xff) as u8
}
#[inline]
pub(crate) fn insn_a(insn: u32) -> u8 {
    ((insn >> 8) & 0xff) as u8
}
#[inline]
pub(crate) fn insn_b(insn: u32) -> u8 {
    ((insn >> 16) & 0xff) as u8
}
#[inline]
pub(crate) fn insn_c(insn: u32) -> u8 {
    (insn >> 24) as u8
}
#[inline]
pub(crate) fn insn_d(insn: u32) -> i32 {
    (insn as i32) >> 16
}
#[inline]
pub(crate) fn insn_e(insn: u32) -> i32 {
    (insn as i32) >> 8
}

/// Line information: C++ `lineinfo` (a byte offset per instruction) and
/// `abslineinfo` (a base line per `2^linegaplog2` instructions).
#[derive(Clone, Debug, Default)]
pub(crate) struct LineInfo {
    pub(crate) gaplog2: u32,
    pub(crate) offsets: Box<[u8]>,
    pub(crate) bases: Box<[i32]>,
}

impl LineInfo {
    /// C++ `luaG_getline`.
    pub(crate) fn line(&self, pc: usize) -> Option<i32> {
        let base = self.bases.get(pc.checked_shr(self.gaplog2)?)?;
        let offset = self.offsets.get(pc)?;
        Some(base.wrapping_add(i32::from(*offset)))
    }
}

/// A function prototype, immutable once loaded and verified.
#[derive(Clone, Debug, Default)]
pub(crate) struct Proto {
    pub(crate) maxstacksize: u8,
    pub(crate) numparams: u8,
    pub(crate) nups: u8,
    pub(crate) is_vararg: bool,
    pub(crate) code: Box<[u32]>,
    pub(crate) k: Box<[Value]>,
    pub(crate) children: Box<[ProtoId]>,
    /// The chunk name the proto was loaded under (C++ `source`).
    pub(crate) source: Option<StrId>,
    pub(crate) debugname: Option<StrId>,
    pub(crate) linedefined: u32,
    pub(crate) lineinfo: Option<LineInfo>,
}

impl Proto {
    /// The source line of the instruction at `pc`, or 0 without line info
    /// (C++ `currentline` returns -1 there; callers only test `> 0`).
    pub(crate) fn line(&self, pc: usize) -> i32 {
        self.lineinfo
            .as_ref()
            .and_then(|info| info.line(pc))
            .unwrap_or(0)
    }
}
