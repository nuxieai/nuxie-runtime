#[allow(non_camel_case_types)]
pub struct CommaSeparatorInserter {
    pub(crate) first: bool,
    pub(crate) comma_position: *const crate::records::position::Position,
}

impl CommaSeparatorInserter {
    pub fn new(
        _writer: &mut dyn crate::records::writer::Writer,
        comma_position: *const crate::records::position::Position,
    ) -> Self {
        Self {
            first: true,
            comma_position: comma_position,
        }
    }

    #[allow(non_snake_case)]
    pub fn call(&mut self, writer: &mut dyn crate::records::writer::Writer) {
        if self.first {
            self.first = false;
        } else {
            if !self.comma_position.is_null() {
                unsafe {
                    writer.advance(&*self.comma_position);
                    self.comma_position = self.comma_position.add(1);
                }
            }
            writer.symbol(",");
        }
    }
}
