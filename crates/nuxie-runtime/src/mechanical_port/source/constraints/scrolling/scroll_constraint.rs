use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    artboard_component_list::ArtboardComponentList,
    backboard::BackboardBase,
    constraints::{
        draggable_constraint::{DraggableConstraintDirection, DraggableProxy},
        scrolling::{
            scroll_constraint_proxy::ViewportDraggableProxy,
            scroll_physics::{ScrollPhysics, ScrollPhysicsType},
            scroll_virtualizer::ScrollVirtualizer,
        },
        transform_constraint::TransformConstraint,
    },
    core::{Core, CoreClone},
    core_context::{CoreContext, StatusCode},
    generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    layout::layout_node_provider::LayoutNodeProvider,
    layout_component::LayoutComponent,
    math::{
        aabb::Aabb, mat2d::Mat2D, math_types, transform_components::TransformComponents,
        vec2d::Vec2D,
    },
    transform_component::TransformComponent,
    virtualizing_component::VirtualizedDirection,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScrollSpace {
    None,
    Percent,
    Index,
}

#[derive(Clone, Copy)]
struct ScrollAxisIntent {
    space: ScrollSpace,
    value: f32,
}
impl Default for ScrollAxisIntent {
    fn default() -> Self {
        Self {
            space: ScrollSpace::None,
            value: 0.0,
        }
    }
}

pub struct ScrollConstraint {
    pub base: ScrollConstraintBase,
    physics: Option<Box<dyn ScrollPhysics>>,
    virtualizer: Option<Box<ScrollVirtualizer>>,
    layout_children: Vec<*mut dyn LayoutNodeProvider>,
    components_a: TransformComponents,
    components_b: TransformComponents,
    scroll_transform: Mat2D,
    offset_x: f32,
    offset_y: f32,
    last_frame_offset_x: f32,
    last_frame_offset_y: f32,
    child_constraint_applied_count: i32,
    is_dragging: bool,
    is_scroll_bar_dragging: bool,
    has_list_children: bool,
    intent_x: ScrollAxisIntent,
    intent_y: ScrollAxisIntent,
}

impl Drop for ScrollConstraint {
    fn drop(&mut self) {
        self.virtualizer = None;
        self.layout_children.clear();
        self.physics = None;
    }
}

impl ScrollConstraint {
    pub fn content_width(&self) -> f32 {
        if self.base.virtualize() && !self.main_axis_is_column() {
            let mut content_size = 0.0;
            for child in self.layout_children.iter().copied() {
                if child.is_null() {
                    continue;
                }
                content_size += unsafe { (*child).layout_bounds().width() };
            }
            let len_offset = if self.base.infinite() { 0 } else { 1 };
            content_size +=
                self.gap().x * self.layout_children.len().wrapping_sub(len_offset) as f32;
            if !self.base.infinite() {
                content_size += self.content().padding_left() + self.content().padding_right();
            }
            return content_size;
        }
        self.content().layout_width()
    }

    pub fn content_height(&self) -> f32 {
        if self.base.virtualize() && self.main_axis_is_column() {
            let mut content_size = 0.0;
            for child in self.layout_children.iter().copied() {
                if child.is_null() {
                    continue;
                }
                content_size += unsafe { (*child).layout_bounds().height() };
            }
            let len_offset = if self.base.infinite() { 0 } else { 1 };
            content_size +=
                self.gap().y * self.layout_children.len().wrapping_sub(len_offset) as f32;
            if !self.base.infinite() {
                content_size += self.content().padding_top() + self.content().padding_bottom();
            }
            return content_size;
        }
        self.content().layout_height()
    }

    pub fn viewport_width(&self) -> f32 {
        if self.direction() == DraggableConstraintDirection::Vertical {
            self.viewport().layout_width()
        } else {
            0.0_f32.max(self.viewport().layout_width() - self.content().layout_x())
        }
    }
    pub fn viewport_height(&self) -> f32 {
        if self.direction() == DraggableConstraintDirection::Horizontal {
            self.viewport().layout_height()
        } else {
            0.0_f32.max(self.viewport().layout_height() - self.content().layout_y())
        }
    }
    pub fn visible_width_ratio(&self) -> f32 {
        if self.content_width() == 0.0 {
            1.0
        } else {
            1.0_f32.min(self.viewport_width() / self.content_width())
        }
    }
    pub fn visible_height_ratio(&self) -> f32 {
        if self.content_height() == 0.0 {
            1.0
        } else {
            1.0_f32.min(self.viewport_height() / self.content_height())
        }
    }
    pub fn min_offset_x(&self) -> f32 {
        if self.base.infinite() && !self.main_axis_is_column() {
            f32::INFINITY
        } else {
            0.0
        }
    }
    pub fn min_offset_y(&self) -> f32 {
        if self.base.infinite() && self.main_axis_is_column() {
            f32::INFINITY
        } else {
            0.0
        }
    }
    pub fn max_offset_x(&self) -> f32 {
        if self.base.infinite() && !self.main_axis_is_column() {
            f32::NEG_INFINITY
        } else {
            0.0_f32
                .min(self.viewport_width() - self.content_width() - self.viewport().padding_right())
        }
    }
    pub fn max_offset_y(&self) -> f32 {
        if self.base.infinite() && self.main_axis_is_column() {
            f32::NEG_INFINITY
        } else {
            0.0_f32.min(
                self.viewport_height() - self.content_height() - self.viewport().padding_bottom(),
            )
        }
    }

    pub fn clamped_offset_x(&self) -> f32 {
        if self.base.infinite() {
            return self.offset_x;
        }
        if self.max_offset_x() > 0.0 {
            return 0.0;
        }
        if let Some(physics) = &self.physics {
            if physics.enabled() {
                return physics
                    .clamp(
                        Vec2D::new(self.max_offset_x(), self.max_offset_y()),
                        Vec2D::new(self.min_offset_x(), self.min_offset_y()),
                        Vec2D::new(self.offset_x, self.offset_y),
                    )
                    .x;
            }
        }
        math_types::clamp(self.offset_x, self.max_offset_x(), 0.0)
    }
    pub fn clamped_offset_y(&self) -> f32 {
        if self.base.infinite() {
            return self.offset_y;
        }
        if self.max_offset_y() > 0.0 {
            return 0.0;
        }
        if let Some(physics) = &self.physics {
            if physics.enabled() {
                return physics
                    .clamp(
                        Vec2D::new(self.max_offset_x(), self.max_offset_y()),
                        Vec2D::new(self.min_offset_x(), self.min_offset_y()),
                        Vec2D::new(self.offset_x, self.offset_y),
                    )
                    .y;
            }
        }
        math_types::clamp(self.offset_y, self.max_offset_y(), 0.0)
    }

    pub fn offset_x(&self) -> f32 {
        self.offset_x
    }
    pub fn offset_y(&self) -> f32 {
        self.offset_y
    }
    pub fn set_offset_x(&mut self, value: f32) {
        if self.offset_x == value {
            return;
        }
        self.offset_x = value;
        self.content_mut().mark_world_transform_dirty();
    }
    pub fn set_offset_y(&mut self, value: f32) {
        if self.offset_y == value {
            return;
        }
        self.offset_y = value;
        self.content_mut().mark_world_transform_dirty();
    }
    pub fn main_axis_is_column(&self) -> bool {
        self.content().main_axis_is_column()
    }

    pub fn constrain(&mut self, _component: &mut TransformComponent) {
        self.resolve_scroll_intents();
        self.scroll_transform = Mat2D::from_translate(
            if self.base.constrains_horizontal() {
                self.clamped_offset_x()
            } else {
                0.0
            },
            if self.base.constrains_vertical() {
                self.clamped_offset_y()
            } else {
                0.0
            },
        );
        self.child_constraint_applied_count = 0;
    }

    pub fn constrain_child(&mut self, child: &mut dyn LayoutNodeProvider) {
        let Some(component) = child.transform_component_mut() else {
            return;
        };
        let target = Mat2D::multiply(*component.world_transform(), self.scroll_transform);
        TransformConstraint::constrain_world(
            component,
            *component.world_transform(),
            self.components_a,
            target,
            self.components_b,
            self.base.strength(),
        );
        self.child_constraint_applied_count += 1;
        self.constrain_virtualized(false);
    }

    pub fn constrain_virtualized(&mut self, force: bool) {
        if self.base.virtualize() && self.virtualizer.is_some() {
            if self.child_constraint_applied_count < self.layout_children.len() as i32 && !force {
                return;
            }
            let column = self.main_axis_is_column();
            let direction = if column {
                VirtualizedDirection::Vertical
            } else {
                VirtualizedDirection::Horizontal
            };
            let offset = if column {
                self.clamped_offset_y()
            } else {
                self.clamped_offset_x()
            };
            let mut children = self.layout_children.clone();
            self.virtualizer
                .as_mut()
                .unwrap()
                .constrain(self, &mut children, offset, direction);
        }
    }
    pub fn add_layout_child(&mut self, child: &mut dyn LayoutNodeProvider) {
        self.layout_children.push(child);
    }

    pub fn drag_view(&mut self, delta: Vec2D, time_stamp: f32) {
        let scaled = Vec2D::new(
            delta.x * self.base.drag_multiplier(),
            delta.y * self.base.drag_multiplier(),
        );
        if let Some(physics) = &mut self.physics {
            physics.accumulate(scaled, time_stamp);
            self.set_authored_scroll_offset_x(self.offset_x() + scaled.x);
            self.set_authored_scroll_offset_y(self.offset_y() + scaled.y);
            return;
        }
        let mut x = self.offset_x() + scaled.x;
        let mut y = self.offset_y() + scaled.y;
        if !self.base.infinite() {
            x = if self.max_offset_x() > 0.0 {
                0.0
            } else {
                math_types::clamp(x, self.max_offset_x(), 0.0)
            };
            y = if self.max_offset_y() > 0.0 {
                0.0
            } else {
                math_types::clamp(y, self.max_offset_y(), 0.0)
            };
        }
        self.set_authored_scroll_offset_x(x);
        self.set_authored_scroll_offset_y(y);
    }

    fn collect_snap_points(&self) -> Vec<Vec2D> {
        let mut points = Vec::new();
        for child in self.layout_children.iter().copied() {
            if child.is_null() {
                continue;
            }
            for node in 0..unsafe { (*child).num_layout_nodes() } {
                let bounds = unsafe { (*child).layout_bounds_for_node(node) };
                if !self.is_bounds_collapsed(bounds) {
                    points.push(Vec2D::new(bounds.left(), bounds.top()));
                }
            }
        }
        points
    }

    pub fn run_physics(&mut self) {
        self.is_dragging = false;
        let points = if self.base.snap() {
            self.collect_snap_points()
        } else {
            Vec::new()
        };
        let column = self.main_axis_is_column();
        let args = (
            Vec2D::new(self.max_offset_x(), self.max_offset_y()),
            Vec2D::new(self.min_offset_x(), self.min_offset_y()),
            Vec2D::new(self.offset_x(), self.offset_y()),
            if column {
                self.content_height()
            } else {
                self.content_width()
            },
            if column {
                self.viewport_height()
            } else {
                self.viewport_width()
            },
        );
        if let Some(physics) = &mut self.physics {
            physics.run(args.0, args.1, args.2, points, args.3, args.4);
        }
    }

    pub fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        if !flags.contains(AdvanceFlags::ADVANCE_NESTED) || self.base.is_collapsed() {
            return false;
        }
        let Some(physics) = &mut self.physics else {
            return false;
        };
        if physics.is_running() {
            let offset = physics.advance(elapsed_seconds);
            self.set_authored_scroll_offset_x(offset.x);
            self.set_authored_scroll_offset_y(offset.y);
        }
        if flags.contains(AdvanceFlags::NEW_FRAME) {
            let moved = self.authored_scroll_offset_x() != self.last_frame_offset_x
                || self.authored_scroll_offset_y() != self.last_frame_offset_y;
            if (self.is_scroll_bar_dragging || self.is_dragging) && !moved {
                self.clear_velocity();
            }
            self.last_frame_offset_x = self.authored_scroll_offset_x();
            self.last_frame_offset_y = self.authored_scroll_offset_y();
        }
        self.physics.as_ref().unwrap().enabled() || self.is_scroll_bar_dragging || self.is_dragging
    }

    pub fn draggables(&mut self) -> Vec<Box<dyn DraggableProxy>> {
        let proxy = self.viewport_mut().proxy_mut();
        vec![Box::new(ViewportDraggableProxy::new(self, proxy))]
    }

    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        self.has_list_children = false;
        let children = self.content_mut().children_mut_ptrs();
        for child in children {
            unsafe {
                if let Some(layout) = LayoutNodeProvider::from(&mut *child) {
                    self.base.add_dependent(&mut *child);
                    layout.add_layout_constraint(self.as_layout_constraint_mut_ptr());
                }
                if (*child).is::<ArtboardComponentList>() {
                    self.has_list_children = true;
                }
            }
        }
    }

    pub fn clone_core(&self) -> Box<dyn Core> {
        let mut cloned = self.base.clone_core();
        if let Some(physics) = &self.physics {
            cloned.as_mut::<ScrollConstraint>().unwrap().physics = Some(physics.clone_physics());
        }
        cloned
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        if let Some(importer) = import_stack.latest::<BackboardImporter>(BackboardBase::TYPE_KEY) {
            let objects = importer.physics();
            let id = self.base.physics_id();
            if id != -1 && (id as usize) < objects.len() {
                if let Some(physics) = objects[id as usize] {
                    self.physics = Some(unsafe { (&*physics).clone_physics() });
                }
            }
        } else {
            return StatusCode::MissingObject;
        }
        self.base.import(import_stack)
    }

    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let result = self.base.on_added_dirty(context);
        if self.base.virtualize() {
            self.virtualizer = Some(Box::new(ScrollVirtualizer::default()));
        }
        self.set_offset_x(self.authored_scroll_offset_x());
        self.set_offset_y(self.authored_scroll_offset_y());
        result
    }

    pub fn init_physics(&mut self) {
        self.is_dragging = true;
        self.clear_scroll_intents();
        self.last_frame_offset_x = self.authored_scroll_offset_x();
        self.last_frame_offset_y = self.authored_scroll_offset_y();
        let direction = self.direction();
        if let Some(physics) = &mut self.physics {
            physics.prepare(direction);
        }
    }
    pub fn stop_physics(&mut self) {
        if let Some(physics) = &mut self.physics {
            physics.reset();
        }
    }
    pub fn clear_velocity(&mut self) {
        if let Some(physics) = &mut self.physics {
            physics.clear_velocity();
        }
    }
    fn max_offset_x_for_percent(&self) -> f32 {
        if self.base.infinite() {
            self.content_width()
        } else {
            self.max_offset_x()
        }
    }
    fn max_offset_y_for_percent(&self) -> f32 {
        if self.base.infinite() {
            self.content_height()
        } else {
            self.max_offset_y()
        }
    }
    pub fn velocity_x(&self) -> f32 {
        self.physics
            .as_ref()
            .map_or(0.0, |physics| physics.speed().x)
    }
    pub fn velocity_y(&self) -> f32 {
        self.physics
            .as_ref()
            .map_or(0.0, |physics| physics.speed().y)
    }
    pub fn set_velocity_x(&mut self, _value: f32) {}
    pub fn set_velocity_y(&mut self, _value: f32) {}
    pub fn scroll_active(&self) -> bool {
        self.is_dragging
            || self.is_scroll_bar_dragging
            || self
                .physics
                .as_ref()
                .is_some_and(|physics| physics.is_running())
    }
    pub fn set_scroll_active(&mut self, _value: bool) {}

    pub fn scroll_percent_x(&self) -> f32 {
        if self.intent_x.space == ScrollSpace::Percent {
            return self.intent_x.value;
        }
        if self.max_offset_x() != 0.0 {
            self.authored_scroll_offset_x() / self.max_offset_x_for_percent()
        } else {
            0.0
        }
    }
    pub fn scroll_percent_y(&self) -> f32 {
        if self.intent_y.space == ScrollSpace::Percent {
            return self.intent_y.value;
        }
        if self.max_offset_y() != 0.0 {
            self.authored_scroll_offset_y() / self.max_offset_y_for_percent()
        } else {
            0.0
        }
    }
    pub fn scroll_index(&self) -> f32 {
        let intent = if self.base.constrains_horizontal() {
            self.intent_x
        } else {
            self.intent_y
        };
        if intent.space == ScrollSpace::Index {
            intent.value
        } else {
            self.index_at_position(Vec2D::new(
                self.authored_scroll_offset_x(),
                self.authored_scroll_offset_y(),
            ))
        }
    }
    pub fn set_scroll_percent_x(&mut self, value: f32) {
        if self.is_dragging {
            return;
        }
        self.stop_physics();
        self.set_intent_x(ScrollAxisIntent {
            space: ScrollSpace::Percent,
            value,
        });
    }
    pub fn set_scroll_percent_y(&mut self, value: f32) {
        if self.is_dragging {
            return;
        }
        self.stop_physics();
        self.set_intent_y(ScrollAxisIntent {
            space: ScrollSpace::Percent,
            value,
        });
    }
    pub fn set_scroll_index(&mut self, value: f32) {
        if self.is_dragging {
            return;
        }
        self.stop_physics();
        if self.base.constrains_horizontal() {
            self.set_intent_x(ScrollAxisIntent {
                space: ScrollSpace::Index,
                value,
            });
        }
        if self.base.constrains_vertical() {
            self.set_intent_y(ScrollAxisIntent {
                space: ScrollSpace::Index,
                value,
            });
        }
    }

    fn scroll_layout_resolvable(&self, is_x: bool) -> bool {
        if is_x {
            self.viewport().layout_width() > 0.0
        } else {
            self.viewport().layout_height() > 0.0
        }
    }
    fn clamp_resolved_offset(&self, value: f32, is_x: bool) -> f32 {
        if self.base.infinite() {
            value
        } else {
            math_types::clamp(
                value,
                if is_x {
                    self.max_offset_x()
                } else {
                    self.max_offset_y()
                },
                0.0,
            )
        }
    }

    fn resolve_intent(&self, intent: ScrollAxisIntent, is_x: bool) -> Option<f32> {
        if intent.space == ScrollSpace::Index
            && (intent.value.is_nan() || (self.base.infinite() && !intent.value.is_finite()))
        {
            return Some(0.0);
        }
        if !self.scroll_layout_resolvable(is_x) {
            return None;
        }
        match intent.space {
            ScrollSpace::Percent => {
                let content_size = if is_x {
                    self.content_width()
                } else {
                    self.content_height()
                };
                if content_size <= 0.0 {
                    return None;
                }
                let maximum = if is_x {
                    self.max_offset_x_for_percent()
                } else {
                    self.max_offset_y_for_percent()
                };
                Some(self.clamp_resolved_offset(intent.value * maximum, is_x))
            }
            ScrollSpace::Index => self.position_at_index(intent.value).map(|position| {
                self.clamp_resolved_offset(if is_x { position.x } else { position.y }, is_x)
            }),
            ScrollSpace::None => None,
        }
    }
    fn set_intent_x(&mut self, intent: ScrollAxisIntent) {
        if let Some(offset) = self.resolve_intent(intent, true) {
            self.intent_x.space = ScrollSpace::None;
            self.set_authored_scroll_offset_x(offset);
        } else {
            self.intent_x = intent;
        }
    }
    fn set_intent_y(&mut self, intent: ScrollAxisIntent) {
        if let Some(offset) = self.resolve_intent(intent, false) {
            self.intent_y.space = ScrollSpace::None;
            self.set_authored_scroll_offset_y(offset);
        } else {
            self.intent_y = intent;
        }
    }
    fn resolve_scroll_intents(&mut self) {
        if self.intent_x.space != ScrollSpace::None {
            self.set_intent_x(self.intent_x);
        }
        if self.intent_y.space != ScrollSpace::None {
            self.set_intent_y(self.intent_y);
        }
    }
    fn clear_scroll_intents(&mut self) {
        self.intent_x.space = ScrollSpace::None;
        self.intent_y.space = ScrollSpace::None;
    }

    fn position_at_index(&self, index: f32) -> Option<Vec2D> {
        if index.is_nan() || (self.base.infinite() && !index.is_finite()) {
            return Some(Vec2D::default());
        }
        let count = self.scroll_item_count();
        if count == 0 {
            return None;
        }
        let content_gap = self.gap();
        let normalized = if self.base.infinite() {
            let mut value = index % count as f32;
            if value < 0.0 {
                value += count as f32;
            }
            value
        } else {
            let value = index.max(0.0);
            if value >= count as f32 {
                if self.content_width() <= 0.0 && self.content_height() <= 0.0 {
                    return None;
                }
                return Some(Vec2D::new(-self.content_width(), -self.content_height()));
            }
            value
        };
        let floor_index = normalized.floor();
        let fraction = normalized - floor_index;
        let target_index = floor_index as usize;
        if !self.has_list_children {
            let bounds = self.bounds_for_flat_index(target_index);
            if !self.is_bounds_collapsed(bounds) {
                return Some(Vec2D::new(
                    -bounds.left() - (bounds.width() + content_gap.x) * fraction,
                    -bounds.top() - (bounds.height() + content_gap.y) * fraction,
                ));
            }
            for index in target_index + 1..count {
                let bounds = self.bounds_for_flat_index(index);
                if !self.is_bounds_collapsed(bounds) {
                    return Some(Vec2D::new(-bounds.left(), -bounds.top()));
                }
            }
            if self.base.infinite() {
                for index in 0..target_index {
                    let bounds = self.bounds_for_flat_index(index);
                    if !self.is_bounds_collapsed(bounds) {
                        return Some(Vec2D::new(-bounds.left(), -bounds.top()));
                    }
                }
            } else {
                for index in (0..target_index).rev() {
                    let bounds = self.bounds_for_flat_index(index);
                    if !self.is_bounds_collapsed(bounds) {
                        return Some(Vec2D::new(-bounds.left(), -bounds.top()));
                    }
                }
            }
            return None;
        }

        let mut flat_index = 0usize;
        let mut last_visible = Vec2D::default();
        let mut has_visible = false;
        let mut reached_target = false;
        for child in self.layout_children.iter().copied() {
            if child.is_null() {
                continue;
            }
            for local in 0..unsafe { (*child).num_layout_nodes() } {
                let bounds = unsafe { (*child).layout_bounds_for_node(local) };
                if flat_index < target_index {
                    if !self.is_bounds_collapsed(bounds) {
                        last_visible = Vec2D::new(-bounds.left(), -bounds.top());
                        has_visible = true;
                    }
                    flat_index += 1;
                    continue;
                }
                if flat_index == target_index {
                    reached_target = true;
                    if !self.is_bounds_collapsed(bounds) {
                        return Some(Vec2D::new(
                            -bounds.left() - (bounds.width() + content_gap.x) * fraction,
                            -bounds.top() - (bounds.height() + content_gap.y) * fraction,
                        ));
                    }
                    flat_index += 1;
                    continue;
                }
                if !self.is_bounds_collapsed(bounds) {
                    return Some(Vec2D::new(-bounds.left(), -bounds.top()));
                }
                flat_index += 1;
            }
        }
        if !reached_target {
            return None;
        }
        if self.base.infinite() {
            flat_index = 0;
            for child in self.layout_children.iter().copied() {
                if child.is_null() {
                    continue;
                }
                for local in 0..unsafe { (*child).num_layout_nodes() } {
                    if flat_index >= target_index {
                        return None;
                    }
                    let bounds = unsafe { (*child).layout_bounds_for_node(local) };
                    if !self.is_bounds_collapsed(bounds) {
                        return Some(Vec2D::new(-bounds.left(), -bounds.top()));
                    }
                    flat_index += 1;
                }
            }
        } else if has_visible {
            return Some(last_visible);
        }
        None
    }

    fn index_at_position(&self, position: Vec2D) -> f32 {
        if self.content().children().is_empty() {
            return 0.0;
        }
        let gap = self.gap();
        if !self.has_list_children {
            let count = self.layout_children.len();
            if self.base.constrains_horizontal() {
                for index in 0..count {
                    let bounds =
                        unsafe { (*self.layout_children[index]).layout_bounds_for_node(0) };
                    let step = bounds.width() + gap.x;
                    if position.x > -bounds.left() - step {
                        return if step != 0.0 {
                            index as f32 + (-position.x - bounds.left()) / step
                        } else {
                            index as f32
                        };
                    }
                }
                return count as f32;
            } else if self.base.constrains_vertical() {
                for index in 0..count {
                    let bounds =
                        unsafe { (*self.layout_children[index]).layout_bounds_for_node(0) };
                    let step = bounds.height() + gap.y;
                    if position.y > -bounds.top() - step {
                        return if step != 0.0 {
                            index as f32 + (-position.y - bounds.top()) / step
                        } else {
                            index as f32
                        };
                    }
                }
                return count as f32;
            }
            return 0.0;
        }
        let mut flat_index = 0.0;
        if self.base.constrains_horizontal() {
            for child in self.layout_children.iter().copied() {
                if child.is_null() {
                    continue;
                }
                let count = unsafe { (*child).num_layout_nodes() };
                for local in 0..count {
                    let bounds = unsafe { (*child).layout_bounds_for_node(local) };
                    let step = bounds.width() + gap.x;
                    if position.x > -bounds.left() - step {
                        return if step != 0.0 {
                            flat_index + local as f32 + (-position.x - bounds.left()) / step
                        } else {
                            flat_index + local as f32
                        };
                    }
                }
                flat_index += count as f32;
            }
            return flat_index;
        } else if self.base.constrains_vertical() {
            for child in self.layout_children.iter().copied() {
                if child.is_null() {
                    continue;
                }
                let count = unsafe { (*child).num_layout_nodes() };
                for local in 0..count {
                    let bounds = unsafe { (*child).layout_bounds_for_node(local) };
                    let step = bounds.height() + gap.y;
                    if position.y > -bounds.top() - step {
                        return if step != 0.0 {
                            flat_index + local as f32 + (-position.y - bounds.top()) / step
                        } else {
                            flat_index + local as f32
                        };
                    }
                }
                flat_index += count as f32;
            }
            return flat_index;
        }
        0.0
    }

    fn is_bounds_collapsed(&self, bounds: Aabb) -> bool {
        (self.base.constrains_horizontal() && bounds.width() <= 0.0)
            || (self.base.constrains_vertical() && bounds.height() <= 0.0)
    }
    pub fn scroll_item_count(&self) -> usize {
        if !self.has_list_children {
            self.layout_children.len()
        } else {
            self.layout_children
                .iter()
                .copied()
                .filter(|child| !child.is_null())
                .map(|child| unsafe { (*child).num_layout_nodes() })
                .sum()
        }
    }
    fn bounds_for_flat_index(&self, index: usize) -> Aabb {
        if !self.has_list_children {
            if index < self.layout_children.len() && !self.layout_children[index].is_null() {
                return unsafe { (*self.layout_children[index]).layout_bounds_for_node(0) };
            }
            return Aabb::default();
        }
        let mut flat_index = 0;
        for child in self.layout_children.iter().copied() {
            if child.is_null() {
                continue;
            }
            let count = unsafe { (*child).num_layout_nodes() };
            if index < flat_index + count {
                return unsafe { (*child).layout_bounds_for_node(index - flat_index) };
            }
            flat_index += count;
        }
        Aabb::default()
    }

    pub fn gap(&self) -> Vec2D {
        Vec2D::new(
            self.content().gap_horizontal(),
            self.content().gap_vertical(),
        )
    }

    pub fn scroll_to_position(&mut self, target_x: f32, target_y: f32) {
        self.clear_scroll_intents();
        if self.physics.is_none() {
            self.set_authored_scroll_offset_x(target_x);
            self.set_authored_scroll_offset_y(target_y);
            return;
        }
        let current = Vec2D::new(self.offset_x, self.offset_y);
        let target = Vec2D::new(target_x, target_y);
        let range_min = Vec2D::new(self.max_offset_x(), self.max_offset_y());
        let range_max = Vec2D::default();
        let horizontal = self.base.constrains_horizontal();
        let vertical = self.base.constrains_vertical();
        self.physics
            .as_mut()
            .unwrap()
            .scroll_to_position(current, target, range_min, range_max, horizontal, vertical);
    }

    fn nearest_snap_in_direction(current: f32, target: f32, points: &[Vec2D], use_x: bool) -> f32 {
        if current == target {
            return target;
        }
        let negative = target < current;
        let mut best = target;
        let mut found = false;
        let mut best_distance = 0.0;
        for point in points {
            let candidate = if use_x { -point.x } else { -point.y };
            if if negative {
                candidate > target
            } else {
                candidate < target
            } {
                continue;
            }
            let distance = if negative {
                target - candidate
            } else {
                candidate - target
            };
            if !found || distance < best_distance {
                best_distance = distance;
                best = candidate;
                found = true;
            }
        }
        if found { best } else { target }
    }

    pub fn nearest_snap_offset_in_direction(&self, current: Vec2D, target: Vec2D) -> Vec2D {
        if !self.base.snap() {
            return target;
        }
        let points = self.collect_snap_points();
        if points.is_empty() {
            return target;
        }
        Vec2D::new(
            if self.base.constrains_horizontal() {
                Self::nearest_snap_in_direction(current.x, target.x, &points, true)
            } else {
                target.x
            },
            if self.base.constrains_vertical() {
                Self::nearest_snap_in_direction(current.y, target.y, &points, false)
            } else {
                target.y
            },
        )
    }
    pub fn effective_scroll_offset_x(&self) -> f32 {
        if let Some(physics) = &self.physics {
            if physics.is_running() && physics.has_target_x() {
                return physics.target_x();
            }
        }
        self.authored_scroll_offset_x()
    }
    pub fn effective_scroll_offset_y(&self) -> f32 {
        if let Some(physics) = &self.physics {
            if physics.is_running() && physics.has_target_y() {
                return physics.target_y();
            }
        }
        self.authored_scroll_offset_y()
    }

    pub fn physics_mut(&mut self) -> Option<&mut dyn ScrollPhysics> {
        self.physics.as_deref_mut()
    }
    pub fn set_physics(&mut self, physics: Box<dyn ScrollPhysics>) {
        self.physics = Some(physics);
    }
    pub fn physics_type(&self) -> ScrollPhysicsType {
        ScrollPhysicsType::from(self.base.physics_type_value())
    }
    pub fn has_layout_parent(&self) -> bool {
        self.base.parent().is::<LayoutComponent>()
    }
    pub fn content(&self) -> &LayoutComponent {
        self.base.parent().as_ref::<LayoutComponent>().unwrap()
    }
    pub fn content_mut(&mut self) -> &mut LayoutComponent {
        self.base.parent_mut().as_mut::<LayoutComponent>().unwrap()
    }
    pub fn viewport(&self) -> &LayoutComponent {
        self.base
            .parent()
            .parent()
            .unwrap()
            .as_ref::<LayoutComponent>()
            .unwrap()
    }
    pub fn viewport_mut(&mut self) -> &mut LayoutComponent {
        self.base
            .parent_mut()
            .parent_mut()
            .unwrap()
            .as_mut::<LayoutComponent>()
            .unwrap()
    }
    pub fn computed_content_width(&self) -> f32 {
        if self.has_layout_parent() {
            self.content_width()
        } else {
            0.0
        }
    }
    pub fn computed_content_height(&self) -> f32 {
        if self.has_layout_parent() {
            self.content_height()
        } else {
            0.0
        }
    }
    pub fn set_computed_content_width(&mut self, _value: f32) {}
    pub fn set_computed_content_height(&mut self, _value: f32) {}
    pub fn authored_scroll_offset_x(&self) -> f32 {
        self.base.scroll_offset_x()
    }
    pub fn authored_scroll_offset_y(&self) -> f32 {
        self.base.scroll_offset_y()
    }
    pub fn set_authored_scroll_offset_x(&mut self, value: f32) {
        self.base.set_scroll_offset_x(value);
        self.set_offset_x(value);
    }
    pub fn set_authored_scroll_offset_y(&mut self, value: f32) {
        self.base.set_scroll_offset_y(value);
        self.set_offset_y(value);
    }
    pub fn scroll_offset_x_changed(&mut self) {
        self.set_offset_x(self.base.scroll_offset_x());
    }
    pub fn scroll_offset_y_changed(&mut self) {
        self.set_offset_y(self.base.scroll_offset_y());
    }
    pub fn direction(&self) -> DraggableConstraintDirection {
        DraggableConstraintDirection::from(self.base.direction_value())
    }
    pub fn infinite(&self) -> bool {
        self.base.infinite()
    }
    pub fn interactive(&self) -> bool {
        self.base.interactive()
    }
    pub fn threshold(&self) -> f32 {
        self.base.threshold()
    }
    pub fn set_is_scroll_bar_dragging(&mut self, value: bool) {
        if !self.is_scroll_bar_dragging && value {
            self.clear_scroll_intents();
            self.last_frame_offset_x = self.authored_scroll_offset_x();
            self.last_frame_offset_y = self.authored_scroll_offset_y();
        }
        self.is_scroll_bar_dragging = value;
    }
}
