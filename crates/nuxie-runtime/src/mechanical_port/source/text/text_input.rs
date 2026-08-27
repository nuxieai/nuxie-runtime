use super::{raw_text_input::RawTextInput, text_interface::TextInterface, text_style::TextStyle};
use crate::mechanical_port::source::{
    advancing_component::{AdvanceFlags, AdvancingComponent},
    animation::listener_invocation::ListenerInvocation,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    generated::text::text_input_base::TextInputBase,
    input::{
        focusable::Focusable,
        key::{Key, KeyModifiers},
    },
    layout::{LayoutDirection, LayoutMeasureMode, LayoutScaleType},
    math::{aabb::Aabb, vec2d::Vec2D},
    renderer::Renderer,
    status_code::StatusCode,
};
use std::ptr::NonNull;
pub struct TextInput {
    pub base: TextInputBase,
    world_bounds: Aabb,
    source_text: String,
    text_style: Option<NonNull<TextStyle>>,
    scroll_constraint: Option<crate::mechanical_port::source::core::CoreHandle>,
    is_dragging: bool,
    focused: bool,
    last_drag_world_position: Vec2D,
    scroll_x: f32,
    scroll_y: f32,
    layout_width: f32,
    #[cfg(feature = "rive_text")]
    raw_text_input: RawTextInput,
}
impl TextInput {
    pub fn draw(&mut self, _renderer: &mut Renderer) {}
    pub fn hit_test(&self) -> Option<CoreHandle> {
        None
    }
    pub fn hit_test_point(&self, position: Vec2D, skip: bool, primary: bool) -> bool {
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
        self.base.layout_bounds()
    }
    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.source_text = self.base.text().to_owned();
        self.text_style = context
            .resolve(self.base.style_id())
            .and_then(|v| v.as_text_style());
        if self.text_style.is_none() {
            return StatusCode::MissingObject;
        }
        #[cfg(feature = "rive_text")]
        {
            self.raw_text_input.set_text(self.displayed_text());
            self.raw_text_input.set_multiline(self.base.multiline());
            self.raw_text_input
                .set_selection_radius(self.base.selection_radius());
        }
        StatusCode::Ok
    }
    pub fn update(&mut self, value: ComponentDirt) {
        #[cfg(feature = "rive_text")]
        if value.intersects(ComponentDirt::TEXT_SHAPE | ComponentDirt::PAINT) {
            let style = unsafe { self.text_style.expect("TextInput style").as_mut() };
            self.raw_text_input.set_style(style);
            self.raw_text_input.shape(self.layout_width);
            self.world_bounds = self
                .base
                .world_transform()
                .map_aabb(self.raw_text_input.bounds());
        }
    }
    pub fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        #[cfg(feature = "rive_text")]
        {
            let max_width = if width_mode == LayoutMeasureMode::Undefined {
                f32::INFINITY
            } else {
                width
            };
            let measured = self.raw_text_input.measure(max_width);
            return Vec2D::new(
                if width_mode == LayoutMeasureMode::Exactly {
                    width
                } else {
                    measured.x
                },
                if height_mode == LayoutMeasureMode::Exactly {
                    height
                } else {
                    measured.y
                },
            );
        }
        #[cfg(not(feature = "rive_text"))]
        {
            Vec2D::new(width, height)
        }
    }
    pub fn control_size(
        &mut self,
        size: Vec2D,
        _w: LayoutScaleType,
        _h: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
        if self.layout_width != size.x {
            self.layout_width = size.x;
            self.mark_shape_dirty();
        }
    }
    pub fn text_changed(&mut self) {
        self.source_text = self.base.text().to_owned();
        self.sync_displayed_text_from_source(true);
    }
    pub fn selection_radius_changed(&mut self) {
        #[cfg(feature = "rive_text")]
        self.raw_text_input
            .set_selection_radius(self.base.selection_radius());
        self.mark_paint_dirty();
    }
    pub fn multiline_changed(&mut self) {
        self.update_multiline(true);
    }
    fn stripped_line_breaks(text: &str) -> String {
        text.chars().filter(|c| *c != '\n' && *c != '\r').collect()
    }
    fn displayed_text(&self) -> String {
        if self.base.multiline() {
            self.source_text.clone()
        } else {
            Self::stripped_line_breaks(&self.source_text)
        }
    }
    fn sync_displayed_text_from_source(&mut self, preserve_cursor: bool) {
        #[cfg(feature = "rive_text")]
        {
            let cursor = preserve_cursor.then(|| self.raw_text_input.cursor());
            self.raw_text_input.set_text(self.displayed_text());
            if let Some(cursor) = cursor {
                self.raw_text_input.set_cursor(cursor);
            }
        }
        self.mark_shape_dirty();
    }
    fn sync_source_text_from_raw(&mut self) {
        #[cfg(feature = "rive_text")]
        {
            let displayed = self.raw_text_input.text().to_owned();
            self.source_text = if self.base.multiline() {
                displayed
            } else {
                displayed
            };
            self.base.set_text(self.source_text.clone());
        }
    }
    fn update_multiline(&mut self, sync: bool) {
        #[cfg(feature = "rive_text")]
        self.raw_text_input.set_multiline(self.base.multiline());
        if sync {
            self.sync_displayed_text_from_source(true);
        }
    }
    pub fn key_input(
        &mut self,
        key: Key,
        modifiers: KeyModifiers,
        pressed: bool,
        repeat: bool,
    ) -> bool {
        if !pressed {
            return false;
        }
        #[cfg(feature = "rive_text")]
        {
            let handled = self.raw_text_input.key_input(key, modifiers, repeat);
            if handled {
                self.sync_source_text_from_raw();
                self.mark_shape_dirty();
            }
            return handled;
        }
        #[cfg(not(feature = "rive_text"))]
        {
            false
        }
    }
    pub fn text_input(&mut self, value: &str) -> bool {
        #[cfg(feature = "rive_text")]
        {
            self.raw_text_input.insert(value);
            self.sync_source_text_from_raw();
            self.mark_shape_dirty();
            true
        }
        #[cfg(not(feature = "rive_text"))]
        {
            false
        }
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
        self.is_dragging = false;
        self.mark_paint_dirty();
    }
    pub fn world_position(&self, out: &mut Vec2D) -> bool {
        *out = self.base.world_transform() * Vec2D::new(0.0, 0.0);
        true
    }
    pub fn world_bounds(&self, out: &mut Aabb) -> bool {
        *out = self.world_bounds;
        true
    }
    fn edge_scroll_speed_for_distance(&self, d: f32) -> f32 {
        let sign = if d < 0.0 { -1.0 } else { 1.0 };
        sign * (d.abs() * 12.0).min(1200.0)
    }
    fn edge_activation_distance(&self, position: f32, edge: f32) -> f32 {
        position - edge
    }
    fn world_to_local_with_viewport(
        &mut self,
        world: Vec2D,
        out: &mut Vec2D,
        enable_scroll: bool,
    ) -> bool {
        let Some(inv) = self.base.world_transform().inverse() else {
            return false;
        };
        *out = inv * world;
        if enable_scroll {
            self.scroll_x = self.edge_scroll_speed_for_distance(
                self.edge_activation_distance(out.x, self.world_bounds.max_x),
            );
            self.scroll_y = self.edge_scroll_speed_for_distance(
                self.edge_activation_distance(out.y, self.world_bounds.max_y),
            );
        }
        true
    }
    pub fn start_drag(&mut self, world: Vec2D) {
        let mut local = Vec2D::default();
        if self.world_to_local_with_viewport(world, &mut local, true) {
            self.is_dragging = true;
            self.last_drag_world_position = world;
            #[cfg(feature = "rive_text")]
            self.raw_text_input.start_drag(local);
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
            #[cfg(feature = "rive_text")]
            self.raw_text_input.drag(local);
            self.mark_paint_dirty();
        }
    }
    pub fn end_drag(&mut self, world: Vec2D) {
        self.drag(world);
        self.is_dragging = false;
        self.scroll_x = 0.0;
        self.scroll_y = 0.0;
    }
    pub fn select_word(&mut self) {
        #[cfg(feature = "rive_text")]
        self.raw_text_input.select_word();
        self.mark_paint_dirty();
    }
    pub fn select_line(&mut self) {
        #[cfg(feature = "rive_text")]
        self.raw_text_input.select_line();
        self.mark_paint_dirty();
    }
    pub fn advance_drag(&mut self, elapsed: f32) -> bool {
        if !self.is_dragging || (self.scroll_x == 0.0 && self.scroll_y == 0.0) {
            return false;
        }
        #[cfg(feature = "rive_text")]
        {
            self.raw_text_input
                .scroll_by(Vec2D::new(self.scroll_x * elapsed, self.scroll_y * elapsed));
            self.drag(self.last_drag_world_position);
        }
        true
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
}
