use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn set_dump_source(&mut self, source: &str) {
        self.dump_source.clear();

        // Faithful to C++ `while (pos != npos)`: each iteration pushes exactly one line
        // (the chunk up to the next '\n', or the remainder), and the loop exits only AFTER
        // pushing the final chunk when no '\n' remains. A trailing '\n' therefore yields a
        // final EMPTY line — which dumpSourceRemarks relies on to emit the trailing newline.
        // The previous `if pos == len { break }` guard dropped that empty line.
        let mut pos: Option<usize> = Some(0);

        while let Some(p) = pos {
            match source[p..].find('\n') {
                None => {
                    self.dump_source.push(source[p..].to_owned());
                    pos = None;
                }
                Some(idx) => {
                    let next = p + idx;
                    self.dump_source.push(source[p..next].to_owned());
                    pos = Some(next + 1);
                }
            }

            if let Some(last) = self.dump_source.last_mut() {
                if !last.is_empty() && last.as_bytes()[last.len() - 1] == b'\r' {
                    last.pop();
                }
            }
        }
    }
}
