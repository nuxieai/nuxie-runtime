//! Shared regular-text selection, translated from text_selection_controller.hpp/.cpp.
//! CoreHandle and Text's back reference are weak. The controller owns only selection
//! state; an already-borrowed Text is projected into draw/update callbacks instead
//! of borrowing its arena slot a second time.
use std::{cell::RefCell, rc::Rc};

use super::{
    cursor::{Cursor, CursorPosition},
    glyph_lookup::GlyphLookup,
    text::Text,
    text_layout_view::TextLayoutView,
    text_selection_path::TextSelectionPath,
    utf::Utf,
};
use crate::mechanical_port::source::{
    artboard::Artboard,
    artboard_component_list::ArtboardComponentList,
    core::CoreHandle,
    file::RuntimeFileHandle,
    input::focusable::{Key, KeyModifiers},
    math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
    renderer::{RenderPaint, RenderPaintStyle, Renderer},
    selection_style::SelectionStyle,
    shapes::paint::color::color_modulate_opacity,
    text_engine::{OrderedLine, TextOverflow},
};

struct Participant {
    text: CoreHandle,
    id: u32,
    separator_before: String,
    source: Vec<u32>,
    lookup: GlyphLookup,
    cursor: Cursor,
    rects: Vec<Aabb>,
    path: TextSelectionPath,
    paint: Option<Box<RenderPaint>>,
    lookup_dirty: bool,
    geometry_dirty: bool,
    radius: f32,
    lookup_builds: usize,
}

impl Participant {
    fn layout<'a>(&'a mut self, text: &'a Text) -> TextLayoutView<'a> {
        if self.lookup_dirty {
            self.lookup
                .compute(text.styled_text.unichars(), &text.shape);
            self.lookup_dirty = false;
            self.lookup_builds += 1;
        }
        TextLayoutView::new(
            &text.shape,
            &text.lines,
            &text.ordered_lines,
            &self.lookup,
            self.source.len() as u32,
        )
    }
}

#[derive(Clone, Copy)]
struct Endpoint {
    participant: usize,
    position: CursorPosition,
}

pub(super) struct SelectionState {
    participants: Vec<Participant>,
    anchor: Option<Endpoint>,
    extent: Option<Endpoint>,
    pointer_id: i32,
    dragging: bool,
    external_ranges: bool,
    next_target_id: u32,
    style: Option<CoreHandle>,
    style_file: Option<RuntimeFileHandle>,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            participants: Vec::new(),
            anchor: None,
            extent: None,
            pointer_id: 0,
            dragging: false,
            external_ranges: false,
            next_target_id: 1,
            style: None,
            style_file: None,
        }
    }
}

/// A non-copyable owner of selection state. Registrations never retain Texts.
#[derive(Default)]
pub struct TextSelectionController {
    state: Rc<RefCell<SelectionState>>,
}

impl Drop for TextSelectionController {
    fn drop(&mut self) {
        for p in &self.state.borrow().participants {
            p.text.with_downcast_mut::<Text, _>(|text| {
                text.selection_controller = std::rc::Weak::new();
            });
        }
    }
}

fn finite(p: Vec2D) -> bool {
    p.x.is_finite() && p.y.is_finite()
}

fn shape_to_root(text: &Text) -> Mat2D {
    text.base
        .with_artboard_mut(|artboard| {
            let origin = artboard.root_transform(Vec2D::new(0.0, 0.0));
            let x = artboard.root_transform(Vec2D::new(1.0, 0.0)) - origin;
            let y = artboard.root_transform(Vec2D::new(0.0, 1.0)) - origin;
            Mat2D::new(x.x, x.y, y.x, y.y, origin.x, origin.y) * text.shape_world_transform()
        })
        .expect("registered Text has an Artboard")
}

fn line_bounds(line: &OrderedLine) -> Aabb {
    let mut x = line.glyph_line().start_x;
    let mut bounds = Aabb::for_expansion();
    for (run, index) in line {
        let font = run.font.as_ref().expect("shaped text retains its font");
        Aabb::expand_to_point(&mut bounds, Vec2D::new(x, line.y() + font.ascent(run.size)));
        x += run.advances[index as usize];
        Aabb::expand_to_point(
            &mut bounds,
            Vec2D::new(x, line.y() + font.descent(run.size)),
        );
    }
    bounds
}

fn eligible_text(text: &Text, p: &Participant) -> bool {
    !text.base.is_hidden()
        && text.base.render_opacity() > 0.0
        && !text.have_modifiers()
        && text.overflow() != TextOverflow::Ellipsis
        && !text.ordered_lines.is_empty()
        && !p.source.is_empty()
}

fn encode(source: &[u32], result: &mut String) {
    for &ch in source {
        let mut bytes = [0u8; 4];
        let count = Utf::encode(&mut bytes, ch) as usize;
        result
            .push_str(std::str::from_utf8(&bytes[..count]).expect("text contains Unicode scalars"));
    }
}

fn collect_selectable(root: &CoreHandle, result: &mut Vec<CoreHandle>) {
    let Some((objects, nested, lists)) = root.with_downcast::<Artboard, _>(|a| {
        (
            a.objects().to_vec(),
            a.nested_artboards(),
            a.artboard_component_lists(),
        )
    }) else {
        return;
    };
    for object in objects.into_iter().flatten() {
        if object
            .with_downcast::<Text, _>(|text| text.base.is_selectable())
            .unwrap_or(false)
        {
            result.push(object);
        }
    }
    for nested in nested {
        if let Some(instance) = nested
            .with(|object| object.as_nested_artboard()?.artboard_instance_default())
            .flatten()
        {
            collect_selectable(&instance.core_handle(), result);
        }
    }
    for list in lists {
        let instances = list
            .with_downcast::<ArtboardComponentList, _>(|list| {
                (0..list.artboard_count())
                    .filter_map(|i| list.artboard_instance(i as i32))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for instance in instances {
            collect_selectable(&instance.core_handle(), result);
        }
    }
}

impl TextSelectionController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<'a>(
        &mut self,
        text: impl Into<Option<&'a CoreHandle>>,
        separator_before: impl Into<String>,
    ) -> bool {
        let Some(text) = text.into() else {
            return false;
        };
        let mut state = self.state.borrow_mut();
        text.with_downcast_mut::<Text, _>(|value| {
            if value.base.artboard_handle().is_none()
                || value.selection_controller.upgrade().is_some()
                || value.have_modifiers()
                || value.overflow() == TextOverflow::Ellipsis
            {
                return false;
            }
            state.clear();
            let id = state.next_target_id;
            state.next_target_id = state.next_target_id.wrapping_add(1);
            value.selection_controller = Rc::downgrade(&self.state);
            state.participants.push(Participant {
                text: text.clone(),
                id,
                separator_before: separator_before.into(),
                source: value.styled_text.unichars().to_vec(),
                lookup: GlyphLookup::default(),
                cursor: Cursor::zero(),
                rects: Vec::new(),
                path: TextSelectionPath::default(),
                paint: None,
                lookup_dirty: true,
                geometry_dirty: true,
                radius: -1.0,
                lookup_builds: 0,
            });
            true
        })
        .unwrap_or(false)
    }

    pub fn remove(&mut self, text: &CoreHandle) {
        let mut state = self.state.borrow_mut();
        if let Some(index) = state.find(text) {
            state.clear();
            text.with_downcast_mut::<Text, _>(|text| {
                text.selection_controller = std::rc::Weak::new()
            });
            state.participants.remove(index);
        }
    }

    pub fn synchronize(&mut self, root: Option<&CoreHandle>) {
        let style_file = root
            .and_then(|root| {
                root.with_downcast::<Artboard, _>(|a| {
                    if a.is_instance() {
                        a.file().upgrade()
                    } else {
                        None
                    }
                })
            })
            .flatten();
        // Releasing the previous File can destroy registered source Texts.
        // Let their destructor unregister after the controller borrow ends.
        let previous_file = std::mem::replace(&mut self.state.borrow_mut().style_file, style_file);
        drop(previous_file);
        let mut texts = Vec::new();
        if let Some(root) = root {
            collect_selectable(root, &mut texts);
        }
        let removed: Vec<_> = self
            .state
            .borrow()
            .participants
            .iter()
            .filter(|p| !texts.contains(&p.text))
            .map(|p| p.text.clone())
            .collect();
        for text in removed {
            self.remove(&text);
        }
        for text in texts {
            let missing = self.state.borrow().find(&text).is_none();
            if missing {
                self.add(&text, "\n");
            }
        }
    }
    pub fn target_ids(&self) -> Vec<u32> {
        self.state
            .borrow()
            .participants
            .iter()
            .map(|p| p.id)
            .collect()
    }
    pub fn target(&self, id: u32) -> Option<CoreHandle> {
        self.state
            .borrow()
            .participants
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.text.clone())
    }
    pub fn is_eligible(&self, text: &CoreHandle) -> bool {
        let state = self.state.borrow();
        state.find(text).is_some_and(|i| state.eligible(i, None))
    }
    pub fn source_text(&self, text: &CoreHandle) -> String {
        let state = self.state.borrow();
        let mut result = String::new();
        if let Some(i) = state.find(text) {
            encode(&state.participants[i].source, &mut result);
        }
        result
    }
    pub fn root_bounds(&self, text: &CoreHandle) -> Aabb {
        let state = self.state.borrow();
        let mut result = Aabb::for_expansion();
        let Some(i) = state.find(text) else {
            return result;
        };
        if !state.eligible(i, None) {
            return result;
        }
        text.with_downcast::<Text, _>(|text| {
            let transform = shape_to_root(text);
            for line in &text.ordered_lines {
                let b = line_bounds(line);
                if b.is_empty_or_nan() {
                    continue;
                }
                for point in [
                    Vec2D::new(b.min_x, b.min_y),
                    Vec2D::new(b.max_x, b.min_y),
                    Vec2D::new(b.min_x, b.max_y),
                    Vec2D::new(b.max_x, b.max_y),
                ] {
                    Aabb::expand_to_point(&mut result, transform * point);
                }
            }
        });
        result
    }
    pub fn hit_test(
        &mut self,
        text: &CoreHandle,
        point: Vec2D,
        nearest: bool,
        offset: &mut u32,
        distance_squared: &mut f32,
    ) -> bool {
        let mut state = self.state.borrow_mut();
        let Some(i) = state.find(text) else {
            return false;
        };
        if !state.eligible(i, None) || !finite(point) {
            return false;
        }
        text.with_downcast_mut::<Text, _>(|text| {
            let transform = shape_to_root(text);
            let mut inverse = Mat2D::default();
            if !transform.invert(&mut inverse) {
                return false;
            }
            let local = inverse * point;
            if !finite(local) {
                return false;
            }
            if !nearest {
                let artboard_point = text.shape_world_transform() * local;
                if !text.base.hit_test_point(&artboard_point, false, true) {
                    return false;
                }
                if text.overflow() == TextOverflow::Clipped {
                    let mut inverse_world = Mat2D::default();
                    if !text.base.world_transform().invert(&mut inverse_world)
                        || !text.local_bounds().contains(inverse_world * artboard_point)
                    {
                        return false;
                    }
                }
            }
            let mut found = false;
            *distance_squared = f32::INFINITY;
            for (index, line) in text.ordered_lines.iter().enumerate() {
                let b = line_bounds(line);
                if !b.is_empty_or_nan() && (nearest || b.contains(local)) {
                    let closest = Vec2D::new(
                        local.x.min(b.max_x).max(b.min_x),
                        local.y.min(b.max_y).max(b.min_y),
                    );
                    let delta = transform * closest - point;
                    let distance = delta.x * delta.x + delta.y * delta.y;
                    if distance < *distance_squared {
                        *distance_squared = distance;
                        *offset = CursorPosition::from_line_x(
                            index as u32,
                            local.x,
                            &state.participants[i].layout(text),
                        )
                        .code_point_index();
                        found = true;
                    }
                }
            }
            found
        })
        .unwrap_or(false)
    }
    pub fn set_range(&mut self, text: &CoreHandle, mut start: u32, mut end: u32) {
        let mut state = self.state.borrow_mut();
        let Some(i) = state.find(text) else {
            return;
        };
        if !state.external_ranges {
            state.clear();
        }
        state.external_ranges = true;
        let p = &mut state.participants[i];
        start = start.min(p.source.len() as u32);
        end = end.min(p.source.len() as u32);
        if p.cursor.first().code_point_index() == start.min(end)
            && p.cursor.last().code_point_index() == start.max(end)
        {
            return;
        }
        p.cursor = Cursor::new(
            CursorPosition::unresolved(start),
            CursorPosition::unresolved(end),
        );
        p.geometry_dirty = true;
    }
    pub fn lookup_build_count(&self, text: &CoreHandle) -> usize {
        let state = self.state.borrow();
        state
            .find(text)
            .map_or(0, |i| state.participants[i].lookup_builds)
    }
    pub fn clear(&mut self) {
        self.state.borrow_mut().clear();
    }
    pub fn is_dragging(&self) -> bool {
        self.state.borrow().dragging
    }
    pub fn pointer_down(
        &mut self,
        text: Option<&CoreHandle>,
        point: Vec2D,
        pointer_id: i32,
        extend: bool,
    ) -> bool {
        let mut state = self.state.borrow_mut();
        if state.dragging {
            return false;
        }
        let endpoint = text
            .and_then(|text| state.find(text))
            .and_then(|i| state.endpoint_at(i, point));
        let Some(endpoint) = endpoint else {
            state.clear();
            return false;
        };
        if !extend || state.external_ranges || !state.valid_selection(None) {
            state.anchor = Some(endpoint);
        }
        state.external_ranges = false;
        state.extent = Some(endpoint);
        state.pointer_id = pointer_id;
        state.dragging = true;
        state.update_ranges();
        true
    }
    pub fn pointer_move(&mut self, point: Vec2D, pointer_id: i32) -> bool {
        self.state.borrow_mut().pointer_move(point, pointer_id)
    }
    pub fn pointer_up(&mut self, point: Vec2D, pointer_id: i32) -> bool {
        let mut state = self.state.borrow_mut();
        if !state.pointer_move(point, pointer_id) {
            return false;
        }
        state.dragging = false;
        true
    }
    pub fn pointer_cancel(&mut self, pointer_id: i32) -> bool {
        let mut state = self.state.borrow_mut();
        if !state.dragging || state.pointer_id != pointer_id {
            return false;
        }
        state.clear();
        true
    }
    pub fn select(
        &mut self,
        anchor: &CoreHandle,
        anchor_offset: u32,
        extent: &CoreHandle,
        extent_offset: u32,
    ) -> bool {
        self.state
            .borrow_mut()
            .select(anchor, anchor_offset, extent, extent_offset)
    }
    pub fn select_all(&mut self) {
        self.state.borrow_mut().select_all();
    }
    pub fn has_selection(&self) -> bool {
        self.state.borrow().has_selection()
    }
    pub fn selection(&self, text: &CoreHandle) -> Cursor {
        let state = self.state.borrow();
        if !state.valid_selection(None) {
            return Cursor::zero();
        }
        state
            .find(text)
            .map_or(Cursor::zero(), |i| state.participants[i].cursor)
    }
    pub fn selected_text(&self) -> String {
        self.state.borrow().selected_text()
    }
    pub fn selection_rects(&mut self, text: &CoreHandle) -> Vec<Aabb> {
        let mut state = self.state.borrow_mut();
        let Some(i) = state.find(text) else {
            return Vec::new();
        };
        text.with_downcast::<Text, _>(|text| {
            if state.update_rects(i, text) {
                state.participants[i].rects.clone()
            } else {
                Vec::new()
            }
        })
        .unwrap_or_default()
    }
    pub fn set_style(&mut self, style: Option<CoreHandle>) {
        self.state.borrow_mut().style = style;
    }
    pub fn style(&self) -> SelectionStyle {
        self.state.borrow().style()
    }
    pub fn key_input(&mut self, key: Key, modifiers: KeyModifiers, pressed: bool) -> bool {
        let mut state = self.state.borrow_mut();
        if !pressed || !state.valid_selection(None) {
            return false;
        }
        if key == Key::ESCAPE {
            state.clear();
            return true;
        }
        if key == Key::A
            && modifiers & (KeyModifiers::CTRL | KeyModifiers::META) != KeyModifiers::NONE
        {
            state.select_all();
            return true;
        }
        false
    }
}

impl SelectionState {
    fn find(&self, text: &CoreHandle) -> Option<usize> {
        self.participants.iter().position(|p| &p.text == text)
    }
    fn eligible(&self, index: usize, current: Option<&Text>) -> bool {
        let p = &self.participants[index];
        if let Some(text) = current {
            if text.base.handle().as_ref() == Some(&p.text) {
                return eligible_text(text, p);
            }
        }
        p.text
            .with_downcast::<Text, _>(|text| eligible_text(text, p))
            .unwrap_or(false)
    }
    fn valid_selection(&self, current: Option<&Text>) -> bool {
        if !self.external_ranges {
            let (Some(anchor), Some(extent)) = (self.anchor, self.extent) else {
                return false;
            };
            if !self.eligible(anchor.participant, current)
                || !self.eligible(extent.participant, current)
            {
                return false;
            }
        }
        self.participants
            .iter()
            .enumerate()
            .all(|(i, p)| !p.cursor.has_selection() || self.eligible(i, current))
    }
    fn clear(&mut self) {
        self.anchor = None;
        self.extent = None;
        self.dragging = false;
        self.external_ranges = false;
        for p in &mut self.participants {
            p.cursor = Cursor::zero();
            p.geometry_dirty = true;
        }
    }
    pub(super) fn remove_dropping_text(&mut self, text: &CoreHandle) {
        if let Some(i) = self.find(text) {
            self.clear();
            self.participants.remove(i);
        }
    }
    pub(super) fn text_updated(&mut self, text: &Text, shape_changed: bool) {
        let Some(i) = text.base.handle().and_then(|h| self.find(&h)) else {
            return;
        };
        if shape_changed {
            if self.participants[i].source != text.styled_text.unichars() {
                self.clear();
                self.participants[i].source = text.styled_text.unichars().to_vec();
            }
            self.participants[i].lookup_dirty = true;
        }
        self.participants[i].geometry_dirty = true;
        if self.anchor.is_some() && !self.valid_selection(Some(text)) {
            self.clear();
        }
    }
    fn endpoint_at(&mut self, i: usize, point: Vec2D) -> Option<Endpoint> {
        if !self.eligible(i, None) || !finite(point) {
            return None;
        }
        let handle = self.participants[i].text.clone();
        handle
            .with_downcast::<Text, _>(|text| {
                let mut inverse = Mat2D::default();
                if !shape_to_root(text).invert(&mut inverse) {
                    return None;
                }
                let local = inverse * point;
                if !finite(local) {
                    return None;
                }
                Some(Endpoint {
                    participant: i,
                    position: CursorPosition::from_translation(
                        local,
                        &self.participants[i].layout(text),
                    ),
                })
            })
            .flatten()
    }
    fn nearest_endpoint(&mut self, point: Vec2D) -> Option<Endpoint> {
        if !finite(point) {
            return None;
        }
        let mut best = f32::INFINITY;
        let mut result = None;
        for i in 0..self.participants.len() {
            let handle = self.participants[i].text.clone();
            handle.with_downcast::<Text, _>(|text| {
                let transform = shape_to_root(text);
                let mut inverse = Mat2D::default();
                if !eligible_text(text, &self.participants[i]) || !transform.invert(&mut inverse) {
                    return;
                }
                let local = inverse * point;
                if !finite(local) {
                    return;
                }
                for (line_index, line) in text.ordered_lines.iter().enumerate() {
                    let b = line_bounds(line);
                    if b.is_empty_or_nan() {
                        continue;
                    }
                    let closest = Vec2D::new(
                        local.x.min(b.max_x).max(b.min_x),
                        local.y.min(b.max_y).max(b.min_y),
                    );
                    let delta = transform * closest - point;
                    let distance = delta.x * delta.x + delta.y * delta.y;
                    if distance < best {
                        best = distance;
                        result = Some(Endpoint {
                            participant: i,
                            position: CursorPosition::from_line_x(
                                line_index as u32,
                                local.x,
                                &self.participants[i].layout(text),
                            ),
                        });
                    }
                }
            });
        }
        result
    }
    fn pointer_move(&mut self, point: Vec2D, id: i32) -> bool {
        if !self.dragging || id != self.pointer_id {
            return false;
        }
        if !self.valid_selection(None) {
            self.clear();
            return false;
        }
        if let Some(endpoint) = self.nearest_endpoint(point) {
            self.extent = Some(endpoint);
            self.update_ranges();
        }
        true
    }
    fn select(
        &mut self,
        anchor: &CoreHandle,
        anchor_offset: u32,
        extent: &CoreHandle,
        extent_offset: u32,
    ) -> bool {
        let (Some(a), Some(e)) = (self.find(anchor), self.find(extent)) else {
            return false;
        };
        if !self.eligible(a, None) || !self.eligible(e, None) {
            return false;
        }
        self.dragging = false;
        self.anchor = anchor.with_downcast::<Text, _>(|text| Endpoint {
            participant: a,
            position: CursorPosition::at_index(anchor_offset, &self.participants[a].layout(text)),
        });
        self.external_ranges = false;
        self.extent = extent.with_downcast::<Text, _>(|text| Endpoint {
            participant: e,
            position: CursorPosition::at_index(extent_offset, &self.participants[e].layout(text)),
        });
        self.update_ranges();
        true
    }
    fn select_all(&mut self) {
        let eligible: Vec<_> = (0..self.participants.len())
            .filter(|&i| self.eligible(i, None))
            .collect();
        if let (Some(&first), Some(&last)) = (eligible.first(), eligible.last()) {
            let a = self.participants[first].text.clone();
            let e = self.participants[last].text.clone();
            self.select(&a, 0, &e, self.participants[last].source.len() as u32);
        } else {
            self.clear();
        }
    }
    fn update_ranges(&mut self) {
        let (mut first, mut last) = (
            self.anchor.expect("selection anchor"),
            self.extent.expect("selection extent"),
        );
        if first.participant > last.participant
            || (first.participant == last.participant && first.position > last.position)
        {
            std::mem::swap(&mut first, &mut last);
        }
        for i in 0..self.participants.len() {
            let (mut start, mut end) = (0, 0);
            if i >= first.participant && i <= last.participant && self.eligible(i, None) {
                start = if i == first.participant {
                    first.position.code_point_index()
                } else {
                    0
                };
                end = if i == last.participant {
                    last.position.code_point_index()
                } else {
                    self.participants[i].source.len() as u32
                };
            }
            self.participants[i].cursor = Cursor::new(
                CursorPosition::unresolved(start),
                CursorPosition::unresolved(end),
            );
            self.participants[i].geometry_dirty = true;
        }
    }
    fn has_selection(&self) -> bool {
        if !self.valid_selection(None) {
            return false;
        }
        if self.external_ranges {
            return self.participants.iter().any(|p| p.cursor.has_selection());
        }
        let (a, e) = (self.anchor.unwrap(), self.extent.unwrap());
        a.participant != e.participant
            || a.position.code_point_index() != e.position.code_point_index()
    }
    fn selected_text(&self) -> String {
        let mut result = String::new();
        if !self.has_selection() {
            return result;
        }
        if self.external_ranges {
            for p in &self.participants {
                if !p.cursor.has_selection() {
                    continue;
                }
                if !result.is_empty() {
                    result.push_str(&p.separator_before);
                }
                encode(
                    &p.source[p.cursor.first().code_point_index() as usize
                        ..p.cursor.last().code_point_index() as usize],
                    &mut result,
                );
            }
            return result;
        }
        let (a, e) = (self.anchor.unwrap(), self.extent.unwrap());
        let (mut started, mut in_range, mut endpoints_seen) = (false, false, 0);
        for (i, p) in self.participants.iter().enumerate() {
            let endpoint = i == a.participant || i == e.participant;
            if endpoint && !in_range {
                in_range = true;
            }
            if in_range && self.eligible(i, None) {
                if started {
                    result.push_str(&p.separator_before);
                }
                started = true;
                encode(
                    &p.source[p.cursor.first().code_point_index() as usize
                        ..p.cursor.last().code_point_index() as usize],
                    &mut result,
                );
            }
            if endpoint {
                endpoints_seen += 1;
                if a.participant == e.participant || endpoints_seen == 2 {
                    break;
                }
            }
        }
        result
    }
    fn style(&self) -> SelectionStyle {
        let handle = self.style.clone().or_else(|| {
            self.style_file
                .as_ref()
                .and_then(|file| file.with_file(|file| file.selection_style(0)))
        });
        let mut result = SelectionStyle::default();
        if let Some(handle) = handle {
            handle.with_downcast::<SelectionStyle, _>(|style| {
                result.set_highlight_color(style.highlight_color());
                result.set_corner_radius(style.corner_radius());
            });
        }
        result
    }
    fn update_rects(&mut self, i: usize, text: &Text) -> bool {
        if !self.participants[i].cursor.has_selection() || !self.valid_selection(Some(text)) {
            return false;
        }
        let radius = self.style().corner_radius().max(0.0);
        let p = &mut self.participants[i];
        if p.geometry_dirty || p.radius != radius {
            p.rects.clear();
            let first = p.cursor.first().code_point_index();
            let last = p.cursor.last().code_point_index();
            // Build the lookup before borrowing its sibling rectangle buffer.
            p.layout(text);
            let view = TextLayoutView::new(
                &text.shape,
                &text.lines,
                &text.ordered_lines,
                &p.lookup,
                p.source.len() as u32,
            );
            if !view.ordered_lines().is_empty() {
                Cursor::new(
                    CursorPosition::new(0, first),
                    CursorPosition::new(view.ordered_lines().len() as u32 - 1, last),
                )
                .selection_rects(&mut p.rects, &view);
            }
            p.path.update(&p.rects, radius);
            p.radius = radius;
            p.geometry_dirty = false;
        }
        !p.rects.is_empty()
    }
    pub(super) fn draw(&mut self, text: &Text, renderer: &mut Renderer) {
        if self.anchor.is_some() && !self.valid_selection(Some(text)) {
            self.clear();
        }
        let Some(i) = text.base.handle().and_then(|h| self.find(&h)) else {
            return;
        };
        if !self.update_rects(i, text) {
            return;
        }
        let factory = text
            .base
            .with_artboard(|a| a.factory())
            .flatten()
            .expect("registered Text factory");
        let color = color_modulate_opacity(
            self.style().highlight_color() as u32,
            text.base.render_opacity(),
        );
        let p = &mut self.participants[i];
        if p.paint.is_none() {
            let mut paint = factory.with_factory_mut(|factory| factory.make_render_paint());
            paint.style(RenderPaintStyle::Fill);
            p.paint = Some(paint);
        }
        let paint = p.paint.as_mut().unwrap();
        paint.color(color);
        renderer.save();
        renderer.transform(nuxie_render_api::Mat2D(
            *text.shape_world_transform().values(),
        ));
        renderer.draw_path(p.path.path.render_path(&factory), paint.as_ref());
        renderer.restore();
    }
}
