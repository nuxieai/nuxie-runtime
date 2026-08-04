use crate::records::ast_array::AstArray;
use crate::records::binding::Binding;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub(crate) fn extract_annotation_colon_positions(
        &mut self,
        bindings: &TempVector<'_, Binding>,
    ) -> AstArray<Position> {
        // C++ uses `TempVector<Position>(scratch_position)`; a local Vec yields the
        // same `copy` and avoids borrowing `self.scratch_position` across the
        // `&mut self` copy call (scratch reuse is a non-observable optimization).
        let mut annotationColonPositions: alloc::vec::Vec<Position> = alloc::vec::Vec::new();
        for i in 0..bindings.size() {
            annotationColonPositions.push(bindings[i].colon_position);
        }

        self.copy_initializer_list_t(&annotationColonPositions)
    }
}
