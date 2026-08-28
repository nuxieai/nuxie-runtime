use super::raw_text_input::{Flags, RawTextInput};
use super::{text_interface::TextInterface, text_style::TextStyle};
use crate::mechanical_port::source::{
    advancing_component::{AdvanceFlags, AdvancingComponent},
    animation::listener_invocation::ListenerInvocation,
    artboard::Artboard,
    component_dirt::ComponentDirt,
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    core::CoreHandle,
    core_context::CoreContext,
    generated::text::text_input_base::TextInputBase,
    input::focusable::{Focusable, Key, KeyModifiers},
    layout::{LayoutDirection, LayoutMeasureMode, LayoutScaleType},
    math::{aabb::Aabb, vec2d::Vec2D},
    renderer::Renderer,
    status_code::StatusCode,
    text_engine::TextSizing,
};
pub struct TextInput {
    pub base: TextInputBase,
    world_bounds: Aabb,
    source_text: String,
    text_style: Option<CoreHandle>,
    scroll_constraint: Option<CoreHandle>,
    is_dragging: bool,
    focused: bool,
    last_drag_world_position: Vec2D,
    scroll_x: f32,
    scroll_y: f32,
    layout_width: f32,
    raw_text_input: RawTextInput,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            base: TextInputBase::default(),
            world_bounds: Aabb::default(),
            source_text: String::new(),
            text_style: None,
            scroll_constraint: None,
            is_dragging: false,
            focused: false,
            last_drag_world_position: Vec2D::new(f32::NAN, f32::NAN),
            scroll_x: 0.0,
            scroll_y: 0.0,
            layout_width: f32::NAN,
            raw_text_input: RawTextInput::default(),
        }
    }
}

impl TextInput {
    pub fn draw(&mut self, _renderer: &mut Renderer) {}
    pub fn hit_test(&self) -> Option<CoreHandle> {
        None
    }
    pub fn hit_test_point(&self, position: Vec2D, skip: bool, primary: bool) -> bool {
        let Some(inverse_world) = self.base.world_transform().inverse() else {
            return false;
        };
        if !self.local_bounds().contains(inverse_world * position) {
            return false;
        }
        self.base.component_hit_test_point(position, skip, primary)
    }
    pub fn raw_text_input(&mut self) -> &mut RawTextInput {
        &mut self.raw_text_input
    }
    pub fn mark_paint_dirty(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT);
    }
    pub fn mark_shape_dirty(&mut self) {
        self.base.add_dirt(ComponentDirt::TEXT_SHAPE);
    }
    pub fn local_bounds(&self) -> Aabb {
        self.raw_text_input.bounds()
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.base.on_added_clean(context);
        self.source_text = self.base.text().to_owned();
        self.text_style = self
            .base
            .children()
            .iter()
            .find(|child| {
                child
                    .with(|child| child.as_text_style().is_some())
                    .unwrap_or(false)
            })
            .cloned();
        {
            if let Some(style) = &self.text_style
                && let Some((font, font_size)) = style
                    .with_mut(|style| {
                        style
                            .as_text_style_mut()
                            .map(|style| (style.font(), style.base.font_size()))
                    })
                    .flatten()
                && let Some(font) = font
            {
                self.raw_text_input.set_font(Some(font));
                self.raw_text_input.set_font_size(font_size);
            }
            self.sync_displayed_text_from_source(false);
        }
        self.scroll_constraint = self
            .base
            .parent_handle()
            .and_then(|parent| {
                parent.with(|parent| {
                    parent
                        .as_component()
                        .and_then(|parent| parent.parent_handle())
                })?
            })
            .and_then(|parent| {
                parent.with(|parent| {
                    parent.as_transform_component().and_then(|parent| {
                        parent
                            .constraints()
                            .iter()
                            .find(|constraint| {
                                constraint
                                    .with_downcast::<ScrollConstraint, _>(|_| ())
                                    .is_some()
                            })
                            .cloned()
                    })
                })?
            });
        self.update_multiline(false);
        if self.text_style.is_none() {
            StatusCode::MissingObject
        } else {
            StatusCode::Ok
        }
    }
    pub fn update(&mut self, value: ComponentDirt) {
        self.base.update(value);
        if value.intersects(ComponentDirt::TEXT_SHAPE | ComponentDirt::PAINT) {
            let font_size = self
                .text_style
                .as_ref()
                .and_then(|style| {
                    style
                        .with(|style| style.as_text_style().map(|style| style.base.font_size()))
                        .flatten()
                })
                .expect("TextInput style");
            self.raw_text_input.set_font_size(font_size);
            let changed = self.raw_text_input.update(self.base.artboard().factory());
            if changed & Flags::ShapeDirty as u8 != 0 {
                self.world_bounds = self
                    .base
                    .world_transform()
                    .map_aabb(self.raw_text_input.bounds());
                if self.raw_text_input.sizing() == TextSizing::AutoHeight {
                    self.base.mark_layout_node_dirty();
                }
            }
            if changed & Flags::SelectionDirty as u8 != 0 {
                for child in self.base.children_mut() {
                    if let Some(drawable) = child.as_text_input_drawable_mut() {
                        drawable.paints.invalidate_stroke_effects();
                    }
                }
            }

            if self.scroll_x == 0.0 && self.scroll_y == 0.0 && !self.is_dragging {
                if let Some(scroll) = self.scroll_constraint.clone() {
                    scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                        let cursor = self.raw_text_input.cursor_visual_position();
                        let viewport_width = scroll.viewport_width();
                        let viewport_height = scroll.viewport_height();
                        let viewport_x = cursor.x() + scroll.authored_scroll_offset_x();
                        let viewport_top = cursor.top() + scroll.authored_scroll_offset_y();
                        let viewport_bottom = cursor.bottom() + scroll.authored_scroll_offset_y();
                        let horizontal =
                            !self.base.multiline() && scroll.base.constrains_horizontal();
                        let vertical = self.base.multiline() && scroll.base.constrains_vertical();
                        if horizontal && viewport_x < 0.0 {
                            scroll.stop_physics();
                            scroll.set_authored_scroll_offset_x(
                                scroll.authored_scroll_offset_x() - viewport_x,
                            );
                        } else if horizontal && viewport_x > viewport_width - 1.0 {
                            scroll.stop_physics();
                            scroll.set_authored_scroll_offset_x(
                                scroll.authored_scroll_offset_x()
                                    - (viewport_x - viewport_width + 1.0),
                            );
                        }
                        if vertical && viewport_top < 0.0 {
                            scroll.stop_physics();
                            scroll.set_authored_scroll_offset_y(
                                scroll.authored_scroll_offset_y() - viewport_top,
                            );
                        } else if vertical && viewport_bottom > viewport_height {
                            scroll.stop_physics();
                            scroll.set_authored_scroll_offset_y(
                                scroll.authored_scroll_offset_y()
                                    - (viewport_bottom - viewport_height),
                            );
                        }
                    });
                }
            }
        }
    }
    pub fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        let max_width = if width_mode == LayoutMeasureMode::Undefined {
            f32::MAX
        } else {
            width
        };
        let max_height = if height_mode == LayoutMeasureMode::Undefined {
            f32::MAX
        } else {
            height
        };
        self.raw_text_input.measure(max_width, max_height).size()
    }
    pub fn control_size(
        &mut self,
        size: Vec2D,
        _w: LayoutScaleType,
        _h: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
        self.layout_width = size.x;
        self.update_multiline(false);
    }
    pub fn text_changed(&mut self) {
        self.source_text = self.base.text().to_owned();
        self.sync_displayed_text_from_source(false);
        self.base.mark_layout_node_dirty();
        self.mark_shape_dirty();
    }
    pub fn selection_radius_changed(&mut self) {
        self.raw_text_input
            .set_selection_corner_radius(self.base.selection_radius());
        self.mark_shape_dirty();
    }
    pub fn multiline_changed(&mut self) {
        self.update_multiline(true);
    }
    fn stripped_line_breaks(text: &str) -> String {
        let mut stripped = String::with_capacity(text.len());
        let mut in_line_break = false;
        for character in text.chars() {
            if character == '\n' || character == '\r' {
                if !in_line_break {
                    stripped.push(' ');
                    in_line_break = true;
                }
            } else {
                stripped.push(character);
                in_line_break = false;
            }
        }
        stripped
    }
    fn displayed_text(&self) -> String {
        if self.base.multiline() {
            self.source_text.clone()
        } else {
            Self::stripped_line_breaks(&self.source_text)
        }
    }
    fn sync_displayed_text_from_source(&mut self, preserve_cursor: bool) {
        {
            let next_display_text = self.displayed_text();
            if self.raw_text_input.text() == next_display_text {
                return;
            }
            if preserve_cursor {
                self.raw_text_input
                    .set_text_preserve_cursor(next_display_text);
            } else {
                self.raw_text_input.set_text(next_display_text);
            }
        }
        self.mark_shape_dirty();
    }
    fn sync_source_text_from_raw(&mut self) {
        {
            let mut displayed = self.raw_text_input.text();
            if !self.base.multiline() {
                let single_line = Self::stripped_line_breaks(&displayed);
                if single_line != displayed {
                    self.raw_text_input
                        .set_text_preserve_cursor(single_line.clone());
                    displayed = single_line;
                }
            }
            self.source_text = displayed;
            self.base.set_text(self.source_text.clone());
        }
    }
    fn update_multiline(&mut self, sync: bool) {
        if self.base.multiline() {
            self.raw_text_input.set_max_width(self.layout_width);
            self.raw_text_input.set_sizing(TextSizing::AutoHeight);
        } else {
            self.raw_text_input.set_max_width(0.0);
            self.raw_text_input.set_sizing(TextSizing::AutoWidth);
        }
        if let Some(scroll) = self.scroll_constraint.clone() {
            scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                if self.base.multiline() && scroll.authored_scroll_offset_x() != 0.0 {
                    scroll.stop_physics();
                    scroll.set_authored_scroll_offset_x(0.0);
                } else if !self.base.multiline() && scroll.authored_scroll_offset_y() != 0.0 {
                    scroll.stop_physics();
                    scroll.set_authored_scroll_offset_y(0.0);
                }
            });
        }
        if sync {
            self.sync_displayed_text_from_source(true);
        }
        self.base.mark_layout_node_dirty();
        self.base.add_dirt(ComponentDirt::TEXT_SHAPE);
    }
    pub fn key_input(
        &mut self,
        key: Key,
        modifiers: KeyModifiers,
        pressed: bool,
        _repeat: bool,
    ) -> bool {
        {
            if pressed {
                let system_modifier = if cfg!(target_os = "windows") {
                    KeyModifiers::CTRL.0
                } else {
                    KeyModifiers::META.0
                };
                let select = modifiers.0 & KeyModifiers::SHIFT.0 != 0;
                let boundary = if modifiers.0 & KeyModifiers::META.0 != 0 {
                    super::raw_text_input::CursorBoundary::Line
                } else if modifiers.0 & KeyModifiers::ALT.0 != 0 {
                    if modifiers.0 & KeyModifiers::CTRL.0 != 0 {
                        super::raw_text_input::CursorBoundary::SubWord
                    } else {
                        super::raw_text_input::CursorBoundary::Word
                    }
                } else {
                    super::raw_text_input::CursorBoundary::Character
                };
                match key {
                    Key::Z
                        if modifiers.0 & (system_modifier | KeyModifiers::SHIFT.0)
                            == system_modifier | KeyModifiers::SHIFT.0 =>
                    {
                        self.raw_text_input.redo();
                        self.sync_source_text_from_raw();
                        self.mark_shape_dirty();
                        return true;
                    }
                    Key::Z if modifiers.0 & system_modifier != 0 => {
                        self.raw_text_input.undo();
                        self.sync_source_text_from_raw();
                        self.mark_shape_dirty();
                        return true;
                    }
                    Key::A if modifiers.0 & system_modifier != 0 => {
                        self.raw_text_input.select_all();
                        self.mark_paint_dirty();
                        return true;
                    }
                    Key::Home => self
                        .raw_text_input
                        .cursor_left(super::raw_text_input::CursorBoundary::Line, select),
                    Key::End => self
                        .raw_text_input
                        .cursor_right(super::raw_text_input::CursorBoundary::Line, select),
                    Key::Backspace => {
                        self.raw_text_input.backspace(-1);
                        self.sync_source_text_from_raw();
                        self.mark_shape_dirty();
                        return true;
                    }
                    Key::DeleteKey => {
                        self.raw_text_input.backspace(1);
                        self.sync_source_text_from_raw();
                        self.mark_shape_dirty();
                        return true;
                    }
                    Key::Left => self.raw_text_input.cursor_left(boundary, select),
                    Key::Right => self.raw_text_input.cursor_right(boundary, select),
                    Key::Up => self.raw_text_input.cursor_up(select),
                    Key::Down => self.raw_text_input.cursor_down(select),
                    Key::Enter if self.base.multiline() => {
                        self.raw_text_input.insert("\n");
                        self.sync_source_text_from_raw();
                        self.mark_paint_dirty();
                        return true;
                    }
                    Key::Enter => return false,
                    _ => return false,
                }
                self.mark_paint_dirty();
                return true;
            }
        }
        false
    }
    pub fn text_input(&mut self, value: &str) -> bool {
        let value = if self.base.multiline() {
            value.to_owned()
        } else {
            Self::stripped_line_breaks(value)
        };
        if value.is_empty() {
            return true;
        }
        self.raw_text_input.insert(&value);
        self.sync_source_text_from_raw();
        self.mark_shape_dirty();
        true
    }
    pub fn gamepad_dispatch(&mut self, _invocation: &ListenerInvocation) -> bool {
        false
    }
    pub fn focused(&mut self) {
        self.focused = true;
        self.mark_paint_dirty();
    }
    pub fn blurred(&mut self) {
        self.focused = false;
        self.raw_text_input.clear_selection();
        self.mark_paint_dirty();
    }
    pub fn world_position(&mut self, out: &mut Vec2D) -> bool {
        let local = self.base.world_transform() * Vec2D::new(0.0, 0.0);
        *out = self
            .base
            .artboard_mut()
            .map_or(local, |artboard| artboard.root_transform(local));
        true
    }
    pub fn world_bounds(&mut self, out: &mut Aabb) -> bool {
        if self.world_bounds.is_empty_or_nan() {
            return false;
        }
        if let Some(artboard) = self.base.artboard_mut() {
            let minimum = artboard.root_transform(self.world_bounds.min());
            let maximum = artboard.root_transform(self.world_bounds.max());
            *out = Aabb::new(minimum.x, minimum.y, maximum.x, maximum.y);
        } else {
            *out = self.world_bounds;
        }
        true
    }
    fn edge_scroll_speed_for_distance(&self, d: f32) -> f32 {
        const EDGE_SCROLL_BASE_SPEED: f32 = 45.0;
        const EDGE_SCROLL_MAX_SPEED: f32 = 400.0;
        const EDGE_SCROLL_SPEED_RAMP: f32 = 4.0;
        (EDGE_SCROLL_BASE_SPEED + d * EDGE_SCROLL_SPEED_RAMP)
            .clamp(EDGE_SCROLL_BASE_SPEED, EDGE_SCROLL_MAX_SPEED)
    }
    fn edge_activation_distance(&self, position: f32, edge: f32) -> f32 {
        if position >= edge {
            0.0
        } else {
            edge - position
        }
    }
    fn world_to_local_with_viewport(
        &mut self,
        world: Vec2D,
        out: &mut Vec2D,
        enable_scroll: bool,
    ) -> bool {
        self.scroll_x = 0.0;
        self.scroll_y = 0.0;
        let Some(inv) = self.base.world_transform().inverse() else {
            return false;
        };
        let mut local = inv * world;
        if let Some(scroll) = self.scroll_constraint.clone() {
            scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                let viewport_width = scroll.viewport_width();
                let viewport_height = scroll.viewport_height();
                let scroll_offset_x = scroll.authored_scroll_offset_x();
                let scroll_offset_y = scroll.authored_scroll_offset_y();
                const EDGE_THRESHOLD: f32 = 20.0;
                let horizontal = !self.base.multiline() && scroll.base.constrains_horizontal();
                let vertical = self.base.multiline() && scroll.base.constrains_vertical();
                if horizontal {
                    let viewport_x = local.x + scroll_offset_x;
                    let left_distance = self.edge_activation_distance(viewport_x, EDGE_THRESHOLD);
                    let right_distance =
                        self.edge_activation_distance(viewport_width - viewport_x, EDGE_THRESHOLD);
                    if enable_scroll && left_distance > 0.0 {
                        self.scroll_x = self.edge_scroll_speed_for_distance(left_distance);
                        if viewport_x < 0.0 {
                            local.x = -scroll_offset_x;
                        }
                    } else if enable_scroll && right_distance > 0.0 {
                        self.scroll_x = -self.edge_scroll_speed_for_distance(right_distance);
                        if viewport_x > viewport_width {
                            local.x = viewport_width - scroll_offset_x;
                        }
                    }
                }
            });
            if vertical {
                let viewport_y = local.y + scroll_offset_y;
                let top_distance = self.edge_activation_distance(viewport_y, EDGE_THRESHOLD);
                let bottom_distance =
                    self.edge_activation_distance(viewport_height - viewport_y, EDGE_THRESHOLD);
                if enable_scroll && top_distance > 0.0 {
                    self.scroll_y = self.edge_scroll_speed_for_distance(top_distance);
                    if viewport_y < 0.0 {
                        local.y = -scroll_offset_y;
                    }
                } else if enable_scroll && bottom_distance > 0.0 {
                    self.scroll_y = -self.edge_scroll_speed_for_distance(bottom_distance);
                    if viewport_y > viewport_height {
                        local.y = viewport_height - scroll_offset_y;
                    }
                }
            }
        }
        *out = local;
        true
    }
    pub fn start_drag(&mut self, world: Vec2D) {
        self.is_dragging = true;
        self.last_drag_world_position = world;
        let mut local = Vec2D::default();
        if self.world_to_local_with_viewport(world, &mut local, false) {
            self.raw_text_input.move_cursor_to(local, false);
            self.mark_paint_dirty();
        }
    }
    pub fn drag(&mut self, world: Vec2D) {
        if !self.is_dragging {
            return;
        }
        let mut local = Vec2D::default();
        if self.world_to_local_with_viewport(world, &mut local, true) {
            self.last_drag_world_position = world;
            self.raw_text_input.move_cursor_to(local, true);
            self.mark_paint_dirty();
        }
    }
    pub fn end_drag(&mut self, _world: Vec2D) {
        self.is_dragging = false;
        self.last_drag_world_position = Vec2D::new(f32::NAN, f32::NAN);
        self.scroll_x = 0.0;
        self.scroll_y = 0.0;
    }
    pub fn select_word(&mut self) {
        self.raw_text_input.select_word();
        self.mark_paint_dirty();
    }
    pub fn select_line(&mut self) {
        self.raw_text_input.select_line();
        self.mark_paint_dirty();
    }
    pub fn advance_drag(&mut self, elapsed: f32) -> bool {
        if !self.is_dragging {
            self.scroll_x = 0.0;
            self.scroll_y = 0.0;
            return false;
        }
        {
            let Some(scroll) = self.scroll_constraint.clone() else {
                return false;
            };
            if self.scroll_x == 0.0 && self.scroll_y == 0.0 {
                return false;
            }
            scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                scroll.stop_physics();
                if self.scroll_x != 0.0 {
                    let mut offset = scroll.authored_scroll_offset_x() + self.scroll_x * elapsed;
                    if !scroll.infinite() {
                        offset = offset.clamp(scroll.max_offset_x(), 0.0);
                    }
                    scroll.set_authored_scroll_offset_x(offset);
                }
                if self.scroll_y != 0.0 {
                    let mut offset = scroll.authored_scroll_offset_y() + self.scroll_y * elapsed;
                    if !scroll.infinite() {
                        offset = offset.clamp(scroll.max_offset_y(), 0.0);
                    }
                    scroll.set_authored_scroll_offset_y(offset);
                }
            });
            if self.last_drag_world_position.x.is_finite()
                && self.last_drag_world_position.y.is_finite()
            {
                let mut local = Vec2D::default();
                if self.world_to_local_with_viewport(
                    self.last_drag_world_position,
                    &mut local,
                    true,
                ) {
                    self.raw_text_input.move_cursor_to(local, true);
                    self.mark_paint_dirty();
                }
            }
        }
        self.is_dragging
    }
    pub fn advance_component(&mut self, elapsed: f32, _flags: AdvanceFlags) -> bool {
        self.advance_drag(elapsed)
    }
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }
    pub fn is_focused(&self) -> bool {
        self.focused
    }
    pub fn accepts_keyboard_input(&self) -> bool {
        true
    }
    pub fn focusable_artboard(&self) -> &Artboard {
        self.base.artboard()
    }
}
