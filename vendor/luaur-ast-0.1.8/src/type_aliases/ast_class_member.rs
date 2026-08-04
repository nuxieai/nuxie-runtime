use crate::records::ast_class_method::AstClassMethod;
use crate::records::ast_class_property::AstClassProperty;
use luaur_common::records::variant::Variant2;

#[allow(non_camel_case_types)]
pub type AstClassMember = Variant2<AstClassProperty, AstClassMethod>;
