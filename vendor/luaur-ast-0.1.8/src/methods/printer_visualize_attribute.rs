use crate::records::ast_attr::AstAttr;
use crate::records::printer::Printer;

impl<'a> Printer<'a> {
    pub fn visualize_attribute(&mut self, attribute: &mut AstAttr) {
        self.advance(&attribute.base.location.begin);
        self.writer.symbol("@");
        let name_val = attribute.name.value;
        let name_str = unsafe { core::ffi::CStr::from_ptr(name_val).to_string_lossy() };
        self.writer.identifier(&name_str);
    }
}
