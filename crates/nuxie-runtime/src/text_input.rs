//! Retained `TextInput` occurrence owner ported from `src/text/text_input.cpp`.

use crate::components::ComponentDirt;
use crate::properties::property_key_for_name;
#[cfg(any(test, feature = "tools"))]
use crate::text::cursor::{Cursor, CursorPosition};
use crate::text::raw_text_input::CursorBoundary;
use crate::{ArtboardInstance, RuntimePathCommand};
use nuxie_render_api::Vec2D as RenderVec2D;

const KEY_A: u32 = 65;
const KEY_Z: u32 = 90;
const KEY_ENTER: u32 = 257;
const KEY_BACKSPACE: u32 = 259;
const KEY_DELETE: u32 = 261;
const KEY_RIGHT: u32 = 262;
const KEY_LEFT: u32 = 263;
const KEY_DOWN: u32 = 264;
const KEY_UP: u32 = 265;
const KEY_HOME: u32 = 268;
const KEY_END: u32 = 269;

const MOD_SHIFT: u32 = 1;
const MOD_CTRL: u32 = 2;
const MOD_ALT: u32 = 4;
const MOD_META: u32 = 8;

fn system_modifier() -> u32 {
    if cfg!(target_os = "windows") {
        MOD_CTRL
    } else {
        MOD_META
    }
}

fn strip_line_breaks(text: &str) -> String {
    let mut stripped = String::with_capacity(text.len());
    let mut in_line_break = false;
    for character in text.chars() {
        if matches!(character, '\n' | '\r') {
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

fn char_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map_or(text.len(), |(byte, _)| byte)
}

fn edge_scroll_speed_for_distance(distance_from_edge: f32) -> f32 {
    const BASE_SPEED: f32 = 45.0;
    const MAX_SPEED: f32 = 400.0;
    const SPEED_RAMP: f32 = 4.0;
    (BASE_SPEED + distance_from_edge * SPEED_RAMP).clamp(BASE_SPEED, MAX_SPEED)
}

fn edge_activation_distance(position: f32, edge_start: f32) -> f32 {
    if position >= edge_start {
        0.0
    } else {
        edge_start - position
    }
}

impl ArtboardInstance {
    pub(crate) fn initialize_text_inputs(&self) {
        let locals = self
            .components()
            .iter()
            .filter(|component| component.concrete.text_input.is_some())
            .map(|component| component.local_id)
            .collect::<Vec<_>>();
        for local_id in locals {
            if self.initialize_text_input(local_id) {
                self.refresh_text_input_geometry(local_id);
            }
        }
    }

    pub(crate) fn refresh_text_input_geometry(&self, local_id: usize) -> bool {
        let Some((runtime, graph)) = self.runtime_file().zip(self.runtime_graph()) else {
            return false;
        };
        let constraint = self.runtime_retained_text_input_layout_constraint(local_id, graph);
        let geometry =
            crate::text::build_text_input_geometry(runtime, graph, self, local_id, constraint);
        let Some(state) = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
        else {
            return false;
        };
        state
            .layout_width
            .set(constraint.map_or(f32::NAN, |constraint| constraint.width));
        let world_bounds = geometry
            .as_ref()
            .and_then(crate::text::TextInputGeometry::local_bounds)
            .map(|(x, y, width, height)| {
                let world = self.runtime_component_world_transform(local_id, graph);
                let points = [
                    world.transform_point(x, y),
                    world.transform_point(x + width, y),
                    world.transform_point(x + width, y + height),
                    world.transform_point(x, y + height),
                ];
                let min_x = points
                    .iter()
                    .map(|point| point.0)
                    .fold(f32::INFINITY, f32::min);
                let min_y = points
                    .iter()
                    .map(|point| point.1)
                    .fold(f32::INFINITY, f32::min);
                let max_x = points
                    .iter()
                    .map(|point| point.0)
                    .fold(f32::NEG_INFINITY, f32::max);
                let max_y = points
                    .iter()
                    .map(|point| point.1)
                    .fold(f32::NEG_INFINITY, f32::max);
                (min_x, min_y, max_x - min_x, max_y - min_y)
            });
        state.world_bounds.set(world_bounds);
        state.raw.borrow_mut().set_geometry(geometry)
    }

    fn ensure_text_input_geometry(&self, local_id: usize) -> bool {
        let dirty = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
            .is_some_and(|state| state.raw.borrow().geometry_dirty());
        if dirty {
            self.refresh_text_input_geometry(local_id)
        } else {
            true
        }
    }

    fn text_input_multiline(&self, local_id: usize) -> bool {
        property_key_for_name("TextInput", "multiline")
            .and_then(|key| self.bool_property(local_id, key))
            .unwrap_or(true)
    }

    fn initialize_text_input(&self, local_id: usize) -> bool {
        let Some(state) = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
        else {
            return false;
        };
        if state.source_text.borrow().is_some() {
            return true;
        }
        let source = property_key_for_name("TextInput", "text")
            .and_then(|key| self.string_property(local_id, key))
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .unwrap_or_default()
            .to_owned();
        let display = if self.text_input_multiline(local_id) {
            source.clone()
        } else {
            strip_line_breaks(&source)
        };
        let has_selected_text =
            crate::text::text_input_selected_text::on_added_clean(self, local_id);
        if let Some(graph) = self.runtime_graph() {
            state.text_style.set(
                graph
                    .components
                    .iter()
                    .find(|component| {
                        component.parent_local == Some(local_id)
                            && component.type_name == "TextStyle"
                    })
                    .and_then(|component| self.component_handle(component.local_id)),
            );
        }
        *state.source_text.borrow_mut() = Some(source);
        let mut raw = state.raw.borrow_mut();
        raw.separate_selection_text = has_selected_text;
        raw.set_text(&display);
        true
    }

    fn ensure_text_input_initialized(&self, local_id: usize) -> bool {
        self.component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
            .is_some_and(|state| state.source_text.borrow().is_some())
    }

    pub(crate) fn text_input_display_text(&self, local_id: usize) -> Option<String> {
        self.ensure_text_input_initialized(local_id);
        Some(
            self.component(local_id)?
                .concrete
                .text_input
                .as_ref()?
                .raw
                .borrow()
                .text(),
        )
    }

    pub(crate) fn text_input_property_changed(&mut self, local_id: usize) -> bool {
        let Some(key) = property_key_for_name("TextInput", "text") else {
            return false;
        };
        let source = self
            .string_property(local_id, key)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .unwrap_or_default()
            .to_owned();
        let display = if self.text_input_multiline(local_id) {
            source.clone()
        } else {
            strip_line_breaks(&source)
        };
        let Some(state) = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
        else {
            return false;
        };
        *state.source_text.borrow_mut() = Some(source);
        state.raw.borrow_mut().set_text(&display);
        crate::text_owner::mark_shape_dirty(self, local_id)
    }

    pub(crate) fn text_input_multiline_changed(&mut self, local_id: usize) -> bool {
        self.ensure_text_input_initialized(local_id);
        let multiline = self.text_input_multiline(local_id);
        let Some(state) = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
        else {
            return false;
        };
        let source = state.source_text.borrow().clone().unwrap_or_default();
        let display = if multiline {
            source
        } else {
            strip_line_breaks(&source)
        };
        state.raw.borrow_mut().set_text_preserve_cursor(&display);
        let scroll_constraint = state.scroll_constraint;
        if let Some(constraint) = scroll_constraint {
            crate::constraints::reset_text_input_cross_axis_scroll(self, constraint, multiline);
        }
        crate::text_owner::mark_shape_dirty(self, local_id)
    }

    pub(crate) fn text_input_selection_radius_changed(&mut self, local_id: usize) -> bool {
        let radius = property_key_for_name("TextInput", "selectionRadius")
            .and_then(|key| self.double_property(local_id, key))
            .unwrap_or(5.0);
        let Some(state) = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
        else {
            return false;
        };
        if !state.raw.borrow_mut().set_selection_corner_radius(radius) {
            return false;
        }
        self.add_dirt(local_id, ComponentDirt::PATH, false)
    }

    fn sync_text_input_source_from_raw(&mut self, local_id: usize) -> bool {
        let Some(display) = self.text_input_display_text(local_id) else {
            return false;
        };
        let source = if self.text_input_multiline(local_id) {
            display
        } else {
            strip_line_breaks(&display)
        };
        if let Some(state) = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
        {
            *state.source_text.borrow_mut() = Some(source.clone());
        }
        property_key_for_name("TextInput", "text")
            .is_some_and(|key| self.set_string_property(local_id, key, source.into_bytes()))
    }

    fn text_input_cursor_boundary(modifiers: u32) -> CursorBoundary {
        if modifiers & MOD_META != 0 {
            CursorBoundary::Line
        } else if modifiers & MOD_ALT != 0 {
            if modifiers & MOD_CTRL != 0 {
                CursorBoundary::SubWord
            } else {
                CursorBoundary::Word
            }
        } else {
            CursorBoundary::Character
        }
    }

    fn text_input_line_range_for_cursor(&self, local_id: usize) -> Option<std::ops::Range<usize>> {
        self.ensure_text_input_initialized(local_id);
        self.ensure_text_input_geometry(local_id);
        let state = self.component(local_id)?.concrete.text_input.as_ref()?;
        let raw = state.raw.borrow();
        let cursor = raw.cursor();
        raw.geometry()?
            .line_range(cursor.end.codepoint_index)
            .filter(|range| {
                range.start <= cursor.end.codepoint_index
                    && cursor.end.codepoint_index <= range.end
                    && (range.end > range.start
                        || self
                            .text_input_display_text(local_id)
                            .is_some_and(|text| text.is_empty()))
            })
            .or_else(|| {
                let text = self.text_input_display_text(local_id)?;
                let chars = text.chars().collect::<Vec<_>>();
                let at = cursor.end.codepoint_index.min(chars.len());
                let start = chars[..at]
                    .iter()
                    .rposition(|character| *character == '\n')
                    .map_or(0, |index| index + 1);
                let end = chars[at..]
                    .iter()
                    .position(|character| *character == '\n')
                    .map_or(chars.len(), |index| at + index);
                Some(start..end)
            })
    }

    pub(crate) fn text_input_key_input(
        &mut self,
        local_id: usize,
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        _is_repeat: bool,
    ) -> bool {
        if !is_pressed || !self.ensure_text_input_initialized(local_id) {
            return false;
        }
        let select = modifiers & MOD_SHIFT != 0;
        let system = system_modifier();
        let horizontal_boundary = Self::text_input_cursor_boundary(modifiers);
        let line_range = (matches!(key, KEY_HOME | KEY_END)
            || (matches!(key, KEY_LEFT | KEY_RIGHT)
                && horizontal_boundary == CursorBoundary::Line))
            .then(|| self.text_input_line_range_for_cursor(local_id))
            .flatten();
        let mut edits_text = false;
        let mut moves_cursor = false;
        {
            let Some(state) = self
                .component(local_id)
                .and_then(|component| component.concrete.text_input.as_ref())
            else {
                return false;
            };
            let mut raw = state.raw.borrow_mut();
            match key {
                KEY_Z if modifiers & (system | MOD_SHIFT) == system | MOD_SHIFT => {
                    edits_text = raw.redo();
                }
                KEY_Z if modifiers & system != 0 => edits_text = raw.undo(),
                KEY_A if modifiers & system != 0 => moves_cursor = raw.select_all(),
                KEY_HOME => {
                    moves_cursor =
                        raw.cursor_horizontal(-1, CursorBoundary::Line, select, line_range)
                }
                KEY_END => {
                    moves_cursor =
                        raw.cursor_horizontal(1, CursorBoundary::Line, select, line_range)
                }
                KEY_BACKSPACE => edits_text = raw.backspace(-1),
                KEY_DELETE => edits_text = raw.backspace(1),
                KEY_LEFT => {
                    moves_cursor =
                        raw.cursor_horizontal(-1, horizontal_boundary, select, line_range)
                }
                KEY_RIGHT => {
                    moves_cursor = raw.cursor_horizontal(1, horizontal_boundary, select, line_range)
                }
                KEY_UP | KEY_DOWN => {}
                KEY_ENTER if self.text_input_multiline(local_id) => edits_text = raw.insert("\n"),
                KEY_ENTER => return false,
                _ => return false,
            }
        }
        if matches!(key, KEY_UP | KEY_DOWN) {
            self.ensure_text_input_geometry(local_id);
            let direction = if key == KEY_UP { -1 } else { 1 };
            let current = self
                .component(local_id)
                .and_then(|component| component.concrete.text_input.as_ref())
                .map(|state| state.raw.borrow().cursor().end.codepoint_index)
                .unwrap_or(0);
            let target = self
                .component(local_id)
                .and_then(|component| component.concrete.text_input.as_ref())
                .and_then(|state| {
                    let raw = state.raw.borrow();
                    raw.geometry()?
                        .vertical_cursor(current, direction, raw.ideal_cursor_x())
                })
                .unwrap_or_else(|| {
                    if direction < 0 {
                        (0, 0.0)
                    } else {
                        (
                            self.text_input_display_text(local_id)
                                .map(|text| text.chars().count())
                                .unwrap_or(0),
                            0.0,
                        )
                    }
                });
            let Some(state) = self
                .component(local_id)
                .and_then(|component| component.concrete.text_input.as_ref())
            else {
                return false;
            };
            moves_cursor = state
                .raw
                .borrow_mut()
                .move_cursor_vertical(target.0, select, target.1);
        }
        if edits_text {
            self.sync_text_input_source_from_raw(local_id);
            crate::text_owner::mark_shape_dirty(self, local_id);
        } else if moves_cursor {
            self.add_dirt(local_id, ComponentDirt::PAINT, false);
        }
        true
    }

    pub(crate) fn text_input_text_input(&mut self, local_id: usize, text: &str) -> bool {
        if !self.ensure_text_input_initialized(local_id) {
            return true;
        }
        let insert = if self.text_input_multiline(local_id) {
            text.to_owned()
        } else {
            strip_line_breaks(text)
        };
        if insert.is_empty() {
            return true;
        }
        let changed = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
            .is_some_and(|state| state.raw.borrow_mut().insert(&insert));
        if changed {
            self.sync_text_input_source_from_raw(local_id);
            crate::text_owner::mark_shape_dirty(self, local_id);
        }
        true
    }

    pub(crate) fn text_input_cursor_geometry(
        &self,
        local_id: usize,
    ) -> Option<((f32, f32), (f32, f32))> {
        self.ensure_text_input_initialized(local_id);
        self.ensure_text_input_geometry(local_id);
        let state = self.component(local_id)?.concrete.text_input.as_ref()?;
        let raw = state.raw.borrow();
        let text = raw.text();
        let byte = char_byte_index(&text, raw.cursor().end.codepoint_index);
        let geometry = raw.geometry()?.caret(byte)?;
        Some(((geometry.0.x, geometry.0.y), (geometry.1.x, geometry.1.y)))
    }

    pub(crate) fn text_input_local_bounds_retained(
        &self,
        local_id: usize,
    ) -> Option<(f32, f32, f32, f32)> {
        self.ensure_text_input_initialized(local_id);
        self.ensure_text_input_geometry(local_id);
        self.component(local_id)?
            .concrete
            .text_input
            .as_ref()?
            .raw
            .borrow()
            .geometry()?
            .local_bounds()
    }

    pub(crate) fn adjust_text_input_scroll_to_caret(&mut self, local_id: usize) -> bool {
        let Some((constraint, multiline, cursor)) = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
            .and_then(|state| {
                (state.scroll_x == 0.0 && state.scroll_y == 0.0 && !state.is_dragging)
                    .then(|| {
                        let raw = state.raw.borrow();
                        let text = raw.text();
                        let byte = char_byte_index(&text, raw.cursor().end.codepoint_index);
                        Some((state.scroll_constraint?, raw.geometry()?.caret(byte)?))
                    })
                    .flatten()
            })
            .map(|(constraint, cursor)| (constraint, self.text_input_multiline(local_id), cursor))
        else {
            return false;
        };
        crate::constraints::scroll_text_input_caret_into_view(
            self,
            constraint,
            multiline,
            (cursor.0.x + cursor.1.x) * 0.5,
            cursor.0.y,
            cursor.1.y,
        )
    }

    pub(crate) fn text_input_selection_path(&self, local_id: usize) -> Vec<RuntimePathCommand> {
        self.ensure_text_input_initialized(local_id);
        let Some(state) = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
        else {
            return Vec::new();
        };
        self.ensure_text_input_geometry(local_id);
        state.raw.borrow_mut().selection_path_commands()
    }

    pub(crate) fn text_input_selection_range(
        &self,
        local_id: usize,
    ) -> Option<std::ops::Range<usize>> {
        self.ensure_text_input_initialized(local_id);
        let state = self.component(local_id)?.concrete.text_input.as_ref()?;
        let raw = state.raw.borrow();
        let cursor = raw.cursor();
        cursor
            .has_selection()
            .then(|| cursor.first().codepoint_index..cursor.last().codepoint_index)
    }

    pub(crate) fn text_input_separates_selection_text(&self, local_id: usize) -> bool {
        self.ensure_text_input_initialized(local_id)
            && self
                .component(local_id)
                .and_then(|component| component.concrete.text_input.as_ref())
                .is_some_and(|state| state.raw.borrow().separate_selection_text)
    }

    pub(crate) fn text_input_move_cursor_to_world(
        &mut self,
        local_id: usize,
        world: (f32, f32),
        select: bool,
    ) -> bool {
        self.text_input_move_cursor_to_world_with_auto_scroll(local_id, world, select, false)
    }

    fn text_input_move_cursor_to_world_with_auto_scroll(
        &mut self,
        local_id: usize,
        world: (f32, f32),
        select: bool,
        enable_auto_scroll: bool,
    ) -> bool {
        self.ensure_text_input_initialized(local_id);
        let text_world = {
            let Some(graph) = self.runtime_graph() else {
                return false;
            };
            self.runtime_component_world_transform(local_id, graph)
        };
        if text_world.determinant() == 0.0 {
            return false;
        }
        let local_point = text_world
            .invert_or_identity()
            .transform_point(world.0, world.1);
        let mut local = RenderVec2D::new(local_point.0, local_point.1);
        let scroll_constraint = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
            .and_then(|state| state.scroll_constraint);
        let viewport = scroll_constraint.and_then(|constraint| {
            crate::constraints::text_input_scroll_viewport(self, constraint)
        });
        let multiline = self.text_input_multiline(local_id);
        let mut scroll_x = 0.0;
        let mut scroll_y = 0.0;
        if let Some(viewport) = viewport {
            const EDGE_THRESHOLD: f32 = 20.0;
            if !multiline && viewport.constrains_horizontal {
                let viewport_x = local.x + viewport.offset_x;
                let left_distance = edge_activation_distance(viewport_x, EDGE_THRESHOLD);
                let right_distance =
                    edge_activation_distance(viewport.width - viewport_x, EDGE_THRESHOLD);
                if enable_auto_scroll && left_distance > 0.0 {
                    scroll_x = edge_scroll_speed_for_distance(left_distance);
                    if viewport_x < 0.0 {
                        local.x = -viewport.offset_x;
                    }
                } else if enable_auto_scroll && right_distance > 0.0 {
                    scroll_x = -edge_scroll_speed_for_distance(right_distance);
                    if viewport_x > viewport.width {
                        local.x = viewport.width - viewport.offset_x;
                    }
                }
            }
            if multiline && viewport.constrains_vertical {
                let viewport_y = local.y + viewport.offset_y;
                let top_distance = edge_activation_distance(viewport_y, EDGE_THRESHOLD);
                let bottom_distance =
                    edge_activation_distance(viewport.height - viewport_y, EDGE_THRESHOLD);
                if enable_auto_scroll && top_distance > 0.0 {
                    scroll_y = edge_scroll_speed_for_distance(top_distance);
                    if viewport_y < 0.0 {
                        local.y = -viewport.offset_y;
                    }
                } else if enable_auto_scroll && bottom_distance > 0.0 {
                    scroll_y = -edge_scroll_speed_for_distance(bottom_distance);
                    if viewport_y > viewport.height {
                        local.y = viewport.height - viewport.offset_y;
                    }
                }
            }
        }
        if let Some(state) = self
            .component_mut(local_id)
            .and_then(|component| component.concrete.text_input.as_mut())
        {
            state.scroll_x = scroll_x;
            state.scroll_y = scroll_y;
        }
        self.ensure_text_input_geometry(local_id);
        let Some(byte) = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
            .and_then(|state| state.raw.borrow().geometry()?.hit(local))
        else {
            return false;
        };
        let text = self.text_input_display_text(local_id).unwrap_or_default();
        let codepoint = text
            .get(..byte)
            .map(str::chars)
            .map(Iterator::count)
            .unwrap_or(0);
        let changed = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
            .is_some_and(|state| state.raw.borrow_mut().move_cursor_to(codepoint, select));
        if changed {
            self.add_dirt(local_id, ComponentDirt::PAINT, false);
        }
        changed
    }

    pub(crate) fn text_input_start_drag(&mut self, local_id: usize, world: (f32, f32)) {
        if let Some(state) = self
            .component_mut(local_id)
            .and_then(|component| component.concrete.text_input.as_mut())
        {
            state.is_dragging = true;
            state.last_drag_world_position = world;
        }
        self.text_input_move_cursor_to_world(local_id, world, false);
    }

    pub(crate) fn text_input_drag(&mut self, local_id: usize, world: (f32, f32)) {
        if let Some(state) = self
            .component_mut(local_id)
            .and_then(|component| component.concrete.text_input.as_mut())
        {
            state.last_drag_world_position = world;
        }
        self.text_input_move_cursor_to_world_with_auto_scroll(local_id, world, true, true);
    }

    pub(crate) fn text_input_end_drag(&mut self, local_id: usize) {
        if let Some(state) = self
            .component_mut(local_id)
            .and_then(|component| component.concrete.text_input.as_mut())
        {
            state.is_dragging = false;
            state.last_drag_world_position = (f32::NAN, f32::NAN);
            state.scroll_x = 0.0;
            state.scroll_y = 0.0;
        }
    }

    pub(crate) fn text_input_select_word(&mut self, local_id: usize) -> bool {
        self.ensure_text_input_initialized(local_id);
        let changed = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
            .is_some_and(|state| state.raw.borrow_mut().select_word());
        if changed {
            self.add_dirt(local_id, ComponentDirt::PAINT, false);
        }
        changed
    }

    pub(crate) fn text_input_select_line(&mut self, local_id: usize) -> bool {
        let Some(range) = self.text_input_line_range_for_cursor(local_id) else {
            return false;
        };
        let changed = self
            .component(local_id)
            .and_then(|component| component.concrete.text_input.as_ref())
            .is_some_and(|state| state.raw.borrow_mut().select_line(range));
        if changed {
            self.add_dirt(local_id, ComponentDirt::PAINT, false);
        }
        changed
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_cursor(&self, local_id: usize) -> Option<(usize, usize)> {
        self.ensure_text_input_initialized(local_id);
        let cursor = self
            .component(local_id)?
            .concrete
            .text_input
            .as_ref()?
            .raw
            .borrow()
            .cursor();
        Some((cursor.start.codepoint_index, cursor.end.codepoint_index))
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_set_text_input_cursor(&self, local_id: usize, start: usize, end: usize) -> bool {
        self.ensure_text_input_initialized(local_id)
            && self
                .component(local_id)
                .and_then(|component| component.concrete.text_input.as_ref())
                .is_some_and(|state| {
                    state.raw.borrow_mut().set_cursor(Cursor {
                        start: CursorPosition::unresolved(start),
                        end: CursorPosition::unresolved(end),
                    })
                })
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_display_text(&self, local_id: usize) -> Option<String> {
        self.text_input_display_text(local_id)
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_key_input(
        &mut self,
        local_id: usize,
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool {
        self.text_input_key_input(local_id, key, modifiers, is_pressed, is_repeat)
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_text_input(&mut self, local_id: usize, text: &str) -> bool {
        self.text_input_text_input(local_id, text)
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_select_word(&mut self, local_id: usize) -> bool {
        self.text_input_select_word(local_id)
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_select_line(&mut self, local_id: usize) -> bool {
        self.text_input_select_line(local_id)
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_selection_radius(&self, local_id: usize) -> Option<f32> {
        self.ensure_text_input_initialized(local_id);
        Some(
            self.component(local_id)?
                .concrete
                .text_input
                .as_ref()?
                .raw
                .borrow()
                .selection_corner_radius,
        )
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_world_point(
        &self,
        local_id: usize,
        local_x: f32,
        local_y: f32,
    ) -> Option<(f32, f32)> {
        let graph = self.runtime_graph()?;
        let (mut x, mut y) = self
            .runtime_component_world_transform(local_id, graph)
            .transform_point(local_x, local_y);
        if self.frame_origin() {
            x += self.origin_x * self.width;
            y += self.origin_y * self.height;
        }
        Some((x, y))
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_cursor_geometry(
        &self,
        local_id: usize,
    ) -> Option<((f32, f32), (f32, f32))> {
        self.text_input_cursor_geometry(local_id)
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_line_metrics(
        &self,
        local_id: usize,
    ) -> Option<Vec<(usize, usize, f32, f32)>> {
        self.ensure_text_input_initialized(local_id);
        self.ensure_text_input_geometry(local_id);
        Some(
            self.component(local_id)?
                .concrete
                .text_input
                .as_ref()?
                .raw
                .borrow()
                .geometry()?
                .line_metrics(),
        )
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_line_directions(&self, local_id: usize) -> Option<Vec<bool>> {
        self.ensure_text_input_initialized(local_id);
        self.ensure_text_input_geometry(local_id);
        Some(
            self.component(local_id)?
                .concrete
                .text_input
                .as_ref()?
                .raw
                .borrow()
                .geometry()?
                .line_directions(),
        )
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_move_cursor_to_local(
        &mut self,
        local_id: usize,
        local_x: f32,
        local_y: f32,
    ) -> bool {
        let Some(world) = self.debug_text_input_world_point(local_id, local_x, local_y) else {
            return false;
        };
        self.text_input_move_cursor_to_world(local_id, world, false)
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_measure(
        &self,
        local_id: usize,
        max_width: f32,
        max_height: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        let (runtime, graph) = self.runtime_file().zip(self.runtime_graph())?;
        crate::text::text_input_layout_measure_bounds(
            runtime,
            graph,
            self,
            local_id,
            crate::text::RuntimeTextLayoutConstraint {
                width: max_width,
                height: max_height,
                width_scale_type: 0,
                height_scale_type: 0,
                layout_direction: 0,
            },
        )
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_text_input_measure_count(&self, local_id: usize) -> Option<usize> {
        Some(
            self.component(local_id)?
                .concrete
                .text_input
                .as_ref()?
                .raw
                .borrow()
                .measure_count(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_single_line_break_stripping_is_ported() {
        assert_eq!(strip_line_breaks("a\nb\r\nc\rd"), "a b c d");
        assert_eq!(strip_line_breaks("\r\n"), " ");
    }

    #[test]
    fn upstream_key_and_modifier_values_remain_pinned() {
        assert_eq!(
            (KEY_LEFT, KEY_RIGHT, KEY_HOME, KEY_END),
            (263, 262, 268, 269)
        );
        assert_eq!((MOD_SHIFT, MOD_CTRL, MOD_ALT, MOD_META), (1, 2, 4, 8));
    }
}
