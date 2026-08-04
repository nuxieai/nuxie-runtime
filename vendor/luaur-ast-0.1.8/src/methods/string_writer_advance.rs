use crate::records::position::Position;
use crate::records::string_writer::StringWriter;

#[allow(non_snake_case)]
pub fn string_writer_advance(writer: &mut StringWriter, new_pos: &Position) {
    writer.advance(new_pos);
}
