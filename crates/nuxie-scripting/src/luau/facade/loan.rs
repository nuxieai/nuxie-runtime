//! What a call from Rust lends to the host callbacks it reaches.
//!
//! Some host data lives only as long as one script call: the renderer a
//! draw callback draws into is borrowed from the caller's frame. The safe
//! VM carries such a borrow down the call (`nuxie_luau_vm::LoanFamily`):
//! [`Function::call_lent`](super::Function::call_lent) starts the call
//! with a loan, every Lua frame and host callback it runs passes it on,
//! and a callback registered to receive it
//! ([`UserDataMethods::add_method_lent`](super::UserDataMethods::add_method_lent))
//! gets it as an argument. Nothing stores it: when the call returns the
//! borrow ends, and Lua code run by a later call, or by a call from Rust
//! that does not pass it on, sees no loan.

use nuxie_render_api::Renderer;

/// Borrowed host data lent to one call (a trait object, so that other
/// scoped host data can be lent the same way).
pub trait ScriptLoan {
    /// The renderer lent to the renderer scope `token`, if this loan is
    /// that scope's.
    fn renderer(&mut self, token: u64) -> Option<&mut dyn Renderer>;
}

/// The facade's loan family: a `dyn ScriptLoan`.
pub enum ScriptLoans {}

impl nuxie_luau_vm::LoanFamily for ScriptLoans {
    type Loan<'l> = dyn ScriptLoan + 'l;

    fn shorten<'a, 'l: 'a>(loan: &'a mut (dyn ScriptLoan + 'l)) -> &'a mut (dyn ScriptLoan + 'a) {
        loan
    }
}

/// The loan a host callback receives, if its call carries one.
pub type Loan<'a> = Option<&'a mut (dyn ScriptLoan + 'a)>;

/// `loan`, reborrowed for one more use (an `Option` does not coerce the
/// trait object's lifetime by itself).
pub fn reborrow<'b>(loan: &'b mut Loan<'_>) -> Loan<'b> {
    match loan {
        Some(loan) => Some(&mut **loan),
        None => None,
    }
}
