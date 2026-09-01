use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    artboard_component_list::ArtboardComponentList,
    component_dirt::ComponentDirt,
    constraints::{
        constraint::Constraint,
        draggable_constraint::{DraggableConstraintDirection, DraggableProxy},
        layout_constraint::LayoutConstraint,
        scrolling::{
            scroll_constraint_proxy::ViewportDraggableProxy,
            scroll_physics::{self, ScrollPhysicsRuntime, ScrollPhysicsType},
            scroll_virtualizer::ScrollVirtualizer,
        },
        transform_constraint::TransformConstraint,
    },
    core::{Core, CoreHandle},
    core_context::{CoreContext, StatusCode},
    generated::{
        constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
        core_registry::CoreCapabilities,
    },
    importers::import_stack::ImportStack,
    layout::layout_node_provider::{self, LayoutNodeProvider},
    layout_component::LayoutComponent,
    math::{
        aabb::Aabb, mat2d::Mat2D, math_types, transform_components::TransformComponents,
        vec2d::Vec2D,
    },
    virtualizing_component::VirtualizedDirection,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScrollSpace {
    None,
    Percent,
    Index,
}

impl LayoutConstraint for ScrollConstraint {
    fn constraint_handle(&self) -> CoreHandle {
        self.handle().expect("arena-owned ScrollConstraint")
    }

    fn layout_child_constrainer(&self) -> fn(&CoreHandle, CoreHandle) -> bool {
        Self::constrain_child_occurrence
    }

    fn add_layout_child(&mut self, child: CoreHandle) {
        ScrollConstraint::add_layout_child(self, child);
    }
}

impl crate::mechanical_port::source::advancing_component::AdvancingComponent for ScrollConstraint {
    fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        ScrollConstraint::advance_component(self, elapsed_seconds, flags)
    }
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
    physics: Option<CoreHandle>,
    virtualizer: Option<Rc<RefCell<ScrollVirtualizer>>>,
    layout_children: Vec<CoreHandle>,
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

impl Deref for ScrollConstraint {
    type Target = ScrollConstraintBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for ScrollConstraint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Default for ScrollConstraint {
    fn default() -> Self {
        Self {
            base: ScrollConstraintBase::default(),
            physics: None,
            virtualizer: None,
            layout_children: Vec::new(),
            components_a: TransformComponents::default(),
            components_b: TransformComponents::default(),
            scroll_transform: Mat2D::default(),
            offset_x: 0.0,
            offset_y: 0.0,
            last_frame_offset_x: 0.0,
            last_frame_offset_y: 0.0,
            child_constraint_applied_count: 0,
            is_dragging: false,
            is_scroll_bar_dragging: false,
            has_list_children: false,
            intent_x: ScrollAxisIntent::default(),
            intent_y: ScrollAxisIntent::default(),
        }
    }
}

impl Drop for ScrollConstraint {
    fn drop(&mut self) {
        self.virtualizer = None;
        self.layout_children.clear();
        if let Some(physics) = self.physics.take() {
            physics.remove_occurrence();
        }
    }
}

impl ScrollConstraint {
    pub fn clone_definition(&self) -> Self {
        let mut twin = Self::default();
        let mut base = std::mem::take(&mut twin.base);
        base.copy(&self.base, &mut twin);
        twin.base = base;
        twin.physics = self.physics.as_ref().map(|physics| {
            physics
                .clone_occurrence()
                .expect("live ScrollPhysics is cloneable")
        });
        twin
    }
    pub fn handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.base.base.handle()
    }

    pub fn physics(&self) -> Option<CoreHandle> {
        self.physics.clone()
    }

    pub fn content_handle(&self) -> Option<CoreHandle> {
        self.base.parent_handle()
    }

    pub fn viewport_handle(&self) -> Option<CoreHandle> {
        self.content_handle()?
            .with(|content| content.component_parent_handle())?
    }

    fn with_content<R>(&self, use_content: impl FnOnce(&LayoutComponent) -> R) -> Option<R> {
        self.content_handle()?
            .with(|content| content.as_layout_component().map(use_content))?
    }

    fn with_content_mut<R>(
        &self,
        use_content: impl FnOnce(&mut LayoutComponent) -> R,
    ) -> Option<R> {
        self.content_handle()?
            .with_mut(|content| content.as_layout_component_mut().map(use_content))?
    }

    fn with_viewport<R>(&self, use_viewport: impl FnOnce(&LayoutComponent) -> R) -> Option<R> {
        self.viewport_handle()?
            .with(|viewport| viewport.as_layout_component().map(use_viewport))?
    }

    fn with_viewport_mut<R>(
        &self,
        use_viewport: impl FnOnce(&mut LayoutComponent) -> R,
    ) -> Option<R> {
        self.viewport_handle()?
            .with_mut(|viewport| viewport.as_layout_component_mut().map(use_viewport))?
    }

    fn with_physics<R>(
        &self,
        use_physics: impl FnOnce(&dyn ScrollPhysicsRuntime) -> R,
    ) -> Option<R> {
        self.physics
            .as_ref()?
            .with(|object| scroll_physics::from_core(object).map(use_physics))?
    }

    fn with_physics_mut<R>(
        &self,
        use_physics: impl FnOnce(&mut dyn ScrollPhysicsRuntime) -> R,
    ) -> Option<R> {
        self.physics
            .as_ref()?
            .with_mut(|object| scroll_physics::from_core_mut(object).map(use_physics))?
    }

    fn with_layout_child<R>(
        child: &CoreHandle,
        use_child: impl FnOnce(&dyn LayoutNodeProvider) -> R,
    ) -> Option<R> {
        child.with(|child| child.as_layout_node_provider().map(use_child))?
    }

    /// A list asks its registered ScrollConstraint whether it virtualizes.
    /// Index/snap queries already borrow that exact constraint, so carry it
    /// through the virtual bounds query rather than rereading its Core slot.
    fn layout_child_bounds_for_node(&self, child: &CoreHandle, index: usize) -> Aabb {
        if child.is_type_of(ArtboardComponentList::TYPE_KEY) {
            child
                .with_downcast::<ArtboardComponentList, _>(|list| {
                    list.layout_bounds_for_node_with_scroll(index, Some(self))
                })
                .expect("live ArtboardComponentList")
        } else {
            Self::with_layout_child(child, |child| child.layout_bounds_for_node(index))
                .expect("ScrollConstraint layout child remains a LayoutNodeProvider")
        }
    }

    fn with_layout_child_mut<R>(
        child: &CoreHandle,
        use_child: impl FnOnce(&mut dyn LayoutNodeProvider) -> R,
    ) -> Option<R> {
        child.with_mut(|child| child.as_layout_node_provider_mut().map(use_child))?
    }

    pub fn content_width(&self) -> f32 {
        if self.base.virtualize() && !self.main_axis_is_column() {
            let mut content_size = 0.0;
            for child in &self.layout_children {
                content_size +=
                    Self::with_layout_child(child, |child| child.layout_bounds().width())
                        .expect("ScrollConstraint layout child remains a LayoutNodeProvider");
            }
            let len_offset = if self.base.infinite() { 0 } else { 1 };
            content_size +=
                self.gap().x * self.layout_children.len().wrapping_sub(len_offset) as f32;
            if !self.base.infinite() {
                content_size += self
                    .with_content(|content| content.padding_left() + content.padding_right())
                    .expect("ScrollConstraint content remains LayoutComponent");
            }
            return content_size;
        }
        self.with_content(LayoutComponent::layout_width)
            .expect("ScrollConstraint content remains LayoutComponent")
    }

    pub fn content_height(&self) -> f32 {
        if self.base.virtualize() && self.main_axis_is_column() {
            let mut content_size = 0.0;
            for child in &self.layout_children {
                content_size +=
                    Self::with_layout_child(child, |child| child.layout_bounds().height())
                        .expect("ScrollConstraint layout child remains a LayoutNodeProvider");
            }
            let len_offset = if self.base.infinite() { 0 } else { 1 };
            content_size +=
                self.gap().y * self.layout_children.len().wrapping_sub(len_offset) as f32;
            if !self.base.infinite() {
                content_size += self
                    .with_content(|content| content.padding_top() + content.padding_bottom())
                    .expect("ScrollConstraint content remains LayoutComponent");
            }
            return content_size;
        }
        self.with_content(LayoutComponent::layout_height)
            .expect("ScrollConstraint content remains LayoutComponent")
    }

    pub fn viewport_width(&self) -> f32 {
        if self.direction() == DraggableConstraintDirection::Vertical {
            self.with_viewport(LayoutComponent::layout_width)
                .expect("ScrollConstraint viewport remains LayoutComponent")
        } else {
            let viewport_width = self
                .with_viewport(LayoutComponent::layout_width)
                .expect("ScrollConstraint viewport remains LayoutComponent");
            let content_x = self
                .with_content(LayoutComponent::layout_x)
                .expect("ScrollConstraint content remains LayoutComponent");
            0.0_f32.max(viewport_width - content_x)
        }
    }
    pub fn viewport_height(&self) -> f32 {
        if self.direction() == DraggableConstraintDirection::Horizontal {
            self.with_viewport(LayoutComponent::layout_height)
                .expect("ScrollConstraint viewport remains LayoutComponent")
        } else {
            let viewport_height = self
                .with_viewport(LayoutComponent::layout_height)
                .expect("ScrollConstraint viewport remains LayoutComponent");
            let content_y = self
                .with_content(LayoutComponent::layout_y)
                .expect("ScrollConstraint content remains LayoutComponent");
            0.0_f32.max(viewport_height - content_y)
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
            0.0_f32.min(
                self.viewport_width()
                    - self.content_width()
                    - self
                        .with_viewport(LayoutComponent::padding_right)
                        .expect("ScrollConstraint viewport remains LayoutComponent"),
            )
        }
    }
    pub fn max_offset_y(&self) -> f32 {
        if self.base.infinite() && self.main_axis_is_column() {
            f32::NEG_INFINITY
        } else {
            0.0_f32.min(
                self.viewport_height()
                    - self.content_height()
                    - self
                        .with_viewport(LayoutComponent::padding_bottom)
                        .expect("ScrollConstraint viewport remains LayoutComponent"),
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
        if let Some(value) = self.with_physics(|physics| {
            physics.enabled().then(|| {
                physics
                    .clamp(
                        Vec2D::new(self.max_offset_x(), self.max_offset_y()),
                        Vec2D::new(self.min_offset_x(), self.min_offset_y()),
                        Vec2D::new(self.offset_x, self.offset_y),
                    )
                    .x
            })
        }) {
            if let Some(value) = value {
                return value;
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
        if let Some(value) = self.with_physics(|physics| {
            physics.enabled().then(|| {
                physics
                    .clamp(
                        Vec2D::new(self.max_offset_x(), self.max_offset_y()),
                        Vec2D::new(self.min_offset_x(), self.min_offset_y()),
                        Vec2D::new(self.offset_x, self.offset_y),
                    )
                    .y
            })
        }) {
            if let Some(value) = value {
                return value;
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
        self.mark_content_transform_dirty();
    }
    pub fn set_offset_y(&mut self, value: f32) {
        if self.offset_y == value {
            return;
        }
        self.offset_y = value;
        self.mark_content_transform_dirty();
    }
    fn mark_content_transform_dirty(&mut self) {
        let content = self
            .content_handle()
            .expect("ScrollConstraint content remains LayoutComponent");
        crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(content)
            .add_dirt_from_scroll(self, ComponentDirt::WORLD_TRANSFORM, true);
    }
    pub fn main_axis_is_column(&self) -> bool {
        self.with_content(LayoutComponent::main_axis_is_column)
            .expect("ScrollConstraint content remains LayoutComponent")
    }

    // The source argument is unused; content/viewport access follows the
    // retained parent pointers without borrowing the supplied component.
    pub fn constrain(&mut self, _component: &CoreHandle) {
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

    pub fn constrain_child_occurrence(scroll: &CoreHandle, provider: CoreHandle) -> bool {
        let Some(owner) = provider
            .with(|provider| {
                provider
                    .as_layout_node_provider()
                    .and_then(LayoutNodeProvider::owner_handle)
            })
            .flatten()
        else {
            return false;
        };
        let (scroll_transform, components_a, components_b, strength) = scroll
            .with_downcast::<Self, _>(|scroll| {
                (
                    scroll.scroll_transform,
                    scroll.components_a,
                    scroll.components_b,
                    scroll.base.strength(),
                )
            })
            .expect("live ScrollConstraint");
        let applied = owner
            .with_mut(|owner| {
                let component = owner.as_transform_component_mut()?;
                let current = *component.world_transform();
                let target = Constraint::offset_in_parent_frame(component, &scroll_transform);
                TransformConstraint::constrain_world(
                    component,
                    current,
                    components_a,
                    target,
                    components_b,
                    strength,
                );
                Some(())
            })
            .flatten()
            .is_some();
        if !applied {
            return false;
        }
        scroll
            .with_downcast_mut::<Self, _>(|scroll| scroll.child_constraint_applied_count += 1)
            .expect("live ScrollConstraint");
        Self::constrain_virtualized_occurrence(scroll, false);
        true
    }

    pub fn constrain_virtualized_occurrence(owner: &CoreHandle, force: bool) {
        let Some((virtualizer, children, offset, direction)) = owner
            .with_downcast::<Self, _>(|scroll| {
                if !scroll.base.virtualize() {
                    return None;
                }
                let virtualizer = scroll.virtualizer.clone()?;
                let children = scroll.layout_children.clone();
                if scroll.child_constraint_applied_count < children.len() as i32 && !force {
                    return None;
                }
                let column = scroll.main_axis_is_column();
                let direction = if column {
                    VirtualizedDirection::Vertical
                } else {
                    VirtualizedDirection::Horizontal
                };
                let offset = if column {
                    scroll.clamped_offset_y()
                } else {
                    scroll.clamped_offset_x()
                };
                Some((virtualizer, children, offset, direction))
            })
            .expect("live ScrollConstraint")
        else {
            return;
        };
        virtualizer
            .borrow_mut()
            .constrain(owner, &children, offset, direction);
    }
    pub fn add_layout_child(&mut self, child: CoreHandle) {
        assert!(!self.layout_children.contains(&child));
        self.layout_children.push(child);
    }

    pub fn drag_view(&mut self, delta: Vec2D, time_stamp: f32) {
        let scaled = Vec2D::new(
            delta.x * self.base.drag_multiplier(),
            delta.y * self.base.drag_multiplier(),
        );
        if self.physics.is_some() {
            self.with_physics_mut(|physics| physics.accumulate(scaled, time_stamp))
                .expect("ScrollConstraint physics remains ScrollPhysics-derived");
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
        for child in &self.layout_children {
            let node_count = Self::with_layout_child(child, |child| child.num_layout_nodes())
                .expect("ScrollConstraint layout child remains a LayoutNodeProvider");
            for node in 0..node_count {
                let bounds = self.layout_child_bounds_for_node(child, node);
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
        self.with_physics_mut(|physics| {
            physics.run(args.0, args.1, args.2, points, args.3, args.4)
        });
    }

    pub fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        if !flags.contains(AdvanceFlags::ADVANCE_NESTED) || self.base.is_collapsed() {
            return false;
        }
        if self.physics.is_none() {
            return false;
        }
        let offset = self.with_physics_mut(|physics| {
            physics
                .is_running()
                .then(|| physics.advance(elapsed_seconds))
        });
        if let Some(Some(offset)) = offset {
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
        self.with_physics(|physics| physics.enabled())
            .expect("ScrollConstraint physics remains ScrollPhysics-derived")
            || self.is_scroll_bar_dragging
            || self.is_dragging
    }

    pub fn draggables(&mut self) -> Vec<Box<dyn DraggableProxy>> {
        let constraint = self.handle().expect("arena-owned ScrollConstraint");
        let viewport = self
            .viewport_handle()
            .expect("ScrollConstraint viewport was validated");
        let viewport = viewport
            .with_mut(|viewport| {
                viewport
                    .as_layout_component_mut()
                    .and_then(LayoutComponent::proxy)
            })
            .flatten()
            .expect("ScrollConstraint viewport retains its drawable proxy");
        vec![Box::new(ViewportDraggableProxy::new(constraint, viewport))]
    }

    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        self.has_list_children = false;
        let children = self
            .with_content(|content| content.children().to_vec())
            .expect("ScrollConstraint content remains LayoutComponent");
        for child in children {
            let layout = layout_node_provider::from_component(&child);
            if let Some(layout) = layout {
                self.base.add_dependent(child.clone());
                layout
                    .with_mut(|child| {
                        child
                            .as_layout_node_provider_mut()
                            .expect("layout capability remains stable")
                            .add_layout_constraint(self);
                    })
                    .expect("ScrollConstraint content retains live children");
            }
            if child.is_type_of(ArtboardComponentList::TYPE_KEY) {
                self.has_list_children = true;
            }
        }
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest_backboard_importer() else {
            return StatusCode::MissingObject;
        };
        let objects = importer.physics();
        let id = self.base.physics_id() as usize;
        self.physics = objects.get(id).map(|physics| {
            physics
                .clone_occurrence()
                .expect("imported ScrollPhysics is cloneable")
        });
        self.base.import(import_stack)
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let result = self.base.on_added_dirty(context);
        if self.base.virtualize() {
            self.virtualizer = Some(Rc::new(RefCell::new(ScrollVirtualizer::default())));
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
        self.with_physics_mut(|physics| physics.prepare(direction));
    }
    pub fn stop_physics(&mut self) {
        self.with_physics_mut(|physics| physics.reset());
    }
    pub fn clear_velocity(&mut self) {
        self.with_physics_mut(|physics| physics.clear_velocity());
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
        self.with_physics(|physics| physics.speed().x)
            .unwrap_or(0.0)
    }
    pub fn velocity_y(&self) -> f32 {
        self.with_physics(|physics| physics.speed().y)
            .unwrap_or(0.0)
    }
    pub fn set_velocity_x(&mut self, _value: f32) {}
    pub fn set_velocity_y(&mut self, _value: f32) {}
    pub fn scroll_active(&self) -> bool {
        self.is_dragging
            || self.is_scroll_bar_dragging
            || self
                .with_physics(|physics| physics.is_running())
                .unwrap_or(false)
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
            self.with_viewport(LayoutComponent::layout_width)
                .is_some_and(|width| width > 0.0)
        } else {
            self.with_viewport(LayoutComponent::layout_height)
                .is_some_and(|height| height > 0.0)
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
        for child in &self.layout_children {
            let count = Self::with_layout_child(child, |child| child.num_layout_nodes())
                .expect("ScrollConstraint layout child remains a LayoutNodeProvider");
            for local in 0..count {
                let bounds = self.layout_child_bounds_for_node(child, local);
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
            for child in &self.layout_children {
                let count = Self::with_layout_child(child, |child| child.num_layout_nodes())
                    .expect("ScrollConstraint layout child remains a LayoutNodeProvider");
                for local in 0..count {
                    if flat_index >= target_index {
                        return None;
                    }
                    let bounds = self.layout_child_bounds_for_node(child, local);
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
        if self
            .with_content(|content| content.children().is_empty())
            .unwrap_or(true)
        {
            return 0.0;
        }
        let gap = self.gap();
        if !self.has_list_children {
            let count = self.layout_children.len();
            if self.base.constrains_horizontal() {
                for index in 0..count {
                    let bounds = self.layout_child_bounds_for_node(&self.layout_children[index], 0);
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
                    let bounds = self.layout_child_bounds_for_node(&self.layout_children[index], 0);
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
            for child in &self.layout_children {
                let count = Self::with_layout_child(child, |child| child.num_layout_nodes())
                    .expect("ScrollConstraint layout child remains a LayoutNodeProvider");
                for local in 0..count {
                    let bounds = self.layout_child_bounds_for_node(child, local);
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
            for child in &self.layout_children {
                let count = Self::with_layout_child(child, |child| child.num_layout_nodes())
                    .expect("ScrollConstraint layout child remains a LayoutNodeProvider");
                for local in 0..count {
                    let bounds = self.layout_child_bounds_for_node(child, local);
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
                .map(|child| {
                    Self::with_layout_child(child, |child| child.num_layout_nodes())
                        .expect("ScrollConstraint layout child remains a LayoutNodeProvider")
                })
                .sum()
        }
    }
    fn bounds_for_flat_index(&self, index: usize) -> Aabb {
        if !self.has_list_children {
            if index < self.layout_children.len() {
                return self.layout_child_bounds_for_node(&self.layout_children[index], 0);
            }
            return Aabb::default();
        }
        let mut flat_index = 0;
        for child in &self.layout_children {
            let count = Self::with_layout_child(child, |child| child.num_layout_nodes())
                .expect("ScrollConstraint layout child remains a LayoutNodeProvider");
            if index < flat_index + count {
                return self.layout_child_bounds_for_node(child, index - flat_index);
            }
            flat_index += count;
        }
        Aabb::default()
    }

    pub fn gap(&self) -> Vec2D {
        self.with_content(|content| Vec2D::new(content.gap_horizontal(), content.gap_vertical()))
            .expect("ScrollConstraint content remains LayoutComponent")
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
        self.with_physics_mut(|physics| {
            physics.scroll_to_position(current, target, range_min, range_max, horizontal, vertical)
        })
        .expect("ScrollConstraint physics remains ScrollPhysics-derived");
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
        if let Some(Some(target)) = self.with_physics(|physics| {
            (physics.is_running() && physics.has_target_x()).then(|| physics.target_x())
        }) {
            return target;
        }
        self.authored_scroll_offset_x()
    }
    pub fn effective_scroll_offset_y(&self) -> f32 {
        if let Some(Some(target)) = self.with_physics(|physics| {
            (physics.is_running() && physics.has_target_y()).then(|| physics.target_y())
        }) {
            return target;
        }
        self.authored_scroll_offset_y()
    }

    pub fn accumulate_physics(&mut self, delta: Vec2D, time_stamp: f32) {
        self.with_physics_mut(|physics| physics.accumulate(delta, time_stamp));
    }
    pub fn set_physics(&mut self, physics: CoreHandle) {
        self.physics = Some(physics);
    }
    pub fn physics_type(&self) -> ScrollPhysicsType {
        ScrollPhysicsType::from(self.base.physics_type_value())
    }
    pub fn has_layout_parent(&self) -> bool {
        self.content_handle()
            .is_some_and(|content| content.is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY))
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
        if self.base.set_scroll_offset_x_value(value) {
            self.scroll_offset_x_changed();
            Core::notify_property_changed(self, ScrollConstraintBase::SCROLL_OFFSET_X_PROPERTY_KEY);
        }
    }
    pub fn set_authored_scroll_offset_y(&mut self, value: f32) {
        if self.base.set_scroll_offset_y_value(value) {
            self.scroll_offset_y_changed();
            Core::notify_property_changed(self, ScrollConstraintBase::SCROLL_OFFSET_Y_PROPERTY_KEY);
        }
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
