use crate::records::string_writer::StringWriter;

#[allow(non_snake_case)]
pub fn string_writer_symbol(writer: &mut StringWriter, s: &str) {
    writer.symbol(s);
}
