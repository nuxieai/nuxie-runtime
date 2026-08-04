use crate::records::bytecode_builder::BytecodeBuilder;
use alloc::string::String;

impl BytecodeBuilder {
    pub fn get_error(message: &str) -> String {
        // 0 acts as a special marker for error bytecode (it's equal to LBC_VERSION_TARGET for valid bytecode blobs)
        let mut result = String::with_capacity(message.len() + 1);
        result.push('\0');
        result.push_str(message);

        result
    }
}
