//! Node: `cxx:Method:Luau.Ast:Ast/src/Parser.cpp:934:Parser::validateAttribute`
//!
//! Validate an `@attr` against the known attribute tables (`kAttributeEntries`
//! plus the FFlag-gated `kDebugAttributeEntries`): resolve its `AstAttr::Type`,
//! reject duplicates, and run the per-attribute argument validator (only
//! `@deprecated` has one). The C++ static tables are inlined here as a `match`
//! rather than a sentinel-terminated array.

use crate::records::ast_array::AstArray;
use crate::records::ast_attr::{AstAttr, AstAttrType};
use crate::records::ast_expr::AstExpr;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;

impl Parser {
    #[allow(non_snake_case)]
    pub(crate) fn validate_attribute(
        &mut self,
        loc: Location,
        attribute_name: &str,
        attributes: &TempVector<'_, *mut AstAttr>,
        args: &AstArray<*mut AstExpr>,
    ) -> Option<AstAttrType> {
        // kAttributeEntries (Parser.cpp): name -> (type, optional args validator).
        // Only "deprecated" carries a validator (deprecatedArgsValidator).
        let mut r#type: Option<AstAttrType> = None;
        let mut has_deprecated_validator = false;

        match attribute_name {
            "checked" => r#type = Some(AstAttrType::Checked),
            "native" => r#type = Some(AstAttrType::Native),
            "deprecated" => {
                r#type = Some(AstAttrType::Deprecated);
                has_deprecated_validator = true;
            }
            _ => {}
        }

        // kDebugAttributeEntries: FFlag-gated debug-only attributes.
        if r#type.is_none()
            && attribute_name == "debugnoinline"
            && luaur_common::FFlag::DebugLuauNoInline.get()
        {
            r#type = Some(AstAttrType::DebugNoinline);
        }

        if let Some(attr_type) = r#type {
            // check that attribute is not duplicated
            for i in 0..attributes.size_ {
                let attr_ptr =
                    unsafe { *(*attributes.storage).as_ptr().add(attributes.offset + i) };
                unsafe {
                    if (*attr_ptr).r#type == attr_type {
                        self.report(
                            loc,
                            format_args!("Cannot duplicate attribute '@{}'", attribute_name),
                        );
                    }
                }
            }

            if has_deprecated_validator {
                let errors_to_report =
                    crate::functions::deprecated_args_validator::deprecated_args_validator(
                        loc, *args,
                    );
                for (error_loc, msg) in errors_to_report {
                    self.report(error_loc, format_args!("{}", msg));
                }
            }
        } else if attribute_name.is_empty() {
            self.report(loc, format_args!("Attribute name is missing"));
        } else {
            self.report(loc, format_args!("Invalid attribute '@{}'", attribute_name));
        }

        r#type
    }
}

// Free-function node surface delegating to the method.
#[allow(non_snake_case)]
pub fn parser_validate_attribute(
    parser: &mut Parser,
    loc: Location,
    attribute_name: &str,
    attributes: &TempVector<'_, *mut AstAttr>,
    args: &AstArray<*mut AstExpr>,
) -> Option<AstAttrType> {
    parser.validate_attribute(loc, attribute_name, attributes, args)
}
