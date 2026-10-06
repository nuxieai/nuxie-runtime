use std::{
    cell::{Cell, RefCell},
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
            virtual_layout::VirtualLayout,
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
    virtualizing_component::{self, VirtualizedDirection},
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
    virtual_layout: Option<Rc<RefCell<VirtualLayout>>>,
    virtual_inputs: RefCell<Vec<f32>>,
    virtual_versions: RefCell<Vec<u32>>,
    pending_inputs: RefCell<Vec<f32>>,
    pending_versions: RefCell<Vec<u32>>,
    column_lines: RefCell<Vec<f32>>,
    row_lines: RefCell<Vec<f32>>,
    contributions_stale: Cell<bool>,
    // Stable shared list across virtualizer callbacks; rebuilt only by dependencies.
    layout_children: Rc<Vec<CoreHandle>>,
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
    is_scrolling: bool,
    scroll_idle_seconds: f32,
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
            virtual_layout: None,
            virtual_inputs: RefCell::new(Vec::new()),
            virtual_versions: RefCell::new(Vec::new()),
            pending_inputs: RefCell::new(Vec::new()),
            pending_versions: RefCell::new(Vec::new()),
            column_lines: RefCell::new(Vec::new()),
            row_lines: RefCell::new(Vec::new()),
            contributions_stale: Cell::new(false),
            layout_children: Rc::new(Vec::new()),
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
            is_scrolling: false,
            scroll_idle_seconds: 0.0,
            intent_x: ScrollAxisIntent::default(),
            intent_y: ScrollAxisIntent::default(),
        }
    }
}

impl Drop for ScrollConstraint {
    fn drop(&mut self) {
        self.virtualizer = None;
        self.virtual_layout = None;
        Rc::make_mut(&mut self.layout_children).clear();
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
        if self.base.virtualize() && !self.virtual_axis_is_column() {
            return self.virtual_content_extent(
                self.with_content(|c| c.padding_left() + c.padding_right())
                    .unwrap(),
            );
        }
        if self.loops_x() {
            return self
                .ensure_virtual_layout()
                .map_or(0.0, |v| v.borrow().column_cycle_extent());
        }
        self.with_content(LayoutComponent::layout_width)
            .expect("ScrollConstraint content remains LayoutComponent")
    }

    pub fn content_height(&self) -> f32 {
        if self.base.virtualize() && self.virtual_axis_is_column() {
            return self.virtual_content_extent(
                self.with_content(|c| c.padding_top() + c.padding_bottom())
                    .unwrap(),
            );
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
        if self.loops_x() { f32::INFINITY } else { 0.0 }
    }
    pub fn min_offset_y(&self) -> f32 {
        if self.loops_y() { f32::INFINITY } else { 0.0 }
    }
    pub fn max_offset_x(&self) -> f32 {
        if self.loops_x() {
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
        if self.loops_y() {
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
        if self.loops_x() {
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
        if self.loops_y() {
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

    pub fn virtual_layout(&self) -> Option<Rc<RefCell<VirtualLayout>>> {
        self.virtual_layout.clone()
    }
    pub fn virtualizes_grid(&self) -> bool {
        self.virtualize()
            && self
                .with_content(LayoutComponent::is_grid_container)
                .unwrap_or(false)
            && self.direction() != DraggableConstraintDirection::Horizontal
            && self.scroll_children_are_lists()
    }
    pub fn virtualizes_grid_columns(&self) -> bool {
        self.virtualizes_grid()
            && (!self.infinite() || self.direction() == DraggableConstraintDirection::All)
    }
    pub fn indexes_grid_cells(&self) -> bool {
        self.direction() == DraggableConstraintDirection::All && self.virtualizes_grid_columns()
    }
    pub fn indexes_horizontally(&self) -> bool {
        if self.virtualize() && self.direction() == DraggableConstraintDirection::All {
            !self.virtual_axis_is_column()
        } else {
            self.constrains_horizontal()
        }
    }
    pub fn loops_x(&self) -> bool {
        self.infinite() && (!self.virtual_axis_is_column() || self.virtualizes_grid_columns())
    }
    pub fn loops_y(&self) -> bool {
        self.infinite() && self.virtual_axis_is_column()
    }
    pub fn scroll_children_are_lists(&self) -> bool {
        self.scroll_children()
            .iter()
            .all(|c| c.core_type() == Some(ArtboardComponentList::TYPE_KEY))
    }
    pub fn virtual_axis_is_column(&self) -> bool {
        if self.virtualizes_grid() {
            return true;
        }
        let column = self.main_axis_is_column();
        if self.virtualize()
            && self
                .with_content(LayoutComponent::wraps_lines)
                .unwrap_or(false)
        {
            !column
        } else {
            column
        }
    }
    fn inner_size(layout: &LayoutComponent, vertical: bool) -> f32 {
        if vertical {
            layout.layout_height() - layout.padding_top() - layout.padding_bottom()
        } else {
            layout.layout_width() - layout.padding_left() - layout.padding_right()
        }
    }
    fn virtual_content_extent(&self, padding: f32) -> f32 {
        self.ensure_virtual_layout().map_or(0.0, |v| {
            let v = v.borrow();
            if self.infinite() {
                v.cycle_extent()
            } else if v.is_grid()
                && v.rows_laid_out()
                && self
                    .with_content(|c| {
                        c.with_style(|s| {
                            s.height_scale_type()
                                == crate::source::layout::layout_enums::LayoutScaleType::Hug
                        })
                        .unwrap_or(false)
                    })
                    .unwrap_or(false)
            {
                self.with_content(LayoutComponent::layout_height).unwrap()
            } else {
                v.extent() + padding
            }
        })
    }
    fn ensure_virtual_layout(&self) -> Option<Rc<RefCell<VirtualLayout>>> {
        if self
            .virtual_layout
            .as_ref()
            .is_some_and(|v| v.borrow().segment_count() != self.scroll_children().len() as i32)
        {
            self.refresh_virtual_layout();
        }
        self.virtual_layout()
    }
    pub fn virtual_item_position(&self, child: &CoreHandle, index: i32) -> Vec2D {
        let Some(model) = self.virtual_layout() else {
            return Vec2D::default();
        };
        let Some(segment) = self.scroll_children().iter().position(|c| c == child) else {
            return Vec2D::default();
        };
        self.ensure_virtual_layout();
        if {
            let v = model.borrow();
            v.segment_start(segment as i32) + index >= v.item_count()
        } {
            self.refresh_virtual_layout();
        }
        let v = model.borrow();
        let item = v.segment_start(segment as i32) + index;
        if item >= v.item_count() {
            return Vec2D::default();
        }
        let scroll = v.line_start(v.line_of_item(item)) + v.item_line_offset(item);
        let flow = v.item_flow_offset(item);
        if self.virtual_axis_is_column() {
            Vec2D::new(flow, scroll)
        } else {
            Vec2D::new(scroll, flow)
        }
    }

    fn refresh_virtual_layout(&self) -> bool {
        use crate::source::layout::grid_track::{GridTrack, GridTrackCollection};
        let Some(model) = self.virtual_layout() else {
            return false;
        };
        let horizontal = !self.virtual_axis_is_column();
        let mut gap = self.gap();
        let children = self.scroll_children();
        let mut inputs = self.pending_inputs.borrow_mut();
        let mut versions = self.pending_versions.borrow_mut();
        inputs.clear();
        versions.clear();
        for child in children {
            if let Some(version) =
                child.with_downcast::<ArtboardComponentList, _>(|c| c.items_version())
            {
                versions.push(version);
            } else {
                versions.push(0);
                let bounds = Self::with_layout_child(child, |c| c.layout_bounds()).unwrap();
                inputs.extend([bounds.width(), bounds.height()]);
                inputs
                    .push(Self::with_layout_child(child, |c| c.num_layout_nodes()).unwrap() as f32);
            }
        }
        let grid = self.virtualizes_grid();
        let wraps = !grid && self.with_content(LayoutComponent::wraps_lines).unwrap();
        let align = self.with_content(LayoutComponent::child_alignment).unwrap();
        inputs.extend([
            grid as u8 as f32,
            wraps as u8 as f32,
            horizontal as u8 as f32,
            gap.x,
            gap.y,
            align.main as u8 as f32,
            align.cross as u8 as f32,
        ]);
        let mut columns = 0;
        let mut rows = 0;
        let mut flow_start = 0.0;
        let mut flow_extent = 0.0;
        let mut hugs = false;
        let mut flow_from_layout = true;
        if grid {
            self.with_content(|c| {
                for track in c.children() {
                    if let Some(collection) =
                        track.with_downcast::<GridTrack, _>(GridTrack::grid_collection)
                    {
                        match collection {
                            GridTrackCollection::TemplateColumns => columns += 1,
                            GridTrackCollection::TemplateRows => rows += 1,
                            _ => {}
                        }
                    }
                }
                let mut column_lines = self.column_lines.borrow_mut();
                let mut row_lines = self.row_lines.borrow_mut();
                let column_gap = c.grid_lines(false, &mut column_lines);
                let row_gap = c.grid_lines(true, &mut row_lines);
                if !column_lines.is_empty() {
                    gap.x = column_gap;
                }
                if !row_lines.is_empty() {
                    gap.y = row_gap;
                }
                inputs.extend([
                    gap.x,
                    gap.y,
                    columns as f32,
                    rows as f32,
                    column_lines.len() as f32,
                ]);
            });
        } else if wraps {
            let (wrap_hugs, start, padding) = self
                .with_content(|c| {
                    (
                        c.hugs_lines(),
                        if horizontal {
                            c.padding_top()
                        } else {
                            c.padding_left()
                        },
                        if horizontal {
                            c.padding_top() + c.padding_bottom()
                        } else {
                            c.padding_left() + c.padding_right()
                        },
                    )
                })
                .unwrap();
            hugs = wrap_hugs;
            let mut extent = if hugs {
                self.with_viewport(|p| Self::inner_size(p, horizontal))
                    .unwrap()
            } else {
                self.with_content(|c| Self::inner_size(c, horizontal))
                    .unwrap()
            };
            if hugs {
                extent -= padding;
                let reference = self.with_viewport(|p| Self::inner_size(p, false)).unwrap();
                let margins = self
                    .with_content(|c| {
                        c.with_style(|s| {
                            let resolve = |value: f32, unit: u8| {
                                if unit == 1 {
                                    value
                                } else if unit == 2 {
                                    value / 100.0 * reference
                                } else {
                                    0.0
                                }
                            };
                            if horizontal {
                                resolve(s.margin_top(), s.margin_top_units_value())
                                    + resolve(s.margin_bottom(), s.margin_bottom_units_value())
                            } else {
                                resolve(s.margin_left(), s.margin_left_units_value())
                                    + resolve(s.margin_right(), s.margin_right_units_value())
                            }
                        })
                        .unwrap_or(0.0)
                    })
                    .unwrap();
                extent -= margins;
            }

            flow_start = start;
            flow_extent = extent;
            flow_from_layout = !hugs && !self.infinite() && self.scroll_children_are_lists();
            inputs.extend([
                flow_start,
                flow_extent,
                hugs as u8 as f32,
                flow_from_layout as u8 as f32,
            ]);
        }
        if *inputs == *self.virtual_inputs.borrow()
            && *versions == *self.virtual_versions.borrow()
            && {
                let mut model = model.borrow_mut();
                model.segment_count() == children.len() as i32
                    && (!grid
                        || (*self.column_lines.borrow() == *model.grid_column_starts()
                            && *self.row_lines.borrow() == *model.grid_row_starts()))
            }
        {
            return false;
        }
        std::mem::swap(&mut *self.virtual_inputs.borrow_mut(), &mut *inputs);
        std::mem::swap(&mut *self.virtual_versions.borrow_mut(), &mut *versions);
        self.contributions_stale.set(true);
        let mut v = model.borrow_mut();
        if grid {
            v.begin_grid(gap.y, columns, align.cross, gap.x, rows);
            v.grid_column_starts()
                .clone_from(&self.column_lines.borrow());
            v.grid_row_starts().clone_from(&self.row_lines.borrow());
        } else if wraps {
            v.begin_wrap(
                if horizontal { gap.x } else { gap.y },
                if horizontal { gap.y } else { gap.x },
                flow_start,
                flow_extent,
                align.main,
                align.cross,
                hugs,
            );
            v.set_flow_from_layout(flow_from_layout);
        } else {
            v.begin_linear(if horizontal { gap.x } else { gap.y });
        }
        for child in children {
            v.begin_segment();
            let count = Self::with_layout_child(child, |c| c.num_layout_nodes()).unwrap();
            for index in 0..count {
                let size = child
                    .with_downcast::<ArtboardComponentList, _>(|c| {
                        crate::source::virtualizing_component::VirtualizingComponent::item_size(
                            c,
                            index as i32,
                        )
                    })
                    .unwrap_or_else(|| {
                        let b = Self::with_layout_child(child, |c| c.layout_bounds()).unwrap();
                        Vec2D::new(b.width(), b.height())
                    });
                v.add_item(
                    if horizontal { size.x } else { size.y },
                    if horizontal { size.y } else { size.x },
                );
            }
        }
        v.end();
        true
    }

    fn anchor_scroll(&mut self, moved: f32, column: bool) {
        let offset = if column {
            self.offset_y()
        } else {
            self.offset_x()
        };
        if moved == 0.0 || offset >= 0.0 {
            return;
        }
        if column {
            self.set_authored_scroll_offset_y(offset - moved);
            self.set_offset_y(offset - moved);
            self.last_frame_offset_y -= moved;
        } else {
            self.set_authored_scroll_offset_x(offset - moved);
            self.set_offset_x(offset - moved);
            self.last_frame_offset_x -= moved;
        }
        self.with_physics_mut(|p| {
            p.shift(if column {
                Vec2D::new(0.0, -moved)
            } else {
                Vec2D::new(-moved, 0.0)
            })
        });
    }

    fn sync_virtual_grid_contributions(&self) {
        self.contributions_stale.set(false);
        if self.content_handle().is_none() {
            return;
        }
        let grid = self.virtualizes_grid()
            && self
                .virtual_layout
                .as_ref()
                .is_some_and(|v| v.borrow().is_grid());
        if grid {
            let v = self.virtual_layout.as_ref().unwrap().borrow();
            self.with_content_mut(|c| {
                c.virtual_grid_contributions(v.grid_row_contents(), v.grid_column_contents())
            });
        } else {
            self.with_content_mut(|c| c.virtual_grid_contributions(&[], &[]));
        }
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
        let Some((virtualizer, children)) = owner
            .with_downcast::<Self, _>(|scroll| {
                if !scroll.base.virtualize() {
                    return None;
                }
                let virtualizer = scroll.virtualizer.clone()?;
                let children = scroll.layout_children.clone();
                if scroll.child_constraint_applied_count < children.len() as i32 && !force {
                    return None;
                }
                scroll.refresh_virtual_layout();
                Some((virtualizer, children))
            })
            .expect("live ScrollConstraint")
        else {
            return;
        };
        let (moved, column) = owner
            .with_downcast::<Self, _>(|s| {
                let model = s.virtual_layout.as_ref().unwrap();
                (
                    virtualizer
                        .borrow()
                        .anchor_moved(&model.borrow(), &children),
                    s.virtual_axis_is_column(),
                )
            })
            .unwrap();
        owner.with_downcast_mut::<Self, _>(|s| s.anchor_scroll(moved, column));
        let (offset, direction, flow, column_start, column_viewport, columns) = owner
            .with_downcast::<Self, _>(|s| {
                let direction = if column {
                    VirtualizedDirection::Vertical
                } else {
                    VirtualizedDirection::Horizontal
                };
                let offset = if column {
                    s.clamped_offset_y()
                } else {
                    s.clamped_offset_x()
                };
                let scrolls_flow = if column {
                    s.constrains_horizontal()
                } else {
                    s.constrains_vertical()
                };
                let flow = if scrolls_flow {
                    -(if column {
                        s.clamped_offset_x()
                    } else {
                        s.clamped_offset_y()
                    })
                } else {
                    0.0
                };
                let columns = s.virtualizes_grid_columns();
                (
                    offset,
                    direction,
                    flow,
                    flow - s.with_content(LayoutComponent::layout_x).unwrap(),
                    if columns {
                        s.with_viewport(LayoutComponent::layout_width).unwrap()
                    } else {
                        0.0
                    },
                    columns,
                )
            })
            .unwrap();
        virtualizer.borrow_mut().constrain(
            owner,
            &children,
            offset,
            direction,
            flow,
            column_start,
            column_viewport,
            columns,
        );
        owner.with_downcast::<Self, _>(|scroll| {
            if scroll.contributions_stale.get() {
                scroll.sync_virtual_grid_contributions();
            }
        });
    }
    pub fn add_layout_child(&mut self, child: CoreHandle) {
        assert!(!self.layout_children.contains(&child));
        Rc::make_mut(&mut self.layout_children).push(child);
    }
    #[cfg(any(test, feature = "testing"))]
    pub fn child_constraint_applied_count(&self) -> i32 {
        self.child_constraint_applied_count
    }
    pub fn scroll_children(&self) -> &[CoreHandle] {
        self.layout_children.as_slice()
    }

    pub fn drag_view(&mut self, delta: Vec2D, time_stamp: f32, track_velocity: bool) {
        let scaled = Vec2D::new(
            delta.x * self.base.drag_multiplier(),
            delta.y * self.base.drag_multiplier(),
        );
        if self.physics.is_some() {
            if track_velocity {
                self.with_physics_mut(|physics| physics.accumulate(scaled, time_stamp))
                    .expect("ScrollConstraint physics remains ScrollPhysics-derived");
            }
            self.set_authored_scroll_offset_x(self.offset_x() + scaled.x);
            self.set_authored_scroll_offset_y(self.offset_y() + scaled.y);
            return;
        }
        let mut x = self.offset_x() + scaled.x;
        let mut y = self.offset_y() + scaled.y;
        if !self.loops_x() {
            x = if self.max_offset_x() > 0.0 {
                0.0
            } else {
                math_types::clamp(x, self.max_offset_x(), 0.0)
            };
        }
        if !self.loops_y() {
            y = if self.max_offset_y() > 0.0 {
                0.0
            } else {
                math_types::clamp(y, self.max_offset_y(), 0.0)
            };
        }
        self.set_authored_scroll_offset_x(x);
        self.set_authored_scroll_offset_y(y);
    }

    pub fn wheel_enabled(&self) -> bool {
        self.interactive() && self.wheel_interactive()
    }
    pub fn is_scrolling(&self) -> bool {
        self.is_scrolling
    }
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }
    pub fn scaled_delta(&self, delta: Vec2D) -> Vec2D {
        Vec2D::new(
            delta.x * self.drag_multiplier(),
            delta.y * self.drag_multiplier(),
        )
    }
    pub fn is_overscrolled(&self) -> bool {
        (self.constrains_horizontal()
            && !self.loops_x()
            && self.offset_x() != self.clamp_resolved_offset(self.offset_x(), true))
            || (self.constrains_vertical()
                && !self.loops_y()
                && self.offset_y() != self.clamp_resolved_offset(self.offset_y(), false))
    }
    pub fn can_stretch(&self, raw_delta: Vec2D) -> bool {
        let delta = self.scaled_delta(raw_delta);
        if !self.wheel_enabled()
            || !self.has_layout_parent()
            || !self.viewport_handle().is_some_and(|p| {
                p.with(|p| p.as_layout_component().is_some())
                    .unwrap_or(false)
            })
            || self.physics.is_none()
            || self.physics_type() != ScrollPhysicsType::Elastic
        {
            return false;
        }
        let wants_x = self.constrains_horizontal() && delta.x != 0.0;
        let wants_y = self.constrains_vertical() && delta.y != 0.0;
        (wants_x && (self.loops_x() || self.max_offset_x() < 0.0))
            || (wants_y && (self.loops_y() || self.max_offset_y() < 0.0))
    }
    pub fn can_consume(&self, raw_delta: Vec2D) -> bool {
        let delta = self.scaled_delta(raw_delta);
        if !self.wheel_enabled()
            || !self.has_layout_parent()
            || !self.viewport_handle().is_some_and(|p| {
                p.with(|p| p.as_layout_component().is_some())
                    .unwrap_or(false)
            })
        {
            return false;
        }
        let wants_x = self.constrains_horizontal() && delta.x != 0.0;
        let wants_y = self.constrains_vertical() && delta.y != 0.0;
        let moves_x = self.loops_x()
            || self.clamp_resolved_offset(self.offset_x() + delta.x, true)
                != self.clamp_resolved_offset(self.offset_x(), true);
        let moves_y = self.loops_y()
            || self.clamp_resolved_offset(self.offset_y() + delta.y, false)
                != self.clamp_resolved_offset(self.offset_y(), false);
        (wants_x && moves_x) || (wants_y && moves_y)
    }
    pub fn scroll_by(&mut self, delta: Vec2D) {
        let scaled = self.scaled_delta(delta);
        if self.constrains_horizontal() {
            self.set_authored_scroll_offset_x(
                self.clamp_resolved_offset(self.offset_x() + scaled.x, true),
            );
        }
        if self.constrains_vertical() {
            self.set_authored_scroll_offset_y(
                self.clamp_resolved_offset(self.offset_y() + scaled.y, false),
            );
        }
    }
    pub fn mark_scroll_activity(&mut self) {
        self.scroll_idle_seconds = 0.0;
    }
    pub fn begin_scroll_gesture(&mut self) -> bool {
        self.mark_scroll_activity();
        if self.is_scrolling {
            return false;
        }
        self.is_scrolling = true;
        self.clear_scroll_intents();
        self.last_frame_offset_x = self.authored_scroll_offset_x();
        self.last_frame_offset_y = self.authored_scroll_offset_y();
        true
    }
    pub fn end_scroll_gesture(&mut self) {
        self.is_scrolling = false;
        self.scroll_idle_seconds = 0.0;
    }

    fn collect_snap_points(&self) -> Vec<Vec2D> {
        let mut points = Vec::new();
        for child in self.layout_children.iter() {
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
        self.start_physics();
    }
    pub fn start_physics(&mut self) {
        let points = if self.base.snap() {
            self.collect_snap_points()
        } else {
            Vec::new()
        };
        if self.physics.is_none() {
            return;
        }
        let args = (
            Vec2D::new(self.max_offset_x(), self.max_offset_y()),
            Vec2D::new(self.min_offset_x(), self.min_offset_y()),
            Vec2D::new(self.offset_x(), self.offset_y()),
            Vec2D::new(self.content_width(), self.content_height()),
            Vec2D::new(self.viewport_width(), self.viewport_height()),
        );
        self.with_physics_mut(|physics| {
            physics.run(args.0, args.1, args.2, points, args.3, args.4)
        });
    }

    pub fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        if self.base.is_collapsed() {
            if self.is_scrolling {
                self.stop_physics();
                self.clear_velocity();
                self.end_scroll_gesture();
            }
            return false;
        }
        if !flags.contains(AdvanceFlags::ADVANCE_NESTED) {
            return false;
        }
        if self.is_scrolling && flags.contains(AdvanceFlags::NEW_FRAME) {
            self.scroll_idle_seconds += elapsed_seconds;
            if self.scroll_idle_seconds >= crate::source::scroll_event::SCROLL_IDLE_SECONDS {
                if self.snap() || self.is_overscrolled() {
                    self.prime_physics();
                    self.start_physics();
                }
                self.end_scroll_gesture();
            }
        }
        if self.physics.is_none() {
            return self.is_scrolling;
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
            if (self.is_scroll_bar_dragging || self.is_dragging || self.is_scrolling) && !moved {
                self.clear_velocity();
            }
            self.last_frame_offset_x = self.authored_scroll_offset_x();
            self.last_frame_offset_y = self.authored_scroll_offset_y();
        }
        self.with_physics(|physics| physics.enabled())
            .expect("ScrollConstraint physics remains ScrollPhysics-derived")
            || self.is_scroll_bar_dragging
            || self.is_dragging
            || self.is_scrolling
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
        if let Some(viewport) = self.viewport_handle() {
            viewport.with_mut(|viewport| {
                if let Some(layout) = viewport.as_layout_component_mut() {
                    layout.mark_interaction_target();
                }
            });
        }
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
            self.virtual_layout = Some(Rc::new(RefCell::new(VirtualLayout::default())));
        }
        self.set_offset_x(self.authored_scroll_offset_x());
        self.set_offset_y(self.authored_scroll_offset_y());
        result
    }

    pub fn virtualize_buffer_changed(&mut self) {
        self.mark_virtualization_constraint_dirty();
    }
    fn mark_virtualization_constraint_dirty(&mut self) {
        let parent = self
            .parent_handle()
            .expect("Constraint parent was validated");
        let occurrence = crate::source::component::ComponentOccurrenceHandle::Authored(parent);
        // Constraint::markConstraintDirty marks its parent transform. The
        // dependency walk can return to this active ScrollConstraint.
        if occurrence.add_dirt_from_scroll(self, ComponentDirt::TRANSFORM, false) {
            occurrence.add_dirt_from_scroll(self, ComponentDirt::WORLD_TRANSFORM, true);
        }
    }
    fn begin_virtualize_changed(&mut self) -> Option<Rc<Vec<CoreHandle>>> {
        if self.virtualize() {
            self.virtualizer
                .get_or_insert_with(|| Rc::new(RefCell::new(ScrollVirtualizer::default())));
            self.virtual_layout
                .get_or_insert_with(|| Rc::new(RefCell::new(VirtualLayout::default())));
            self.mark_virtualization_constraint_dirty();
            return None;
        }
        self.virtualizer = None;
        self.virtual_layout = None;
        if !self.has_layout_parent() {
            return None;
        }
        self.sync_virtual_grid_contributions();
        Some(self.layout_children.clone())
    }
    fn realize_all_children(children: &[CoreHandle], active_scroll: Option<&Self>) {
        for child in children {
            let Some(()) = ScrollVirtualizer::with_virtualizer_mut(child, |virt| {
                let mut held = Vec::new();
                virt.realized_indices(&mut held);
                for index in held {
                    virt.set_virtualizable_cell(index, -1, -1);
                }
                virt.clear_virtual_window();
            }) else {
                continue;
            };
            let mut index = 0;
            while index
                < ScrollVirtualizer::with_virtualizer_mut(child, |virt| virt.item_count()).unwrap()
            {
                if ScrollVirtualizer::with_virtualizer_mut(child, |virt| virt.item(index).is_none())
                    .unwrap()
                {
                    virtualizing_component::add_virtualizable_handle_with_scroll(
                        child,
                        index,
                        active_scroll,
                    );
                }
                index += 1;
            }
            ScrollVirtualizer::with_virtualizer_mut(child, |virt| virt.virtualizable_changed());
        }
    }
    pub fn virtualize_changed_occurrence(owner: &CoreHandle) {
        let Some(children) = owner
            .with_downcast_mut::<Self, _>(Self::begin_virtualize_changed)
            .flatten()
        else {
            return;
        };
        Self::realize_all_children(&children, None);
        crate::source::constraints::constraint::Constraint::mark_constraint_dirty_occurrence(owner);
    }
    pub fn virtualize_changed(&mut self) {
        if let Some(children) = self.begin_virtualize_changed() {
            Self::realize_all_children(&children, Some(self));
            self.mark_virtualization_constraint_dirty();
        }
    }

    pub fn init_physics(&mut self) {
        self.is_dragging = true;
        self.prime_physics();
    }
    pub fn ensure_physics_primed(&mut self) -> bool {
        if self
            .with_physics(|physics| !physics.is_primed())
            .unwrap_or(false)
        {
            self.prime_physics();
            return true;
        }
        false
    }
    pub fn prime_physics(&mut self) {
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
        if self.loops_x() {
            self.content_width()
        } else {
            self.max_offset_x()
        }
    }
    fn max_offset_y_for_percent(&self) -> f32 {
        if self.loops_y() {
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
            || self.is_scrolling
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
        let intent = if self.indexes_horizontally() {
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
    /// Computed scroll writes can publish X and then Y. Complete X's observer
    /// notification before inspecting Y's live enablement, as the source does.
    pub(crate) fn set_scroll_value_occurrence(owner: &CoreHandle, key: u16, value: f32) -> bool {
        let Some(proceed) = owner.with_downcast_mut::<Self, _>(|scroll| {
            if scroll.is_dragging {
                return false;
            }
            scroll.stop_physics();
            true
        }) else {
            return false;
        };
        if !proceed {
            return true;
        }
        let index = key == ScrollConstraintBase::SCROLL_INDEX_PROPERTY_KEY;
        for is_x in [true, false] {
            let mut completion = crate::source::core::PropertySetterCompletion::default();
            owner.with_downcast_mut::<Self, _>(|scroll| {
                let enabled = if index {
                    if is_x {
                        scroll.base.constrains_horizontal()
                    } else {
                        scroll.base.constrains_vertical()
                    }
                } else {
                    key == if is_x {
                        ScrollConstraintBase::SCROLL_PERCENT_X_PROPERTY_KEY
                    } else {
                        ScrollConstraintBase::SCROLL_PERCENT_Y_PROPERTY_KEY
                    }
                };
                if !enabled {
                    return;
                }
                let intent = ScrollAxisIntent {
                    space: if index {
                        ScrollSpace::Index
                    } else {
                        ScrollSpace::Percent
                    },
                    value,
                };
                if is_x {
                    scroll.set_intent_x_with_completion(intent, &mut completion);
                } else {
                    scroll.set_intent_y_with_completion(intent, &mut completion);
                }
            });
            completion.finish();
        }
        true
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
        if if is_x { self.loops_x() } else { self.loops_y() } {
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
        let mut completion = crate::source::core::PropertySetterCompletion::default();
        self.set_intent_x_with_completion(intent, &mut completion);
        completion.finish();
    }
    fn set_intent_x_with_completion(
        &mut self,
        intent: ScrollAxisIntent,
        completion: &mut crate::source::core::PropertySetterCompletion,
    ) {
        if let Some(offset) = self.resolve_intent(intent, true) {
            self.intent_x.space = ScrollSpace::None;
            self.set_authored_scroll_offset_x_with_completion(offset, completion);
        } else {
            self.intent_x = intent;
        }
    }
    fn set_intent_y(&mut self, intent: ScrollAxisIntent) {
        let mut completion = crate::source::core::PropertySetterCompletion::default();
        self.set_intent_y_with_completion(intent, &mut completion);
        completion.finish();
    }
    fn set_intent_y_with_completion(
        &mut self,
        intent: ScrollAxisIntent,
        completion: &mut crate::source::core::PropertySetterCompletion,
    ) {
        if let Some(offset) = self.resolve_intent(intent, false) {
            self.intent_y.space = ScrollSpace::None;
            self.set_authored_scroll_offset_y_with_completion(offset, completion);
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
        for child in self.layout_children.iter() {
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
            for child in self.layout_children.iter() {
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
        if self.indexes_grid_cells() && self.virtual_layout.is_some() {
            let model = self.ensure_virtual_layout().unwrap();
            let v = model.borrow();
            if v.line_count() == 0 {
                return 0.0;
            }
            let mut y = -position.y;
            let cycle = v.cycle_extent();
            if self.loops_y() && cycle > 0.0 {
                y -= (y / cycle).floor() * cycle;
            }
            let row = v.window(y, 0.0, self.loops_y(), 0).visible_start;
            let column = v.wrap_column(
                v.column_window(-position.x, 0.0, 0, self.loops_x())
                    .visible_start,
            );
            return (v.line_first_item(row) + column).min(v.line_last_item(row)) as f32;
        }
        let gap = self.gap();
        if !self.has_list_children {
            let count = self.layout_children.len();
            if self.indexes_horizontally() {
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
        if self.indexes_horizontally() {
            for child in self.layout_children.iter() {
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
            for child in self.layout_children.iter() {
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
        for child in self.layout_children.iter() {
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
        let mut completion = crate::source::core::PropertySetterCompletion::default();
        self.set_authored_scroll_offset_x_with_completion(value, &mut completion);
        completion.finish();
    }
    fn set_authored_scroll_offset_x_with_completion(
        &mut self,
        value: f32,
        completion: &mut crate::source::core::PropertySetterCompletion,
    ) {
        if self.base.set_scroll_offset_x_value(value) {
            self.scroll_offset_x_changed();
            completion.record(self, ScrollConstraintBase::SCROLL_OFFSET_X_PROPERTY_KEY);
        }
    }
    pub fn set_authored_scroll_offset_y(&mut self, value: f32) {
        let mut completion = crate::source::core::PropertySetterCompletion::default();
        self.set_authored_scroll_offset_y_with_completion(value, &mut completion);
        completion.finish();
    }
    fn set_authored_scroll_offset_y_with_completion(
        &mut self,
        value: f32,
        completion: &mut crate::source::core::PropertySetterCompletion,
    ) {
        if self.base.set_scroll_offset_y_value(value) {
            self.scroll_offset_y_changed();
            completion.record(self, ScrollConstraintBase::SCROLL_OFFSET_Y_PROPERTY_KEY);
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

#[cfg(test)]
mod virtualize_setter_tests {
    use super::*;
    use crate::source::{
        core::CoreArena,
        generated::core_registry::{CoreField, CoreRegistry, CoreRegistryObject},
        node::Node,
    };

    #[test]
    fn virtualize_callbacks_run_for_borrowed_and_released_setters() {
        struct Context<'a> {
            arena: &'a CoreArena,
            parent: CoreHandle,
        }
        impl CoreContext for Context<'_> {
            fn core_arena(&self) -> &CoreArena {
                self.arena
            }
            fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
                (id == 1).then(|| self.parent.clone())
            }
        }
        let arena = CoreArena::default();
        for path in 0..3 {
            let parent = arena.insert(Node::default());
            let handle = arena.insert(ScrollConstraint::default());
            handle.with_mut(|owner| {
                owner
                    .as_component_mut()
                    .unwrap()
                    .base
                    .set_parent_id_value(1);
            });
            let mut context = Context {
                arena: &arena,
                parent,
            };
            assert_eq!(
                handle.with_downcast_mut::<ScrollConstraint, _>(
                    |scroll| scroll.on_added_dirty(&mut context)
                ),
                Some(StatusCode::Ok),
            );
            for value in [true, false] {
                match path {
                    0 => {
                        handle.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                            CoreRegistry::set_bool(scroll, 850, value);
                        });
                    }
                    1 => {
                        handle.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                            CoreRegistryObject::set_bool(
                                scroll,
                                CoreField::ScrollConstraintVirtualize,
                                value,
                            );
                        });
                    }
                    _ => {
                        assert!(CoreRegistry::set_bool_handle(&handle, 850, value));
                    }
                }
                handle
                    .with_downcast::<ScrollConstraint, _>(|scroll| {
                        assert_eq!(scroll.virtualize(), value);
                        assert_eq!(scroll.virtual_layout.is_some(), value);
                        assert_eq!(scroll.virtualizer.is_some(), value);
                    })
                    .unwrap();
            }
        }
    }
}
