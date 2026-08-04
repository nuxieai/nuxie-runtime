#[allow(non_camel_case_types)]
pub struct ArgNameInserter<'a> {
    pub(crate) writer: &'a mut dyn crate::records::writer::Writer,
    pub(crate) names: crate::records::ast_array::AstArray<
        Option<crate::type_aliases::ast_argument_name::AstArgumentName>,
    >,
    pub(crate) colon_positions:
        crate::records::ast_array::AstArray<crate::records::position::Position>,
    pub(crate) idx: usize,
}

impl<'a> ArgNameInserter<'a> {
    pub(crate) fn new(
        writer: &'a mut dyn crate::records::writer::Writer,
        names: crate::records::ast_array::AstArray<
            Option<crate::type_aliases::ast_argument_name::AstArgumentName>,
        >,
        colon_positions: crate::records::ast_array::AstArray<crate::records::position::Position>,
    ) -> Self {
        Self {
            writer,
            names,
            colon_positions,
            idx: 0,
        }
    }
}
