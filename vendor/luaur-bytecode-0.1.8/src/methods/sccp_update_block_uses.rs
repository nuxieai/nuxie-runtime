use crate::enums::bc_block_flag::BcBlockFlag;
use crate::records::sccp::Sccp;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn update_block_uses(&mut self) {
        let entry = self.func().entry_block.index;
        let exit = self.func().exit_block.index;
        let mut reachable = luaur_common::records::dense_hash_set2::DenseHashSet2::<u32>::new();
        let mut worklist = vec![entry];
        reachable.insert(entry);
        reachable.insert(exit);
        while let Some(index) = worklist.pop() {
            let successors = self.func().blocks[index as usize].successors.clone();
            for edge in successors.iter() {
                let successor = edge.target.index;
                if !reachable.contains(&successor) {
                    reachable.insert(successor);
                    worklist.push(successor);
                }
            }
        }
        for index in 0..self.func().blocks.len() {
            let count = self.block_uses.get_or_insert(index as u32).size() as u32;
            let block = &mut self.func_mut().blocks[index];
            block.useCount = count;
            if !reachable.contains(&(index as u32)) {
                block.flags |= BcBlockFlag::Dead as u8;
            }
        }
    }
}
