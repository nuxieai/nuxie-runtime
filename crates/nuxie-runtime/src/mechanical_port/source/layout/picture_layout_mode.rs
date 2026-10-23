//! Complete picture cache changes after the enclosing arena borrows end.
use std::{cell::RefCell, collections::VecDeque};

use crate::source::{container_component::ContainerComponent, core::CoreHandle};

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
    Inherited(CoreHandle, Option<CoreHandle>),
    Boundary(CoreHandle, bool),
    Children(Vec<CoreHandle>, bool, Option<CoreHandle>),
    Geometry(CoreHandle),
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
            Some(Refresh::Inherited(host, parent)) => {
                let css = parent
                    .as_ref()
                    .and_then(|parent| {
                        parent.with(|parent| {
                            parent
                                .as_component()
                                .map(|component| component.picture_uses_css_layout())
                        })
                    })
                    .flatten()
                    .unwrap_or(false);
                host.with_mut(|host| {
                    let Some(component) = host.as_component_mut() else {
                        return;
                    };
                    if component.parent_handle() != parent
                        || !component.inherit_picture_layout_mode(css)
                    {
                        return;
                    }
                    if let Some(container) = host.as_container_component_mut() {
                        container.refresh_attached_picture_layout_mode();
                    }
                    if let Some(sizeable) = host.as_intrinsically_sizeable_mut() {
                        sizeable.layout_mode_changed();
                    }
                });
            }
            Some(Refresh::Boundary(host, css)) => {
                host.with_mut(|host| {
                    if let Some(container) = host.as_container_component_mut() {
                        container.refresh_picture_layout_mode(css);
                    }
                    if let Some(sizeable) = host.as_intrinsically_sizeable_mut() {
                        sizeable.layout_mode_changed();
                    }
                });
            }
            Some(Refresh::Children(children, css, parent)) => {
                ContainerComponent::refresh_picture_children(&children, css, parent.as_ref());
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

pub(crate) fn inherited(host: CoreHandle, parent: Option<CoreHandle>) {
    collect(Refresh::Inherited(host, parent));
}

pub(crate) fn boundary(host: CoreHandle, css: bool) {
    collect(Refresh::Boundary(host, css));
}

pub(crate) fn children(children: Vec<CoreHandle>, css: bool, parent: Option<CoreHandle>) {
    if !children.is_empty() {
        collect(Refresh::Children(children, css, parent));
    }
}

pub(crate) fn geometry(host: CoreHandle) {
    collect(Refresh::Geometry(host));
}
