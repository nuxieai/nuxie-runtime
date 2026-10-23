//! Record picture modes during attachment, after the enclosing arena borrows end.
use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
};

use crate::source::core::CoreHandle;

#[derive(Default)]
struct Pending {
    borrows: usize,
    completing: bool,
    requests: VecDeque<Refresh>,
}

thread_local! {
    static PENDING: RefCell<Pending> = RefCell::new(Pending::default());
}

enum Refresh {
    Attached(CoreHandle),
    Geometry(CoreHandle),
}

// Only attachment consults the fixed hierarchy. CSS record presence, rather
// than registration order, decides the mode of the nearest box or participant.
// Malformed parent cycles have no enclosing layout context.
fn attachment_mode(host: &CoreHandle) -> bool {
    let mut visited = HashSet::new();
    let mut current = Some(host.clone());
    while let Some(object) = current {
        if !visited.insert(object.clone()) {
            return false;
        }
        let Some((mode, parent)) = object.with(|object| {
            let container = object.as_container_component();
            let boundary = object.as_layout_component().is_some()
                || container.is_some_and(|container| {
                    container.children().iter().any(|child| {
                        child.is_type_of(
                            crate::source::layout::layout_participant::LayoutParticipant::TYPE_KEY,
                        )
                    })
                });
            let mode = boundary.then(|| {
                container.is_some_and(|container| {
                    crate::css_layout::record::find_in(container.children()).is_some()
                })
            });
            (mode, object.component_parent_handle())
        }) else {
            return false;
        };
        if let Some(mode) = mode {
            return mode;
        }
        current = parent;
    }
    false
}

/// Both arena handles and runtime artboard handles lend RefCell receivers.
/// Nested callbacks collect work until the outermost receiver is released.
#[must_use]
pub(crate) struct BorrowScope(bool);

impl BorrowScope {
    pub(crate) fn new() -> Self {
        Self(
            PENDING
                .try_with(|pending| pending.borrow_mut().borrows += 1)
                .is_ok(),
        )
    }
}

impl Drop for BorrowScope {
    fn drop(&mut self) {
        if !self.0 {
            return;
        }
        let (complete, discarded) = PENDING
            .try_with(|pending| {
                let mut pending = pending.borrow_mut();
                pending.borrows -= 1;
                let discarded = if pending.borrows == 0 && std::thread::panicking() {
                    std::mem::take(&mut pending.requests)
                } else {
                    VecDeque::new()
                };
                let complete = pending.borrows == 0
                    && !std::thread::panicking()
                    && !pending.completing
                    && !pending.requests.is_empty();
                (complete, discarded)
            })
            .unwrap_or_default();
        drop(discarded);
        if complete {
            complete_pending();
        }
    }
}

struct Completing;
impl Drop for Completing {
    fn drop(&mut self) {
        let discarded = PENDING.try_with(|pending| {
            let mut pending = pending.borrow_mut();
            pending.completing = false;
            if std::thread::panicking() {
                std::mem::take(&mut pending.requests)
            } else {
                VecDeque::new()
            }
        });
        drop(discarded);
    }
}

fn collect(refresh: Refresh) {
    // At thread teardown there is no live scene left to refresh.
    let ready = PENDING
        .try_with(|pending| {
            let mut pending = pending.borrow_mut();
            pending.requests.push_back(refresh);
            pending.borrows == 0
        })
        .unwrap_or(false);
    if ready {
        complete_pending();
    }
}

fn complete_pending() {
    let ready = PENDING
        .try_with(|pending| {
            let mut pending = pending.borrow_mut();
            if pending.completing || pending.borrows != 0 || pending.requests.is_empty() {
                return false;
            }
            pending.completing = true;
            true
        })
        .unwrap_or(false);
    if !ready {
        return;
    }
    let _completing = Completing;
    loop {
        let request = PENDING.with(|pending| pending.borrow_mut().requests.pop_front());
        match request {
            Some(Refresh::Attached(host)) => {
                let sizeable = host
                    .with_mut(|host| host.as_intrinsically_sizeable_mut().is_some())
                    .unwrap_or(false);
                if !sizeable {
                    continue;
                }
                let css = attachment_mode(&host);
                host.with_mut(|host| {
                    if host
                        .as_component_mut()
                        .is_some_and(|component| component.record_picture_layout_mode(css))
                    {
                        if let Some(sizeable) = host.as_intrinsically_sizeable_mut() {
                            sizeable.layout_mode_changed();
                        }
                    }
                });
            }
            Some(Refresh::Geometry(host)) => {
                host.with_mut(|host| {
                    if let Some(sizeable) = host.as_intrinsically_sizeable_mut() {
                        sizeable.layout_mode_changed();
                    }
                });
            }
            None => break,
        }
    }
}

pub(crate) fn attached(host: CoreHandle) {
    collect(Refresh::Attached(host));
}

pub(crate) fn geometry(host: CoreHandle) {
    collect(Refresh::Geometry(host));
}
