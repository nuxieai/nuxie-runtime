use crate::functions::log_2::log2;
use crate::functions::write_byte::writeByte;
use crate::functions::write_int::writeInt;
use crate::records::bytecode_builder::BytecodeBuilder;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn write_line_info(&self, ss: &mut String) {
        LUAU_ASSERT!(!self.lines.is_empty());

        let mut span = 1 << 24;

        let mut offset = 0;
        while offset < self.lines.len() {
            let mut next = offset;
            let mut min = self.lines[offset];
            let mut max = self.lines[offset];

            while next < self.lines.len() && next < offset + span {
                min = cmp::min(min, self.lines[next]);
                max = cmp::max(max, self.lines[next]);

                if max - min > 255 {
                    break;
                }
                next += 1;
            }

            if next < self.lines.len() && next - offset < span {
                span = 1 << log2((next - offset) as i32);
            } else {
                offset += span;
            }
        }

        let mut baseline_one = 0;
        let mut baseline_scratch = Vec::new();
        let baseline_size = (self.lines.len() - 1) / span + 1;

        if baseline_size > 1 {
            baseline_scratch.resize(baseline_size, 0);
        }

        let baseline = if baseline_size > 1 {
            &mut baseline_scratch
        } else {
            core::slice::from_mut(&mut baseline_one)
        };

        for offset in (0..self.lines.len()).step_by(span) {
            let mut next = offset;
            let mut min = self.lines[offset];

            while next < self.lines.len() && next < offset + span {
                min = cmp::min(min, self.lines[next]);
                next += 1;
            }

            baseline[offset / span] = min;
        }

        let logspan = log2(span as i32);
        writeByte(ss, logspan as u8);

        let mut last_offset = 0u8;
        for i in 0..self.lines.len() {
            let delta = self.lines[i] - baseline[i >> logspan];
            LUAU_ASSERT!(delta >= 0 && delta <= 255);

            writeByte(ss, (delta as u8).wrapping_sub(last_offset));
            last_offset = delta as u8;
        }

        let mut last_line = 0;
        for i in 0..baseline_size {
            writeInt(ss, baseline[i] - last_line);
            last_line = baseline[i];
        }
    }
}
