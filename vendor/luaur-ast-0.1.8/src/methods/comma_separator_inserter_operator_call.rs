use crate::records::comma_separator_inserter::CommaSeparatorInserter;

impl CommaSeparatorInserter {
    pub fn operator_call(&mut self, writer: &mut dyn crate::records::writer::Writer) {
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
