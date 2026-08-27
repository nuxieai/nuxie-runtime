use std::cell::{Cell, RefCell};

use nuxie_render_api::{Aabb, Vec2D};
use nuxie_schema::definition_by_name;

use crate::rectangles_to_contour::RuntimeRectanglesToContour;
use crate::{
    ArtboardInstance, HitTestArea, HitTestCommandPath, Mat2D, properties::property_key_for_name,
    text_owner,
};

/// Retained fields owned by one pinned C++ `TextValueRun` occurrence.
///
/// Object pointers use occurrence-local ids because this state is embedded in
/// the same component arena as its Text and TextStylePaint owners. A clone
/// starts cold and the added lifecycle reconstructs those links.
#[derive(Debug)]
pub(crate) struct RuntimeTextValueRunState {
    rectangles_to_contour: RefCell<Option<RuntimeRectanglesToContour>>,
    local_bounds: Cell<Aabb>,
    is_hit_target: Cell<bool>,
    glyph_hit_rects: RefCell<Vec<Aabb>>,
    style_local: Cell<Option<usize>>,
    length: Cell<Option<u32>>,
    text_component_local: Cell<Option<usize>>,
}

impl Default for RuntimeTextValueRunState {
    fn default() -> Self {
        Self {
            rectangles_to_contour: RefCell::new(None),
            local_bounds: Cell::new(Aabb::default()),
            is_hit_target: Cell::new(false),
            glyph_hit_rects: RefCell::new(Vec::new()),
            style_local: Cell::new(None),
            length: Cell::new(None),
            text_component_local: Cell::new(None),
        }
    }
}

impl Clone for RuntimeTextValueRunState {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl RuntimeTextValueRunState {
    pub(crate) fn reset_added_state(&self) {
        self.style_local.set(None);
        self.text_component_local.set(None);
    }

    pub(crate) fn style_local(&self) -> Option<usize> {
        self.style_local.get()
    }

    pub(crate) fn set_style_local(&self, style_local: usize) {
        self.style_local.set(Some(style_local));
    }

    pub(crate) fn text_component_local(&self, parent_local: Option<usize>) -> Option<usize> {
        self.text_component_local.get().or(parent_local)
    }

    pub(crate) fn set_text_component_local(&self, text_local: usize) {
        self.text_component_local.set(Some(text_local));
    }

    pub(crate) fn invalidate_length(&self) {
        self.length.set(None);
    }

    /// `TextValueRun::length`: count UTF-8 code points before the first NUL
    /// and retain the result until `textChanged` invalidates it.
    pub(crate) fn length(&self, text: &[u8]) -> Option<u32> {
        if let Some(length) = self.length.get() {
            return Some(length);
        }
        let end = text
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(text.len());
        let length = u32::try_from(std::str::from_utf8(&text[..end]).ok()?.chars().count()).ok()?;
        self.length.set(Some(length));
        Some(length)
    }

    pub(crate) fn is_hit_target(&self) -> bool {
        self.is_hit_target.get()
    }

    pub(crate) fn set_is_hit_target(&self, value: bool) {
        self.is_hit_target.set(value);
    }

    pub(crate) fn reset_hit_test(&self) {
        self.glyph_hit_rects.borrow_mut().clear();
        self.local_bounds.set(Aabb::for_expansion());
    }

    pub(crate) fn add_hit_rect(&self, rect: Aabb) {
        let mut bounds = self.local_bounds.get();
        bounds.expand_to(rect.min());
        bounds.expand_to(rect.max());
        self.local_bounds.set(bounds);
        self.glyph_hit_rects.borrow_mut().push(rect);
    }

    pub(crate) fn compute_hit_contours(&self) {
        let mut converter = self.rectangles_to_contour.borrow_mut();
        let converter = converter.get_or_insert_with(RuntimeRectanglesToContour::default);
        converter.reset();
        for rect in self.glyph_hit_rects.borrow().iter().copied() {
            converter.add_rect(rect);
        }
        converter.compute_contours();
    }

    pub(crate) fn can_hit_test(&self, text_component_local: Option<usize>) -> bool {
        self.is_hit_target()
            && text_component_local.is_some()
            && !self.local_bounds.get().is_empty_or_nan()
    }

    pub(crate) fn hit_test_aabb(
        &self,
        position: Vec2D,
        text_component_local: Option<usize>,
        overflow_visible: bool,
        text_world: Mat2D,
        text_local_bounds: Aabb,
        text_local_transform: Mat2D,
    ) -> bool {
        if !self.can_hit_test(text_component_local) {
            return false;
        }
        if !overflow_visible {
            let Some(inverse_world) = text_world.invert() else {
                return false;
            };
            let local = inverse_world * (position.x, position.y);
            if !text_local_bounds.contains(Vec2D::new(local.0, local.1)) {
                return false;
            }
        }
        let Some(inverse_world) = (text_world * text_local_transform).invert() else {
            return false;
        };
        let local = inverse_world * (position.x, position.y);
        self.local_bounds
            .get()
            .contains(Vec2D::new(local.0, local.1))
    }

    pub(crate) fn hit_test_hifi(
        &self,
        position: Vec2D,
        hit_radius: f32,
        text_component_local: Option<usize>,
        text_world: Mat2D,
        text_local_transform: Mat2D,
    ) -> bool {
        if !self.can_hit_test(text_component_local) {
            return false;
        }
        let mut tester =
            HitTestCommandPath::new(HitTestArea::around(position.x, position.y, hit_radius));
        tester.set_transform(text_world * text_local_transform);
        let contours = self.rectangles_to_contour.borrow();
        let Some(contours) = contours.as_ref() else {
            return false;
        };
        for index in 0..contours.contour_count() {
            let contour = contours.contour(index);
            let mut points = contour.points();
            let Some(first) = points.next() else {
                continue;
            };
            tester.move_to(first.x, first.y);
            for point in points {
                tester.line_to(point.x, point.y);
            }
            tester.close();
        }
        tester.was_hit()
    }
}

pub(crate) fn length(instance: &ArtboardInstance, local_id: usize) -> Option<u32> {
    let text_key = property_key_for_name("TextValueRun", "text")?;
    let text = instance.string_property(local_id, text_key)?;
    instance
        .component(local_id)?
        .concrete
        .text_value_run
        .as_ref()?
        .length(text)
}

/// Direct `TextValueRun::offset`: sum the cached code-point lengths of every
/// preceding run in the owning Text's retained `m_runs` order.
pub(crate) fn offset(instance: &ArtboardInstance, local_id: usize) -> Option<u32> {
    let text_local = instance.component_parent_local(local_id)?;
    let runs = instance
        .component(text_local)?
        .concrete
        .text
        .as_ref()?
        .run_locals();
    let mut offset = 0u32;
    for run_local in runs {
        if run_local == local_id {
            break;
        }
        offset = offset.wrapping_add(length(instance, run_local)?);
    }
    Some(offset)
}

/// Direct `TextValueRun::textChanged`: invalidate the retained code-point
/// count before publishing Text shape dirt.
pub(crate) fn string_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name.is_some_and(|type_name| type_is_a(type_name, "TextValueRun"))
        && property_key_for_name("TextValueRun", "text") == Some(property_key))
    .then(|| {
        let Some(state) = instance
            .component(local_id)
            .and_then(|run| run.concrete.text_value_run.as_ref())
        else {
            return false;
        };
        state.invalidate_length();
        let Some(text_local) =
            state.text_component_local(instance.component_parent_local(local_id))
        else {
            return false;
        };
        if instance
            .component(text_local)
            .is_none_or(|component| !type_is_a(component.type_name, "Text"))
        {
            return false;
        }
        text_owner::mark_shape_dirty(instance, text_local)
    })
}

/// Direct `TextValueRun::styleIdChanged`: invalid targets leave the retained
/// style untouched; a resolved TextStylePaint replaces it and then dirties the
/// owning Text shape.
pub(crate) fn uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name.is_some_and(|type_name| type_is_a(type_name, "TextValueRun"))
        && property_key_for_name("TextValueRun", "styleId") == Some(property_key))
    .then(|| {
        let Some(style_local) = instance
            .uint_property(local_id, property_key)
            .and_then(|value| usize::try_from(value).ok())
        else {
            return false;
        };
        if instance
            .component(style_local)
            .is_none_or(|component| !type_is_a(component.type_name, "TextStylePaint"))
        {
            return false;
        }
        let Some(run_state) = instance
            .component(local_id)
            .and_then(|run| run.concrete.text_value_run.as_ref())
        else {
            return false;
        };
        run_state.set_style_local(style_local);
        let Some(text_local) =
            run_state.text_component_local(instance.component_parent_local(local_id))
        else {
            return false;
        };
        if instance
            .component(text_local)
            .is_none_or(|component| !type_is_a(component.type_name, "Text"))
        {
            return false;
        }
        text_owner::mark_shape_dirty(instance, text_local)
    })
}

fn type_is_a(type_name: &str, base: &str) -> bool {
    type_name == base
        || definition_by_name(type_name).is_some_and(|definition| definition.is_a(base))
}
