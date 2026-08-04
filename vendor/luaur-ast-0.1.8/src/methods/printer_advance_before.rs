use crate::records::position::Position;
use crate::records::printer::Printer;

impl<'a> Printer<'a> {
    pub fn advance_before(&mut self, new_pos: Position, token_length: u32) {
        if new_pos.column >= token_length {
            let new_pos = Position::new(new_pos.line, new_pos.column - token_length);
            self.advance(&new_pos);
        } else {
            self.advance(&new_pos);
        }
    }
}

#[allow(non_snake_case)]
pub fn printer_advance_before(printer: &mut Printer<'_>, new_pos: Position, token_length: u32) {
    printer.advance_before(new_pos, token_length);
}
