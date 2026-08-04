use crate::records::lexeme::Lexeme;
use luaur_common::LUAU_ASSERT;

impl Lexeme {
    pub fn getBlock_depth(&self) -> u32 {
        LUAU_ASSERT!(
            self.r#type == crate::records::lexeme::Type::RawString
                || self.r#type == crate::records::lexeme::Type::BlockComment
        );

        unsafe {
            let data_ptr = self.data.data;
            let length = self.length as usize;

            // If we have a well-formed string, we are guaranteed to see 2 `]` characters after the end of the string contents
            LUAU_ASSERT!(*data_ptr.add(length) == b']' as i8);

            let mut depth: u32 = 0;
            loop {
                depth += 1;
                if *data_ptr.add(length + depth as usize) == b']' as i8 {
                    break;
                }
            }

            depth - 1
        }
    }
}

#[allow(non_snake_case)]
impl Lexeme {
    pub fn get_block_depth(&self) -> u32 {
        self.getBlock_depth()
    }
}
