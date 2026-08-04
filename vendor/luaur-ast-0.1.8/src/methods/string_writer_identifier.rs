use crate::records::string_writer::StringWriter;

#[allow(non_snake_case)]
pub fn string_writer_identifier(writer: &mut StringWriter, s: &str) {
    writer.identifier(s);
}
