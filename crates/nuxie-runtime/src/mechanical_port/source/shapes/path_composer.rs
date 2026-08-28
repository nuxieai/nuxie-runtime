use crate::mechanical_port::source::{
    component::{Component, ComponentOccurrenceHandle},
    component_dirt::ComponentDirt,
    core::CoreHandle,
    shapes::{paint::shape_paint_path::ShapePaintPath, path_flags::PathFlags},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct RuntimePathComposerHandle(Rc<RefCell<PathComposer>>);

impl RuntimePathComposerHandle {
    pub fn new() -> Self {
        Self(Rc::new_cyclic(|weak| {
            let mut component = Component::default();
            component
                .bind_runtime_occurrence(ComponentOccurrenceHandle::PathComposer(weak.clone()));
            RefCell::new(PathComposer {
                component,
                shape: None,
                local_path: ShapePaintPath::new(true),
                world_path: ShapePaintPath::new(false),
                local_clockwise_path: ShapePaintPath::new(true),
                deferred_path_dirt: false,
            })
        }))
    }
    pub fn occurrence(&self) -> ComponentOccurrenceHandle {
        ComponentOccurrenceHandle::PathComposer(Rc::downgrade(&self.0))
    }
    pub fn with<R>(&self, f: impl FnOnce(&PathComposer) -> R) -> R {
        f(&self.0.borrow())
    }
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut PathComposer) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }
    pub fn bind_shape(&self, shape: CoreHandle) {
        self.0.borrow_mut().shape = Some(shape);
    }
    pub fn add_dependent(&self, dependent: impl Into<ComponentOccurrenceHandle>) {
        self.0.borrow_mut().component.add_dependent(dependent);
    }
    pub fn add_dirt_from_shape(
        &self,
        shape: &mut crate::mechanical_port::source::shapes::shape::Shape,
        value: ComponentDirt,
        recurse: bool,
    ) -> bool {
        let dirty = self.with_mut(|helper| helper.component.add_dirt_state(value));
        let Some(_) = dirty else {
            return false;
        };
        if self.with(|helper| helper.deferred_path_dirt) {
            shape.path_changed();
        }
        let occurrence = self.occurrence();
        occurrence.notify_artboard();
        if recurse {
            for dependent in occurrence
                .with_component(Component::dependents_snapshot)
                .unwrap_or_default()
            {
                dependent.add_dirt(value, true);
            }
        }
        true
    }
    pub fn collapse_from_shape(
        &self,
        shape: &mut crate::mechanical_port::source::shapes::shape::Shape,
        value: bool,
    ) -> bool {
        if self
            .with_mut(|helper| helper.component.collapse_state(value))
            .is_none()
        {
            return false;
        }
        if self.with(|helper| helper.deferred_path_dirt) {
            shape.path_changed();
        }
        self.occurrence().notify_artboard();
        self.with_mut(|helper| helper.component.update_collapsables());
        true
    }
}
impl Default for RuntimePathComposerHandle {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PathComposer {
    pub component: Component,
    shape: Option<CoreHandle>,
    local_path: ShapePaintPath,
    world_path: ShapePaintPath,
    local_clockwise_path: ShapePaintPath,
    deferred_path_dirt: bool,
}

impl PathComposer {
    pub fn shape(&self) -> CoreHandle {
        self.shape.clone().expect("arena-installed Shape")
    }

    pub(crate) fn dirty_shape(&self) -> Option<CoreHandle> {
        self.deferred_path_dirt.then(|| self.shape())
    }

    /// Shape adds itself before this tail, while it already has its own mutable
    /// borrow. The remaining path edges require no reborrow of that Shape slot.
    pub(crate) fn build_path_dependencies(&mut self, paths: &[CoreHandle]) {
        let dependent = self
            .component
            .occurrence_handle()
            .expect("PathComposer occurrence");
        for path in paths {
            path.with_mut(|path| {
                path.as_component_mut()
                    .expect("Shape path Component")
                    .add_dependent(dependent.clone());
            });
        }
    }

    pub fn update(&mut self, value: ComponentDirt) -> Option<CoreHandle> {
        if !value.intersects(ComponentDirt::PATH | ComponentDirt::N_SLICER) {
            return None;
        }
        let shape_handle = self.shape();
        let (can_defer, local, clockwise, world, transform, paths) = shape_handle
            .with(|shape| {
                let shape = shape.as_shape().expect("PathComposer Shape");
                (
                    shape.can_defer_path_update(),
                    shape.is_flagged(PathFlags::LOCAL),
                    shape.is_flagged(PathFlags::LOCAL_CLOCKWISE),
                    shape.is_flagged(PathFlags::WORLD),
                    *shape.world_transform(),
                    shape.paths(),
                )
            })
            .expect("live PathComposer Shape");
        if can_defer {
            self.deferred_path_dirt = true;
            return None;
        }
        self.deferred_path_dirt = false;
        if local {
            self.local_path.rewind();
            let inverse = transform.invert_or_identity();
            for handle in &paths {
                handle.with(|object| {
                    let path = object.as_path().expect("Shape path");
                    if !path.is_hidden() && !path.is_collapsed() {
                        let path_transform = object
                            .as_points_path()
                            .map(|points| *points.path_transform())
                            .unwrap_or_else(|| path.path_transform());
                        self.local_path
                            .add_path(path.raw_path(), Some(&(inverse * path_transform)));
                    }
                });
            }
        }
        if clockwise {
            self.local_clockwise_path.rewind();
            let inverse = transform.invert_or_identity();
            for handle in &paths {
                handle.with(|object| {
                    let path = object.as_path().expect("Shape path");
                    if path.is_hidden() || path.is_collapsed() {
                        return;
                    }
                    let path_transform = object
                        .as_points_path()
                        .map(|points| *points.path_transform())
                        .unwrap_or_else(|| path.path_transform());
                    let local_transform = inverse * path_transform;
                    let not_clockwise = object.as_points_path().is_some_and(|points| {
                        local_transform.determinant()
                            * if points.is_clockwise() { 1.0 } else { -1.0 }
                            < 0.0
                    });
                    if not_clockwise != path.is_hole() {
                        self.local_clockwise_path
                            .add_path_backwards(path.raw_path(), Some(&local_transform));
                    } else {
                        self.local_clockwise_path
                            .add_path(path.raw_path(), Some(&local_transform));
                    }
                });
            }
        }
        if world {
            self.world_path.rewind();
            for handle in &paths {
                handle.with(|object| {
                    let path = object.as_path().expect("Shape path");
                    if !path.is_hidden() && !path.is_collapsed() {
                        let path_transform = object
                            .as_points_path()
                            .map(|points| *points.path_transform())
                            .unwrap_or_else(|| path.path_transform());
                        self.world_path
                            .add_path(path.raw_path(), Some(&path_transform));
                    }
                });
            }
        }
        Some(shape_handle)
    }

    pub fn local_path(&mut self) -> &mut ShapePaintPath {
        &mut self.local_path
    }
    pub fn world_path(&mut self) -> &mut ShapePaintPath {
        &mut self.world_path
    }
    pub fn local_clockwise_path(&mut self) -> &mut ShapePaintPath {
        &mut self.local_clockwise_path
    }
}
