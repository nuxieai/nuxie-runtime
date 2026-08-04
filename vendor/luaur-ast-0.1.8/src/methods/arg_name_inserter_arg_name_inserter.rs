use crate::records::arg_name_inserter::ArgNameInserter;
use crate::records::ast_array::AstArray;
use crate::records::position::Position;
use crate::records::writer::Writer;
use crate::type_aliases::ast_argument_name::AstArgumentName;

impl<'a> ArgNameInserter<'a> {
    #[allow(non_snake_case)]
    pub fn arg_name_inserter_arg_name_inserter(
        writer: &'a mut dyn Writer,
        names: AstArray<Option<AstArgumentName>>,
        colon_positions: AstArray<Position>,
    ) -> Self {
        ArgNameInserter::new(writer, names, colon_positions)
    }
}
