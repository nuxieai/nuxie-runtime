use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::location::Location;
use alloc::string::String;
use alloc::vec::Vec;

#[allow(non_camel_case_types)]
pub type AttributeArgumentsValidator =
    alloc::boxed::Box<dyn Fn(Location, &AstArray<*mut AstExpr>) -> Vec<(Location, String)>>;
