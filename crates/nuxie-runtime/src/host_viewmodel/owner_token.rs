//! An opaque value a host attaches to a runtime value it made, kept alive for
//! exactly as long as the runtime keeps that value.
//!
//! A script host charges the memory a script makes the runtime hold (a
//! decoded font, a blob, a view-model instance) against a budget. The
//! runtime owns that memory, and its own graph decides how long it lives:
//! a font bound into a text run, a blob copied into another property, an
//! instance kept by its File's arena. A charge kept by the host's wrapper
//! would be refunded when the wrapper goes, while the runtime still holds
//! the memory. So the host hands the runtime this token with the value, the
//! runtime stores it beside (or inside) the value and drops it with it, and
//! the host's refund runs then. The runtime never looks inside.

use std::any::Any;
use std::sync::Arc;

/// See the module documentation. Cloning shares the same owner.
#[derive(Clone)]
pub struct HostOwnerToken(Arc<dyn Any + Send + Sync>);

impl HostOwnerToken {
    /// A token that keeps `owner` alive.
    pub fn new<T: Any + Send + Sync>(owner: T) -> Self {
        Self(Arc::new(owner))
    }

    /// A token that keeps an owner the host already shares.
    pub fn from_arc<T: Any + Send + Sync>(owner: Arc<T>) -> Self {
        Self(owner)
    }

    /// The owner, for the host that made it: `None` if it is another type.
    pub fn downcast<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        Arc::clone(&self.0).downcast().ok()
    }

    /// Whether both tokens keep the same owner.
    pub fn same(&self, other: &Self) -> bool {
        std::ptr::addr_eq(Arc::as_ptr(&self.0), Arc::as_ptr(&other.0))
    }
}

impl std::fmt::Debug for HostOwnerToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HostOwnerToken")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_keeps_its_owner_until_the_last_clone_drops() {
        let owner = Arc::new(7_u32);
        let token = HostOwnerToken::from_arc(Arc::clone(&owner));
        let clone = token.clone();
        assert!(token.same(&clone));
        assert_eq!(token.downcast::<u32>().as_deref(), Some(&7));
        assert!(token.downcast::<u64>().is_none());
        drop(token);
        assert_eq!(Arc::strong_count(&owner), 2);
        drop(clone);
        assert_eq!(Arc::strong_count(&owner), 1);
    }
}
