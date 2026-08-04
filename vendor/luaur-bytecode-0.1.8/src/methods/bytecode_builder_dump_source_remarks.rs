use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::functions::format_append::formatAppend;

impl BytecodeBuilder {
    pub fn dump_source_remarks(&self) -> alloc::string::String {
        let mut result = alloc::string::String::new();
        let mut next_remark: usize = 0;

        let mut remarks: Vec<(i32, alloc::string::String)> = self.dump_remarks.clone();
        // C++ `std::sort(remarks)` orders by the WHOLE (line, message) pair, so within a
        // line remarks are ordered lexicographically by text ("builtin ..." before
        // "inlining ...") and the consecutive-duplicate skip can collapse repeated inline
        // remarks. Sorting by line only left them in insertion order.
        remarks.sort();

        for i in 0..self.dump_source.len() {
            let line: &alloc::string::String = &self.dump_source[i];

            let mut indent: usize = 0;
            let line_len = line.len();
            while indent < line_len
                && (line.as_bytes()[indent] == b' ' || line.as_bytes()[indent] == b'\t')
            {
                indent += 1;
            }

            while next_remark < remarks.len() && remarks[next_remark].0 == (i as i32 + 1) {
                formatAppend(
                    &mut result,
                    format_args!("{:.*}-- remark: {}\n", indent, line, remarks[next_remark].1),
                );
                next_remark += 1;

                // skip duplicate remarks (due to inlining/unrolling)
                while next_remark < remarks.len()
                    && remarks[next_remark] == remarks[next_remark - 1]
                {
                    next_remark += 1;
                }
            }

            result.push_str(line);
            if i + 1 < self.dump_source.len() {
                result.push('\n');
            }
        }

        result
    }
}
