use crate::records::ast_local::AstLocal;
use crate::records::position::Position;
use crate::records::printer::Printer;

impl<'a> Printer<'a> {
    pub fn visualize_ast_local_position(&mut self, local: &AstLocal, colon_position: Position) {
        self.advance(&local.location.begin);

        let name_val = local.name.value;
        let name_str = unsafe { core::ffi::CStr::from_ptr(name_val).to_string_lossy() };
        self.writer.identifier(&name_str);
        if self.write_types && !local.annotation.is_null() {
            self.maybe_advance_and_write(&colon_position, ":", true);
            unsafe {
                self.visualize_type_annotation(&mut *local.annotation);
            }
        }
    }
}
