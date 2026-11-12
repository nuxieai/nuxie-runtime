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

/// Opcode numbers (luaur-common `LuauOpcode`, in declaration order).
pub(crate) mod op {
    pub(crate) const NOP: u8 = 0;
    pub(crate) const BREAK: u8 = 1;
    pub(crate) const LOADNIL: u8 = 2;
    pub(crate) const LOADB: u8 = 3;
    pub(crate) const LOADN: u8 = 4;
    pub(crate) const LOADK: u8 = 5;
    pub(crate) const MOVE: u8 = 6;
    pub(crate) const GETGLOBAL: u8 = 7;
    pub(crate) const SETGLOBAL: u8 = 8;
    pub(crate) const GETUPVAL: u8 = 9;
    pub(crate) const SETUPVAL: u8 = 10;
    pub(crate) const CLOSEUPVALS: u8 = 11;
    pub(crate) const GETIMPORT: u8 = 12;
    pub(crate) const GETTABLE: u8 = 13;
    pub(crate) const SETTABLE: u8 = 14;
    pub(crate) const GETTABLEKS: u8 = 15;
    pub(crate) const SETTABLEKS: u8 = 16;
    pub(crate) const GETTABLEN: u8 = 17;
    pub(crate) const SETTABLEN: u8 = 18;
    pub(crate) const NEWCLOSURE: u8 = 19;
    pub(crate) const NAMECALL: u8 = 20;
    pub(crate) const CALL: u8 = 21;
    pub(crate) const RETURN: u8 = 22;
    pub(crate) const JUMP: u8 = 23;
    pub(crate) const JUMPBACK: u8 = 24;
    pub(crate) const JUMPIF: u8 = 25;
    pub(crate) const JUMPIFNOT: u8 = 26;
    pub(crate) const JUMPIFEQ: u8 = 27;
    pub(crate) const JUMPIFLE: u8 = 28;
    pub(crate) const JUMPIFLT: u8 = 29;
    pub(crate) const JUMPIFNOTEQ: u8 = 30;
    pub(crate) const JUMPIFNOTLE: u8 = 31;
    pub(crate) const JUMPIFNOTLT: u8 = 32;
    pub(crate) const ADD: u8 = 33;
    pub(crate) const SUB: u8 = 34;
    pub(crate) const MUL: u8 = 35;
    pub(crate) const DIV: u8 = 36;
    pub(crate) const MOD: u8 = 37;
    pub(crate) const POW: u8 = 38;
    pub(crate) const ADDK: u8 = 39;
    pub(crate) const SUBK: u8 = 40;
    pub(crate) const MULK: u8 = 41;
    pub(crate) const DIVK: u8 = 42;
    pub(crate) const MODK: u8 = 43;
    pub(crate) const POWK: u8 = 44;
    pub(crate) const AND: u8 = 45;
    pub(crate) const OR: u8 = 46;
    pub(crate) const ANDK: u8 = 47;
    pub(crate) const ORK: u8 = 48;
    pub(crate) const CONCAT: u8 = 49;
    pub(crate) const NOT: u8 = 50;
    pub(crate) const MINUS: u8 = 51;
    pub(crate) const LENGTH: u8 = 52;
    pub(crate) const NEWTABLE: u8 = 53;
    pub(crate) const DUPTABLE: u8 = 54;
    pub(crate) const SETLIST: u8 = 55;
    pub(crate) const FORNPREP: u8 = 56;
    pub(crate) const FORNLOOP: u8 = 57;
    pub(crate) const FORGLOOP: u8 = 58;
    pub(crate) const FORGPREP_INEXT: u8 = 59;
    pub(crate) const FASTCALL3: u8 = 60;
    pub(crate) const FORGPREP_NEXT: u8 = 61;
    pub(crate) const NATIVECALL: u8 = 62;
    pub(crate) const GETVARARGS: u8 = 63;
    pub(crate) const DUPCLOSURE: u8 = 64;
    pub(crate) const PREPVARARGS: u8 = 65;
    pub(crate) const LOADKX: u8 = 66;
    pub(crate) const JUMPX: u8 = 67;
    pub(crate) const FASTCALL: u8 = 68;
    pub(crate) const COVERAGE: u8 = 69;
    pub(crate) const CAPTURE: u8 = 70;
    pub(crate) const SUBRK: u8 = 71;
    pub(crate) const DIVRK: u8 = 72;
    pub(crate) const FASTCALL1: u8 = 73;
    pub(crate) const FASTCALL2: u8 = 74;
    pub(crate) const FASTCALL2K: u8 = 75;
    pub(crate) const FORGPREP: u8 = 76;
    pub(crate) const JUMPXEQKNIL: u8 = 77;
    pub(crate) const JUMPXEQKB: u8 = 78;
    pub(crate) const JUMPXEQKN: u8 = 79;
    pub(crate) const JUMPXEQKS: u8 = 80;
    pub(crate) const IDIV: u8 = 81;
    pub(crate) const IDIVK: u8 = 82;
    pub(crate) const GETUDATAKS: u8 = 83;
    pub(crate) const SETUDATAKS: u8 = 84;
    pub(crate) const NAMECALLUDATA: u8 = 85;
    pub(crate) const NEWCLASSMEMBER: u8 = 86;
    pub(crate) const CALLFB: u8 = 87;
    pub(crate) const CMPPROTO: u8 = 88;
    pub(crate) const NEWCLASS: u8 = 89;

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
