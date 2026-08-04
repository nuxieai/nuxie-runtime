use crate::records::bytecode_builder::BytecodeBuilder;
use core::cmp;

impl BytecodeBuilder {
    pub fn fill_baseline_info(&self, span: usize, baseline: &mut [i32], _baseline_size: usize) {
        for offset in (0..self.lines.len()).step_by(span) {
            let mut next = offset;
            let mut min = self.lines[offset];

            while next < self.lines.len() && next < offset + span {
                min = cmp::min(min, self.lines[next]);
                next += 1;
            }

            baseline[offset / span] = min;
        }
    }
}
