use crate::records::position::Position;
use crate::records::string_writer::StringWriter;

#[allow(non_snake_case)]
pub fn string_writer_maybe_space(writer: &mut StringWriter, new_pos: &Position, reserve: i32) {
    writer.maybe_space(new_pos, reserve);
}
