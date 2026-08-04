use crate::records::fenv_visitor::FenvVisitor;

impl<'a> FenvVisitor<'a> {
    pub fn fenv_visitor(getfenv_used: &'a mut bool, setfenv_used: &'a mut bool) -> Self {
        FenvVisitor {
            getfenv_used,
            setfenv_used,
        }
    }
}
