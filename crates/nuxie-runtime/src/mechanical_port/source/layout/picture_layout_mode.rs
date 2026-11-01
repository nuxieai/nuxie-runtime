//! Record picture modes during attachment, after the attaching receivers are
//! released.
//!
//! Only attachment requests a picture's mode refresh: `CoreObject`'s
//! `on_added_dirty` and `on_added_clean` for an intrinsically sizeable object,
//! and the initialization pass that caches every mode once all records are
//! attached. Those callbacks hold their own object mutably, and the refresh
//! reads that object and its ancestors and updates its fitted geometry.
//!
//! In the runtime every attachment runs inside `Artboard::initialize_handle`,
//! which opens a [`Batch`]: callbacks queue their refreshes, and initialization
//! completes them wherever it holds no receiver, after each per-object
//! callback and after the caching pass. Those are the points where the last
//! receiver used to be released while every `CoreHandle` access counted
//! borrows, so refreshes keep their order and timing, and initialization never
//! returns with one outstanding: nothing after it can observe an unrecorded
//! mode. Plain `CoreHandle::with`/`with_mut` keep no count.
//!
//! A batch completes only the requests collected since it opened, so a nested
//! initialization never refreshes an outer object whose callback still holds
//! it. A batch left by an early return completes the rest; one unwound by a
//! panic discards them. Outside any batch (a callback driven directly, as unit
//! tests do) a callback refreshes its own receiver in place, and a refresh by
//! handle completes at once. After thread teardown requests are dropped.
use std::{cell::RefCell, collections::HashSet};

use crate::source::core::{CoreHandle, CoreObject};

#[derive(Default)]
struct Pending {
    batches: usize,
    completing: bool,
    requests: Vec<Refresh>,
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
    host.with(mode_step)
        .is_some_and(|step| mode_from(host, step))
}

/// The mode `object` decides as a box or participant, and its parent.
fn mode_step<O: CoreObject + ?Sized>(object: &O) -> (Option<bool>, Option<CoreHandle>) {
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
}

/// Finish the walk from `host`, given its own step.
fn mode_from(host: &CoreHandle, (mode, parent): (Option<bool>, Option<CoreHandle>)) -> bool {
    if let Some(mode) = mode {
        return mode;
    }
    let mut visited = HashSet::from([host.clone()]);
    let mut current = parent;
    while let Some(object) = current {
        if !visited.insert(object.clone()) {
            return false;
        }
        let Some((mode, parent)) = object.with(mode_step) else {
            return false;
        };
        if let Some(mode) = mode {
            return mode;
        }
        current = parent;
    }
    false
}

/// Defers the refreshes an initialization requests to its receiver-free points.
#[must_use]
pub(crate) struct Batch(Option<usize>);

impl Batch {
    pub(crate) fn open() -> Self {
        Self(
            PENDING
                .try_with(|pending| {
                    let mut pending = pending.borrow_mut();
                    pending.batches += 1;
                    pending.requests.len()
                })
                .ok(),
        )
    }

    /// Complete this batch's requests. The caller holds no receiver.
    pub(crate) fn complete(&self) {
        if let Some(mark) = self.0 {
            complete_from(mark);
        }
    }
}

impl Drop for Batch {
    fn drop(&mut self) {
        let Some(mark) = self.0 else {
            return;
        };
        let panicking = std::thread::panicking();
        let discarded = PENDING.try_with(|pending| {
            let mut pending = pending.borrow_mut();
            pending.batches -= 1;
            let mark = mark.min(pending.requests.len());
            if panicking {
                pending.requests.split_off(mark)
            } else {
                Vec::new()
            }
        });
        drop(discarded);
        if !panicking {
            complete_from(mark);
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
                Vec::new()
            }
        });
        drop(discarded);
    }
}

/// Queue `refresh` while a batch is open. Otherwise hand it back to be
/// completed now; at thread teardown drop it.
fn collect(refresh: Refresh) -> Option<Refresh> {
    PENDING
        .try_with(|pending| {
            let mut pending = pending.borrow_mut();
            if pending.batches == 0 {
                return Some(refresh);
            }
            pending.requests.push(refresh);
            None
        })
        .unwrap_or(None)
}

/// Complete the requests at `mark` and after, in request order.
fn complete_from(mark: usize) {
    let ready = PENDING
        .try_with(|pending| {
            let mut pending = pending.borrow_mut();
            if pending.completing || pending.requests.len() <= mark {
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
        let requests = PENDING.with(|pending| {
            let mut pending = pending.borrow_mut();
            let mark = mark.min(pending.requests.len());
            pending.requests.split_off(mark)
        });
        if requests.is_empty() {
            break;
        }
        requests.into_iter().for_each(refresh);
    }
}

fn refresh(request: Refresh) {
    match request {
        Refresh::Attached(host) => {
            let sizeable = host
                .with_mut(|host| host.as_intrinsically_sizeable_mut().is_some())
                .unwrap_or(false);
            if !sizeable {
                return;
            }
            let css = attachment_mode(&host);
            host.with_mut(|host| record(host, css));
        }
        Refresh::Geometry(host) => {
            host.with_mut(geometry_changed);
        }
    }
}

fn record<O: CoreObject + ?Sized>(object: &mut O, css: bool) {
    if object
        .as_component_mut()
        .is_some_and(|component| component.record_picture_layout_mode(css))
    {
        geometry_changed(object);
    }
}

fn geometry_changed<O: CoreObject + ?Sized>(object: &mut O) {
    if let Some(sizeable) = object.as_intrinsically_sizeable_mut() {
        sizeable.layout_mode_changed();
    }
}

/// Record `host`'s mode, which must be released.
pub(crate) fn attached(host: CoreHandle) {
    if let Some(request) = collect(Refresh::Attached(host)) {
        refresh(request);
    }
}

/// An attachment callback records its sizeable receiver's mode.
pub(crate) fn receiver_attached<O: CoreObject + ?Sized>(object: &mut O) {
    let Some(host) = object.core().handle() else {
        return;
    };
    if collect(Refresh::Attached(host.clone())).is_some() {
        let css = mode_from(&host, mode_step(&*object));
        record(object, css);
    }
}

/// An attachment callback refits its sizeable receiver to its mode.
pub(crate) fn receiver_geometry<O: CoreObject + ?Sized>(object: &mut O) {
    let Some(host) = object.core().handle() else {
        return;
    };
    if collect(Refresh::Geometry(host)).is_some() {
        geometry_changed(object);
    }
}
