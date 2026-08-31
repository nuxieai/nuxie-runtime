//! renderer/cmd/id_allocator.hpp at e949498e. Both wire id spaces use u32.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Allocation {
    pub id: u32,
    pub generation: u32,
}
#[derive(Default)]
pub struct IdAllocator {
    free: Vec<Allocation>,
    next: u32,
}
impl IdAllocator {
    pub fn alloc(&mut self) -> Allocation {
        if let Some(allocation) = self.free.pop() {
            return allocation;
        }
        assert!(self.next < super::handle_flags::HANDLE_FOREIGN_FLAG);
        let allocation = Allocation {
            id: self.next,
            generation: 0,
        };
        self.next += 1;
        allocation
    }
    pub fn release(&mut self, id: u32, generation: u32) {
        if generation != u32::MAX {
            self.free.push(Allocation {
                id,
                generation: generation + 1,
            });
        }
    }
}
