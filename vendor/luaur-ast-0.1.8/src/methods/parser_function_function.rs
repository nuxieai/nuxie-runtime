use crate::records::function::Function;

impl Function {
    pub fn new() -> Self {
        Self {
            vararg: false,
            loop_depth: 0,
        }
    }
}

#[allow(non_snake_case)]
pub fn parser_function_function() -> Function {
    Function::new()
}
