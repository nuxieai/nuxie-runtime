use crate::records::string_writer::StringWriter;

impl StringWriter {
    pub fn write_c_char(&mut self, c: char) {
        self.ss.push(c);
        self.pos.column += 1;
        self.last_char = c;
    }
}

#[allow(non_snake_case)]
pub fn string_writer_write(writer: &mut StringWriter, c: char) {
    writer.write_c_char(c);
}
