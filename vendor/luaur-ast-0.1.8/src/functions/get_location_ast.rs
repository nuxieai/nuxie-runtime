use crate::records::ast_array::AstArray;
use crate::records::location::Location;

#[allow(non_snake_case)]
pub fn get_location<T>(array: AstArray<*mut T>) -> Location
where
    T: AsRef<crate::records::ast_node::AstNode>,
{
    if array.size == 0 {
        return Location::default();
    }

    unsafe {
        let first = *array.data;
        let last = *array.data.add(array.size - 1);

        Location {
            begin: (*first).as_ref().location.begin,
            end: (*last).as_ref().location.end,
        }
    }
}

// Pinned overload name advertised by the dependency cards.
#[allow(unused_imports, non_snake_case)]
pub use get_location as get_location_ast_array_t;
