use crate::records::arg_name_inserter::ArgNameInserter;

impl<'a> ArgNameInserter<'a> {
    pub fn operator_call(&mut self) {
        if self.idx < self.names.size {
            let name = unsafe { *self.names.data.add(self.idx) };
            if let Some(name_val) = name {
                // In Luau's Rust port, std::pair<AstName, Location> is translated as a tuple (AstName, Location)
                // name_val.first -> name_val.0, name_val.second -> name_val.1
                self.writer.advance(&name_val.1.begin);
                let name_str =
                    unsafe { core::ffi::CStr::from_ptr(name_val.0.value).to_string_lossy() };
                self.writer.identifier(&name_str);

                if self.idx < self.colon_positions.size {
                    unsafe {
                        self.writer
                            .advance(&*self.colon_positions.data.add(self.idx));
                    }
                }

                self.writer.symbol(":");
            }
        }
        self.idx += 1;
    }
}
