use crate::records::allocator::Allocator;
use crate::records::ast_name_table::AstNameTable;

impl AstNameTable {
    /// Re-point the interning allocator to `allocator`. The fixture calls this
    /// before each parse so the name table uses the allocator at its *current*
    /// address after the owning struct has been moved.
    pub fn rebind_allocator(&mut self, allocator: *mut Allocator) {
        self.allocator = allocator;
    }
}
