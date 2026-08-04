use crate::records::sccp::Sccp;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn rewrite(&mut self) {
        self.arith_to_k();
        self.replace_uses();
        self.simplify_phis();
        self.update_block_uses();
    }
}
