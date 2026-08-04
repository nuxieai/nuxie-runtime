use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::function::Function;
use luaur_common::functions::format_append::formatAppend;

impl BytecodeBuilder {
    pub fn dump_everything(&self) -> String {
        let mut result = String::new();

        for (i, function) in self.functions.iter().enumerate() {
            let debugname = if function.dumpname.is_empty() {
                "??"
            } else {
                &function.dumpname
            };

            formatAppend(
                &mut result,
                format_args!("Function {} ({}):\n", i as i32, debugname),
            );

            result.push_str(&function.dump);
            result.push('\n');
        }

        result
    }
}
