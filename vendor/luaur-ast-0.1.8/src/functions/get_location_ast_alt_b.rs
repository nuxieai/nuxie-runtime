#[allow(non_snake_case)]
pub fn get_location(
    type_list: &crate::records::ast_type_list::AstTypeList,
) -> crate::records::location::Location {
    let mut result = crate::records::location::Location::default();

    if type_list.types.size > 0 {
        unsafe {
            let first = *type_list.types.data.add(0);
            let last = *type_list.types.data.add(type_list.types.size - 1);

            result = crate::records::location::Location {
                begin: (*first).base.location.begin,
                end: (*last).base.location.end,
            };
        }
    }

    if !type_list.tail_type.is_null() {
        unsafe {
            result.end = (*type_list.tail_type).base.location.end;
        }
    }

    result
}

// Pinned overload name advertised by the dependency cards.
#[allow(unused_imports, non_snake_case)]
pub use get_location as get_location_ast_type_list;
