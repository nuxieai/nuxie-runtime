extern crate alloc;

use luaur_ast::records::ast_name::AstName;
use luaur_common::enums::luau_bytecode_type::{
    LuauBytecodeType, LBC_TYPE_ANY, LBC_TYPE_BOOLEAN, LBC_TYPE_BUFFER, LBC_TYPE_INTEGER,
    LBC_TYPE_INVALID, LBC_TYPE_NIL, LBC_TYPE_NUMBER, LBC_TYPE_STRING, LBC_TYPE_THREAD,
    LBC_TYPE_VECTOR,
};

#[allow(non_snake_case)]
pub(crate) fn get_primitive_type(name: AstName) -> LuauBytecodeType {
    if name.operator_eq_c_char(c"nil".as_ptr()) {
        LBC_TYPE_NIL
    } else if name.operator_eq_c_char(c"boolean".as_ptr()) {
        LBC_TYPE_BOOLEAN
    } else if name.operator_eq_c_char(c"number".as_ptr()) {
        LBC_TYPE_NUMBER
    } else if name.operator_eq_c_char(c"integer".as_ptr()) {
        LBC_TYPE_INTEGER
    } else if name.operator_eq_c_char(c"string".as_ptr()) {
        LBC_TYPE_STRING
    } else if name.operator_eq_c_char(c"thread".as_ptr()) {
        LBC_TYPE_THREAD
    } else if name.operator_eq_c_char(c"buffer".as_ptr()) {
        LBC_TYPE_BUFFER
    } else if name.operator_eq_c_char(c"vector".as_ptr()) {
        LBC_TYPE_VECTOR
    } else if name.operator_eq_c_char(c"any".as_ptr())
        || name.operator_eq_c_char(c"unknown".as_ptr())
    {
        LBC_TYPE_ANY
    } else {
        LBC_TYPE_INVALID
    }
}
