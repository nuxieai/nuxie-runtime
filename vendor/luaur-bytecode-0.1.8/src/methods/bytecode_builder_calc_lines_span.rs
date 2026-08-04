use crate::functions::log_2::log2;
use crate::records::bytecode_builder::BytecodeBuilder;
use core::cmp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn calc_lines_span(&self) -> usize {
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

        span
    }
}
