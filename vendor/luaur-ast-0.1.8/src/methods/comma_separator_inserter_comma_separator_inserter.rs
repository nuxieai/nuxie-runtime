use crate::records::comma_separator_inserter::CommaSeparatorInserter;
use crate::records::position::Position;
use crate::records::writer::Writer;

impl CommaSeparatorInserter {
    // The implementation of CommaSeparatorInserter::new already exists in the record file.
    // As per the dependency policy for already-implemented containers and foundation types,
    // we provide a minimal stub here to avoid duplicate definition errors.
}

#[allow(non_snake_case)]
pub fn comma_separator_inserter_comma_separator_inserter<'a>(
    writer: &'a mut dyn Writer,
    comma_position: *const Position,
) -> CommaSeparatorInserter {
    CommaSeparatorInserter::new(writer, comma_position)
}
