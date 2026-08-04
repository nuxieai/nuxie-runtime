use crate::records::bytecode_builder::BytecodeBuilder;
use alloc::string::String;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub struct HotComment {
    pub content: String,
    pub header: bool,
}

pub fn has_native_comment_directive(hotcomments: &[HotComment]) -> bool {
    for hc in hotcomments {
        if hc.content.is_empty()
            || hc.content.as_bytes().first() == Some(&b' ')
            || hc.content.as_bytes().first() == Some(&b'\t')
        {
            continue;
        }

        if hc.header {
            let bytes = hc.content.as_bytes();
            let space_pos = bytes.iter().position(|&b| b == b' ' || b == b'\t');

            let first = if let Some(pos) = space_pos {
                &hc.content[..pos]
            } else {
                &hc.content[..]
            };

            if first == "native" {
                return true;
            }
        }
    }

    false
}

impl BytecodeBuilder {
    pub fn end_function(&mut self, maxstacksize: u8, numupvalues: u8, flags: u8) {
        LUAU_ASSERT!(self.current_function != u32::MAX);

        let current_function = self.current_function;
        let dump = if let Some(dump_fn) = self.dump_function_ptr {
            let mut dumpinstoffs =
                core::mem::take(&mut self.functions[current_function as usize].dumpinstoffs);
            let dump = dump_fn(self, &mut dumpinstoffs);
            self.functions[current_function as usize].dumpinstoffs = dumpinstoffs;
            Some(dump)
        } else {
            None
        };
        {
            let func = &mut self.functions[current_function as usize];
            func.maxstacksize = maxstacksize;
            func.numupvalues = numupvalues;

            if let Some(dump) = dump {
                func.dump = dump;
            }

            func.data.reserve(32 + self.insns.len() * 7);
        }

        if let Some(encoder) = self.encoder {
            unsafe {
                (*encoder).encode(&mut self.insns);
            }
        }

        let mut data = String::new();
        self.write_function(&mut data, current_function, flags);
        self.functions[current_function as usize].data = data;

        self.current_function = u32::MAX;
        self.total_instruction_count += self.insns.len();

        self.insns.clear();
        self.lines.clear();
        self.constants.clear();
        self.protos.clear();
        self.jumps.clear();
        self.fb_slots.clear();
        self.table_shapes.clear();

        self.debug_locals.clear();
        self.debug_upvals.clear();

        self.typed_locals.clear();
        self.typed_upvals.clear();

        self.constant_map.clear();
        self.table_shape_map.clear();
        self.proto_map.clear();

        self.debug_remarks.clear();
        self.debug_remark_buffer.clear();
    }
}
