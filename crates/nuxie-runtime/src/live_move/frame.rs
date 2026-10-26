//! Projection of the active scene into host VM graph walks. Built on membership
//! changes, then borrowed for one C API frame. A write to a sleeping model adds
//! only that model's ancestor path for this frame; it never loses rollback or
//! notification coverage merely because its Screen is asleep.
use crate::{
    CoreHandle, RuntimeOwnedViewModelHandle,
    source::viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_list::ViewModelInstanceList,
    },
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};
#[derive(Default)]
pub(super) struct Projection {
    excluded: HashSet<CoreHandle>,
    lists: HashMap<CoreHandle, (u64, Vec<CoreHandle>)>,
    sleeping_edges: HashMap<CoreHandle, Vec<CoreHandle>>,
    revision: u64,
    root: Option<CoreHandle>,
}
struct Scope {
    id: u64,
    projection: Rc<Projection>,
    touched: HashSet<CoreHandle>,
    extra: HashMap<CoreHandle, Vec<CoreHandle>>,
}
thread_local! {
    static FRAME: RefCell<Option<Scope>> = const { RefCell::new(None) };
    static NEXT_SCOPE: Cell<u64> = const { Cell::new(0) };
}
pub struct FrameScope(u64);
impl FrameScope {
    pub(super) fn refresh(&mut self, projection: Rc<Projection>) {
        FRAME.with(|slot| {
            let mut slot = slot.borrow_mut();
            let scope = slot.as_mut().expect("active live frame");
            assert_eq!(scope.id, self.0, "live frame owner");
            // Keep touched owners and explicit sleeping edges accumulated by
            // refresh callbacks; update only the shared scene projection.
            scope.projection = projection;
        });
    }
}
impl Drop for FrameScope {
    fn drop(&mut self) {
        FRAME.with(|s| {
            if s.borrow().as_ref().is_some_and(|scope| scope.id == self.0) {
                s.borrow_mut().take();
            }
        });
    }
}
impl Projection {
    pub(super) fn build(
        root: CoreHandle,
        roots: &[RuntimeOwnedViewModelHandle],
        excluded: HashSet<CoreHandle>,
    ) -> Rc<Self> {
        let mut result = Self {
            excluded,
            revision: crate::hosted_scene::membership_revision(&root),
            root: Some(root),
            ..Self::default()
        };
        let mut pending = roots.iter().map(|r| r.native_handle()).collect::<Vec<_>>();
        let mut seen = HashSet::new();
        while let Some(vm) = pending.pop() {
            if !seen.insert(vm.clone()) {
                continue;
            }
            let properties = vm
                .with_downcast::<ViewModelInstance, _>(|v| v.property_values().to_vec())
                .unwrap_or_default();
            for property in properties {
                if let Some(items) =
                    property.with_downcast::<ViewModelInstanceList, _>(|l| l.list_items().to_vec())
                {
                    let mut active = Vec::new();
                    let mut filtered = false;
                    for item in items {
                        if let Some(child) = item
                            .with(|o| {
                                o.as_view_model_instance_list_item()
                                    .and_then(|i| i.view_model_instance())
                            })
                            .flatten()
                        {
                            if result.excluded.contains(&child) {
                                filtered = true;
                                result
                                    .sleeping_edges
                                    .entry(child.clone())
                                    .or_default()
                                    .push(property.clone());
                            } else {
                                active.push(child.clone());
                            }
                            pending.push(child);
                        }
                    }
                    if filtered {
                        // Native change sequences cover membership edits and
                        // rollback. No global observer or ordinary-file hook.
                        let version = property
                            .with(|o| o.as_view_model_instance_value().unwrap().change_sequence())
                            .unwrap();
                        result.lists.insert(property.clone(), (version, active));
                    }
                } else if let Some(child) = property
                    .with(|o| {
                        o.as_view_model_instance_view_model()
                            .and_then(|v| v.reference_view_model_instance())
                    })
                    .flatten()
                {
                    pending.push(child);
                }
            }
        }
        Rc::new(result)
    }
    pub(super) fn is_valid(&self) -> bool {
        self.lists.iter().all(|(property, (version, _))| {
            property
                .with(|o| {
                    o.as_view_model_instance_value()
                        .map(|v| v.change_sequence())
                        == Some(*version)
                })
                .unwrap_or(false)
        })
    }
    pub(super) fn enter(self: &Rc<Self>) -> FrameScope {
        let id = NEXT_SCOPE.with(|n| {
            let id = n.get().wrapping_add(1);
            n.set(id);
            id
        });
        FRAME.with(|s| {
            assert!(s.borrow().is_none(), "nested live frame");
            *s.borrow_mut() = Some(Scope {
                id,
                projection: self.clone(),
                touched: HashSet::new(),
                extra: HashMap::new(),
            });
        });
        FrameScope(id)
    }
}
pub(crate) fn is_active() -> bool {
    FRAME.with(|scope| scope.borrow().is_some())
}
pub(crate) fn clear_for(root: &CoreHandle) {
    FRAME.with(|frame| {
        let matches = frame
            .borrow()
            .as_ref()
            .is_some_and(|s| s.projection.root.as_ref() == Some(root));
        if matches {
            frame.borrow_mut().take();
        }
    });
}
pub(crate) fn graph_excluded(owner: &CoreHandle) -> bool {
    FRAME.with(|s| {
        s.borrow()
            .as_ref()
            .is_some_and(|s| s.projection.excluded.contains(owner) && !s.touched.contains(owner))
    })
}
pub(crate) fn graph_list_children(property: &CoreHandle) -> Option<Vec<CoreHandle>> {
    FRAME.with(|s| {
        let scope = s.borrow();
        let scope = scope.as_ref()?;
        if !scope.projection.is_valid()
            || scope.projection.revision
                != crate::hosted_scene::membership_revision(scope.projection.root.as_ref()?)
        {
            return None;
        }
        let mut children = scope.projection.lists.get(property)?.1.clone();
        children.extend(scope.extra.get(property).into_iter().flatten().cloned());
        Some(children)
    })
}
pub(crate) fn touch(owner: &CoreHandle) {
    let active = FRAME.with(|s| {
        s.borrow()
            .as_ref()
            .is_some_and(|scope| !scope.touched.contains(owner))
    });
    if !active {
        return;
    }
    let mut pending = vec![owner.clone()];
    let mut seen = HashSet::new();
    while let Some(owner) = pending.pop() {
        if !seen.insert(owner.clone()) {
            continue;
        }
        let already = FRAME.with(|s| {
            let mut scope = s.borrow_mut();
            let scope = scope.as_mut().expect("frame scope");
            if !scope.touched.insert(owner.clone()) {
                return true;
            }
            if let Some(edges) = scope.projection.sleeping_edges.get(&owner) {
                for edge in edges {
                    scope
                        .extra
                        .entry(edge.clone())
                        .or_default()
                        .push(owner.clone());
                }
            }
            false
        });
        if already {
            continue;
        }
        pending.extend(
            owner
                .with_downcast::<ViewModelInstance, _>(ViewModelInstance::parents)
                .unwrap_or_default(),
        );
    }
}

#[cfg(feature = "test-support")]
pub(crate) fn active_scope_count() -> usize {
    FRAME.with(|s| usize::from(s.borrow().is_some()))
}
