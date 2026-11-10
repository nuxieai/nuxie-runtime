//! The bytecode verifier (plan decision D7). C++ Luau trusts its compiler;
//! the device loads bytecode it did not compile, so every prototype is
//! checked before any closure over it exists:
//!
//! - every opcode is one this VM executes, and every AUX word is present;
//! - register operands (including the ranges `CALL`, `RETURN`, `SETLIST`,
//!   `CONCAT`, the loops and `GETVARARGS` touch) are below `maxstacksize`;
//! - constant operands are in range and of the kind the opcode reads;
//! - upvalue operands are below `nups`, child-proto operands below the
//!   child count;
//! - jump targets land on an instruction (not an AUX word) inside the code;
//! - `CAPTURE` appears exactly after `NEWCLOSURE`/`DUPCLOSURE`, once per
//!   upvalue of the closure, and `NAMECALL` is followed by a call.
//!
//! The interpreter does not depend on the verifier for memory safety or
//! panic-freedom (it reads code, registers and constants through checked
//! accessors whose failure is an error), only for never having to report
//! malformed code as a runtime error.

use super::{Proto, insn_a, insn_b, insn_c, insn_d, insn_e, insn_op, op};
use crate::error::{Error, LoadError};
use crate::heap::{Function, Heap};
use crate::value::Value;

const CAPTURE_VAL: u8 = 0;
const CAPTURE_REF: u8 = 1;
const CAPTURE_UPVAL: u8 = 2;

struct Checker<'a> {
    proto: &'a Proto,
    heap: &'a Heap,
    pc: usize,
}

impl Checker<'_> {
    fn fail(&self, what: &str) -> Error {
        let op = self
            .proto
            .code
            .get(self.pc)
            .map_or(0, |&insn| insn_op(insn));
        Error::Load(LoadError::Malformed(format!(
            "pc {} (opcode {op}): {what}",
            self.pc
        )))
    }

    fn reg(&self, r: usize) -> Result<(), Error> {
        if r < usize::from(self.proto.maxstacksize) {
            Ok(())
        } else {
            Err(self.fail("register out of range"))
        }
    }

    /// Registers `first .. first + count` (empty when `count` is 0).
    fn regs(&self, first: usize, count: usize) -> Result<(), Error> {
        match count.checked_sub(1) {
            None => Ok(()),
            Some(last) => self.reg(first.saturating_add(last)),
        }
    }

    fn konst(&self, index: i64) -> Result<Value, Error> {
        usize::try_from(index)
            .ok()
            .and_then(|i| self.proto.k.get(i).copied())
            .ok_or_else(|| self.fail("constant out of range"))
    }

    fn string_k(&self, index: i64) -> Result<(), Error> {
        match self.konst(index)? {
            Value::String(_) => Ok(()),
            _ => Err(self.fail("constant is not a string")),
        }
    }

    fn number_k(&self, index: i64) -> Result<(), Error> {
        match self.konst(index)? {
            Value::Number(_) => Ok(()),
            _ => Err(self.fail("constant is not a number")),
        }
    }

    fn upval(&self, index: u8) -> Result<(), Error> {
        if index < self.proto.nups {
            Ok(())
        } else {
            Err(self.fail("upvalue out of range"))
        }
    }

    fn aux(&self) -> Result<u32, Error> {
        self.proto
            .code
            .get(self.pc.saturating_add(1))
            .copied()
            .ok_or_else(|| self.fail("missing AUX word"))
    }
}

/// Verifies `proto`. `child_nups` holds the upvalue count of each child.
pub(crate) fn verify(proto: &Proto, child_nups: &[u8], heap: &Heap) -> Result<(), Error> {
    let code = &proto.code;
    if usize::from(proto.numparams) > usize::from(proto.maxstacksize) {
        return Err(Error::Load(LoadError::Malformed(
            "more parameters than registers".to_owned(),
        )));
    }

    // Pass 1: instruction starts, so jumps can be checked against them.
    let mut starts = vec![false; code.len()];
    let mut pc = 0usize;
    while let Some(&insn) = code.get(pc) {
        if let Some(start) = starts.get_mut(pc) {
            *start = true;
        }
        pc = pc.saturating_add(if op::has_aux(insn_op(insn)) { 2 } else { 1 });
    }

    let mut c = Checker { proto, heap, pc: 0 };
    let jump = |c: &Checker<'_>, from: usize, offset: i64| -> Result<(), Error> {
        let target = i64::try_from(from).ok().and_then(|f| f.checked_add(offset));
        match target.and_then(|t| usize::try_from(t).ok()) {
            Some(t) if starts.get(t).copied().unwrap_or(false) => Ok(()),
            _ => Err(c.fail("jump target is not an instruction")),
        }
    };

    // The CALL a FASTCALL falls back to is at `next + C` (`next + C - 1`
    // for the forms with an AUX word); fast paths are skipped, see interp.rs.
    // Captures still expected after a NEWCLOSURE/DUPCLOSURE.
    let mut captures_left = 0usize;
    // DUPCLOSURE reuses a closure across calls, so it only admits captures
    // by value (C++ asserts this).
    let mut dup_captures = false;
    while let Some(&insn) = code.get(c.pc) {
        let o = insn_op(insn);
        let a = usize::from(insn_a(insn));
        let b = usize::from(insn_b(insn));
        let cc = usize::from(insn_c(insn));
        let d = i64::from(insn_d(insn));
        let next = c.pc.saturating_add(if op::has_aux(o) { 2 } else { 1 });
        if op::has_aux(o) {
            c.aux()?;
        }
        if captures_left > 0 && o != op::CAPTURE {
            return Err(c.fail("closure is missing CAPTURE instructions"));
        }
        match o {
            op::NOP | op::COVERAGE => {}
            op::LOADNIL | op::LOADN => c.reg(a)?,
            op::LOADB => {
                c.reg(a)?;
                jump(&c, next, cc as i64)?;
            }
            op::LOADK => {
                c.reg(a)?;
                c.konst(d)?;
            }
            op::LOADKX => {
                c.reg(a)?;
                c.konst(i64::from(c.aux()?))?;
            }
            op::MOVE | op::NOT | op::MINUS | op::LENGTH => {
                c.reg(a)?;
                c.reg(b)?;
            }
            op::GETGLOBAL | op::SETGLOBAL => {
                c.reg(a)?;
                c.string_k(i64::from(c.aux()?))?;
            }
            op::GETUPVAL | op::SETUPVAL => {
                c.reg(a)?;
                c.upval(insn_b(insn))?;
            }
            op::CLOSEUPVALS => {
                if a > usize::from(proto.maxstacksize) {
                    return Err(c.fail("register out of range"));
                }
            }
            op::GETIMPORT => {
                c.reg(a)?;
                c.konst(d)?;
                let aux = c.aux()?;
                let count = aux >> 30;
                if count == 0 || count > 3 {
                    return Err(c.fail("bad import path length"));
                }
                for id in [(aux >> 20) & 1023, (aux >> 10) & 1023, aux & 1023]
                    .iter()
                    .take(count as usize)
                {
                    c.string_k(i64::from(*id))?;
                }
            }
            op::GETTABLE
            | op::SETTABLE
            | op::ADD
            | op::SUB
            | op::MUL
            | op::DIV
            | op::MOD
            | op::POW
            | op::IDIV
            | op::AND
            | op::OR => {
                c.reg(a)?;
                c.reg(b)?;
                c.reg(cc)?;
            }
            op::GETTABLEKS | op::SETTABLEKS => {
                c.reg(a)?;
                c.reg(b)?;
                c.string_k(i64::from(c.aux()?))?;
            }
            op::GETTABLEN | op::SETTABLEN => {
                c.reg(a)?;
                c.reg(b)?;
            }
            op::NEWCLOSURE => {
                c.reg(a)?;
                let child = usize::try_from(d)
                    .ok()
                    .and_then(|i| child_nups.get(i).copied());
                let Some(nups) = child else {
                    return Err(c.fail("child proto out of range"));
                };
                captures_left = usize::from(nups);
                dup_captures = false;
            }
            op::DUPCLOSURE => {
                c.reg(a)?;
                let Value::Function(f) = c.konst(d)? else {
                    return Err(c.fail("constant is not a closure"));
                };
                let Function::Lua(closure) = c.heap.function(f) else {
                    return Err(c.fail("constant is not a Lua closure"));
                };
                captures_left = usize::from(closure.proto.nups);
                dup_captures = true;
            }
            op::CAPTURE => {
                if captures_left == 0 {
                    return Err(c.fail("CAPTURE outside a closure"));
                }
                captures_left = captures_left.saturating_sub(1);
                match insn_a(insn) {
                    CAPTURE_REF if dup_captures => {
                        return Err(c.fail("DUPCLOSURE captures by reference"));
                    }
                    CAPTURE_VAL | CAPTURE_REF => c.reg(b)?,
                    CAPTURE_UPVAL => c.upval(insn_b(insn))?,
                    _ => return Err(c.fail("unknown capture kind")),
                }
            }
            op::NAMECALL => {
                c.reg(a)?;
                c.reg(a.saturating_add(1))?;
                c.reg(b)?;
                c.string_k(i64::from(c.aux()?))?;
                let follows = code.get(next).map(|&i| insn_op(i));
                if !matches!(follows, Some(op::CALL) | Some(op::CALLFB)) {
                    return Err(c.fail("NAMECALL is not followed by a call"));
                }
            }
            op::CALL | op::CALLFB => {
                c.reg(a)?;
                if b > 0 {
                    c.regs(a, b)?;
                }
                if cc > 0 {
                    c.regs(a, cc.saturating_sub(1))?;
                }
            }
            op::RETURN => {
                if b > 0 {
                    c.regs(a, b.saturating_sub(1))?;
                } else if a > usize::from(proto.maxstacksize) {
                    return Err(c.fail("register out of range"));
                }
            }
            op::JUMP | op::JUMPBACK => jump(&c, next, d)?,
            op::JUMPX => jump(&c, next, i64::from(insn_e(insn)))?,
            op::JUMPIF | op::JUMPIFNOT => {
                c.reg(a)?;
                jump(&c, next, d)?;
            }
            op::JUMPIFEQ
            | op::JUMPIFLE
            | op::JUMPIFLT
            | op::JUMPIFNOTEQ
            | op::JUMPIFNOTLE
            | op::JUMPIFNOTLT => {
                c.reg(a)?;
                c.reg(c.aux()? as usize)?;
                jump(&c, c.pc.saturating_add(1), d)?;
            }
            op::JUMPXEQKNIL | op::JUMPXEQKB => {
                c.reg(a)?;
                jump(&c, c.pc.saturating_add(1), d)?;
            }
            op::JUMPXEQKN => {
                c.reg(a)?;
                c.number_k(i64::from(c.aux()? & 0x00ff_ffff))?;
                jump(&c, c.pc.saturating_add(1), d)?;
            }
            op::JUMPXEQKS => {
                c.reg(a)?;
                c.string_k(i64::from(c.aux()? & 0x00ff_ffff))?;
                jump(&c, c.pc.saturating_add(1), d)?;
            }
            op::ADDK | op::SUBK | op::MULK | op::DIVK | op::MODK | op::POWK | op::IDIVK => {
                c.reg(a)?;
                c.reg(b)?;
                c.number_k(cc as i64)?;
            }
            op::ANDK | op::ORK => {
                c.reg(a)?;
                c.reg(b)?;
                c.konst(cc as i64)?;
            }
            op::SUBRK | op::DIVRK => {
                c.reg(a)?;
                c.number_k(b as i64)?;
                c.reg(cc)?;
            }
            op::CONCAT => {
                c.reg(a)?;
                if b > cc {
                    return Err(c.fail("empty CONCAT range"));
                }
                c.reg(cc)?;
            }
            op::NEWTABLE => c.reg(a)?,
            op::DUPTABLE => {
                c.reg(a)?;
                if !matches!(c.konst(d)?, Value::Table(_)) {
                    return Err(c.fail("constant is not a table"));
                }
            }
            op::SETLIST => {
                c.reg(a)?;
                c.reg(b)?;
                if cc > 0 {
                    c.regs(b, cc.saturating_sub(1))?;
                }
            }
            op::FORNPREP | op::FORNLOOP | op::FORGPREP | op::FORGPREP_INEXT | op::FORGPREP_NEXT => {
                c.regs(a, 3)?;
                jump(&c, next, d)?;
            }
            op::FORGLOOP => {
                let vars = (c.aux()? & 0xff) as usize;
                c.regs(a, 3usize.saturating_add(vars))?;
                jump(&c, c.pc.saturating_add(1), d)?;
            }
            op::FASTCALL => jump(&c, next, cc as i64)?,
            op::FASTCALL1 => {
                c.reg(b)?;
                jump(&c, next, cc as i64)?;
            }
            op::FASTCALL2 => {
                c.reg(b)?;
                c.reg(c.aux()? as usize)?;
                jump(&c, next, (cc as i64).saturating_sub(1))?;
            }
            op::FASTCALL2K => {
                c.reg(b)?;
                c.konst(i64::from(c.aux()?))?;
                jump(&c, next, (cc as i64).saturating_sub(1))?;
            }
            op::FASTCALL3 => {
                let aux = c.aux()?;
                c.reg(b)?;
                c.reg((aux & 0xff) as usize)?;
                c.reg(((aux >> 8) & 0xff) as usize)?;
                jump(&c, next, (cc as i64).saturating_sub(1))?;
            }
            op::GETVARARGS => {
                if b > 0 {
                    c.regs(a, b.saturating_sub(1))?;
                } else {
                    c.reg(a)?;
                }
            }
            op::PREPVARARGS => {
                if a != usize::from(proto.numparams) {
                    return Err(c.fail("PREPVARARGS parameter count differs from the proto"));
                }
            }
            op::BREAK => return Err(c.fail("BREAK needs a debugger, which this VM does not have")),
            op::NATIVECALL => return Err(c.fail("NATIVECALL is native code generation only")),
            op::GETUDATAKS | op::SETUDATAKS | op::NAMECALLUDATA => {
                return Err(
                    c.fail("userdata direct-access opcodes are never emitted by the compiler")
                );
            }
            op::NEWCLASSMEMBER | op::NEWCLASS | op::CMPPROTO => {
                return Err(c.fail("user-defined classes and CMPPROTO are not admitted"));
            }
            _ => return Err(c.fail("unknown opcode")),
        }
        c.pc = next;
    }
    if captures_left > 0 {
        return Err(c.fail("closure is missing CAPTURE instructions"));
    }
    Ok(())
}
