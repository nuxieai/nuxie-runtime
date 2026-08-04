extern crate alloc;

#[cfg(test)]
mod ast_visitor_tests;
pub mod enums;
pub mod functions;
#[cfg(test)]
mod lexer_oracle_test;
#[cfg(test)]
mod lexer_tests;
pub mod macros;
pub mod methods;
#[cfg(test)]
mod parser_tests;
pub mod records;
pub mod rtti;
pub mod testdata;
pub mod type_aliases;
pub mod visit;

pub static mut LUAU_TELEMETRY_PARSED_RETURN_TYPE_VARIADIC_WITH_TYPE_SUFFIX: bool = false;
