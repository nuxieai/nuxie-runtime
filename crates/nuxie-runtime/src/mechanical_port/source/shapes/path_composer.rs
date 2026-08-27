use crate::mechanical_port::source::{
    component::{Component, ComponentDirt, has_dirt},
    math::mat2d::Mat2D,
    shapes::{
        path_flags::PathFlags, points_path::PointsPath, shape::Shape,
        shape_paint_path::ShapePaintPath,
    },
};

pub struct PathComposer {
    pub component: Component,
    shape: Shape,
    local_path: ShapePaintPath,
    world_path: ShapePaintPath,
    local_clockwise_path: ShapePaintPath,
    deferred_path_dirt: bool,
}

impl PathComposer {
    pub fn new(shape: Shape) -> Self {
        Self {
            component: Component::default(),
            shape,
            local_path: ShapePaintPath::new(true),
            world_path: ShapePaintPath::new(false),
            local_clockwise_path: ShapePaintPath::new(true),
            deferred_path_dirt: false,
        }
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn build_dependencies(&mut self) {
        self.shape.add_dependent(&mut self.component);
        for path in self.shape.paths_mut() {
            path.add_dependent(&mut self.component);
        }
    }

    pub fn on_dirty(&mut self, _dirt: ComponentDirt) {
        if self.deferred_path_dirt {
            self.shape.path_changed();
        }
    }

    pub fn update(&mut self, value: ComponentDirt) {
        if !has_dirt(value, ComponentDirt::PATH | ComponentDirt::N_SLICER) {
            return;
        }
        if self.shape.can_defer_path_update() {
            self.deferred_path_dirt = true;
            return;
        }
        self.deferred_path_dirt = false;

        if self.shape.is_flagged(PathFlags::LOCAL) {
            self.local_path.rewind();
            let inverse_world = self.shape.world_transform().invert_or_identity();
            for path in self.shape.paths() {
                if !path.is_hidden() && !path.is_collapsed() {
                    let local_transform = inverse_world * path.path_transform();
                    self.local_path.add_path(path.raw_path(), local_transform);
                }
            }
        }
        if self.shape.is_flagged(PathFlags::LOCAL_CLOCKWISE) {
            self.local_clockwise_path.rewind();
            let inverse_world = self.shape.world_transform().invert_or_identity();
            for path in self.shape.paths() {
                if path.is_hidden() || path.is_collapsed() {
                    continue;
                }
                let local_transform = inverse_world * path.path_transform();
                let is_not_clockwise = path
                    .as_points_path()
                    .map(|points: &PointsPath| {
                        local_transform.determinant()
                            * if points.is_clockwise() { 1.0 } else { -1.0 }
                            < 0.0
                    })
                    .unwrap_or(false);
                if is_not_clockwise != path.is_hole() {
                    self.local_clockwise_path
                        .add_path_backwards(path.raw_path(), local_transform);
                } else {
                    self.local_clockwise_path
                        .add_path(path.raw_path(), local_transform);
                }
            }
        }
        if self.shape.is_flagged(PathFlags::WORLD) {
            self.world_path.rewind();
            for path in self.shape.paths() {
                if !path.is_hidden() && !path.is_collapsed() {
                    self.world_path
                        .add_path(path.raw_path(), path.path_transform());
                }
            }
        }
        self.shape.mark_bounds_dirty();
    }

    pub fn path_collapse_changed(&mut self) {
        self.component.add_dirt(ComponentDirt::PATH);
        for dependent in self.component.dependents_mut() {
            dependent.add_dirt_with_recurse(ComponentDirt::PATH, true);
        }
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
