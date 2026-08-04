use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::debug_local_bytecode_builder::DebugLocal;
use crate::records::jump::Jump;
use crate::records::typed_local_bytecode_builder::TypedLocal;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::functions::get_op_length::getOpLength;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_insn_d::LUAU_INSN_D;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;

impl BytecodeBuilder {
    pub fn expand_jumps(&mut self) {
        if !self.has_long_jumps {
            return;
        }

        // we have some jump instructions that couldn't be patched which means their offset didn't fit into 16 bits
        // our strategy for replacing instructions is as follows: instead of
        //   OP jumpoffset
        // we will synthesize a jump trampoline before our instruction (note that jump offsets are relative to next instruction):
        //   JUMP +1
        //   JUMPX jumpoffset
        //   OP -2
        // the idea is that during forward execution, we will jump over JUMPX into OP; if OP decides to jump, it will jump to JUMPX
        // JUMPX can carry a 24-bit jump offset

        // jump trampolines expand the code size, which can increase existing jump distances.
        // because of this, we may need to expand jumps that previously fit into 16-bit just fine.
        // the worst-case expansion is 3x, so to be conservative we will repatch all jumps that have an offset >= 32767/3
        const K_MAX_JUMP_DISTANCE_CONSERVATIVE: i32 = 32767 / 3;

        // we will need to process jumps in order
        self.jumps.sort_by(|lhs, rhs| lhs.source.cmp(&rhs.source));

        // first, let's add jump thunks for every jump with a distance that's too big
        // we will create new instruction buffers, with remap table keeping track of the moves: remap[oldpc] = newpc
        let mut remap: Vec<u32> = vec![0; self.insns.len()];

        let mut newinsns: Vec<u32> = Vec::with_capacity(self.insns.len());
        let mut newlines: Vec<i32> = Vec::with_capacity(self.insns.len());

        LUAU_ASSERT!(self.insns.len() == self.lines.len());

        let mut current_jump: usize = 0;
        let mut pending_trampolines: usize = 0;

        let mut i: usize = 0;
        while i < self.insns.len() {
            let op = LUAU_INSN_OP(self.insns[i]) as u8;
            LUAU_ASSERT!(op < LuauOpcode::LOP__COUNT as u8);

            if current_jump < self.jumps.len() && self.jumps[current_jump].source == i as u32 {
                let offset = (self.jumps[current_jump].target as i32)
                    - (self.jumps[current_jump].source as i32)
                    - 1;

                if offset.abs() > K_MAX_JUMP_DISTANCE_CONSERVATIVE {
                    // insert jump trampoline as described above; we keep JUMPX offset uninitialized in this pass
                    newinsns.push(LuauOpcode::LOP_JUMP as u32 | (1 << 16));
                    newinsns.push(LuauOpcode::LOP_JUMPX as u32);

                    newlines.push(self.lines[i]);
                    newlines.push(self.lines[i]);

                    pending_trampolines += 1;
                }

                current_jump += 1;
            }

            let oplen = getOpLength(unsafe { core::mem::transmute::<u8, LuauOpcode>(op) });

            // copy instruction and line info to the new stream
            for _ in 0..oplen {
                remap[i] = newinsns.len() as u32;

                newinsns.push(self.insns[i]);
                newlines.push(self.lines[i]);

                i += 1;
            }
        }

        LUAU_ASSERT!(current_jump == self.jumps.len());
        LUAU_ASSERT!(pending_trampolines > 0);

        // now we need to recompute offsets for jump instructions - we could not do this in the first pass because the offsets are between *target*
        // instructions
        for jump in &mut self.jumps {
            let offset = (jump.target as i32) - (jump.source as i32) - 1;
            let newoffset =
                (remap[jump.target as usize] as i32) - (remap[jump.source as usize] as i32) - 1;

            if offset.abs() > K_MAX_JUMP_DISTANCE_CONSERVATIVE {
                // fix up jump trampoline
                let trampoline_pos = remap[jump.source as usize] as usize - 1;
                let op_pos = trampoline_pos + 1;

                let (left, right) = newinsns.split_at_mut(op_pos);
                let insnt = &mut left[trampoline_pos];
                let insnj = &mut right[0];

                LUAU_ASSERT!(LUAU_INSN_OP(*insnt) == LuauOpcode::LOP_JUMPX as u32);

                // patch JUMPX to JUMPX to target location; note that newoffset is the offset of the jump *relative to OP*, so we need to add 1 to make it
                // relative to JUMPX
                *insnt &= 0xff;
                *insnt |= ((newoffset + 1) as u32) << 8;

                // patch OP to OP -2
                *insnj &= 0xffff;
                *insnj |= ((-2i16) as u32) << 16;

                pending_trampolines -= 1;
            } else {
                let insn = &mut newinsns[remap[jump.source as usize] as usize];

                // make sure jump instruction had the correct offset before we started
                LUAU_ASSERT!(LUAU_INSN_D(*insn) == offset);

                // patch instruction with the new offset
                LUAU_ASSERT!(i32::from((newoffset as i16)) == newoffset);

                *insn &= 0xffff;
                *insn |= (newoffset as u32) << 16;
            }
        }

        LUAU_ASSERT!(pending_trampolines == 0);

        // this was hard, but we're done.
        core::mem::swap(&mut self.insns, &mut newinsns);
        core::mem::swap(&mut self.lines, &mut newlines);

        for debug_local in &mut self.debug_locals {
            // endpc is exclusive, to get the right remapping, we need to remap the location before the end
            if debug_local.startpc != debug_local.endpc {
                debug_local.endpc = remap[(debug_local.endpc - 1) as usize] + 1;
            } else {
                debug_local.endpc = remap[debug_local.endpc as usize];
            }

            debug_local.startpc = remap[debug_local.startpc as usize];
        }

        for typed_local in &mut self.typed_locals {
            // endpc is exclusive, to get the right remapping, we need to remap the location before the end
            if typed_local.startpc != typed_local.endpc {
                typed_local.endpc = remap[(typed_local.endpc - 1) as usize] + 1;
            } else {
                typed_local.endpc = remap[typed_local.endpc as usize];
            }

            typed_local.startpc = remap[typed_local.startpc as usize];
        }
    }
}
