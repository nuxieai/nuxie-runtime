use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::trim_path_base::TrimPathBase,
    math::{
        contour_measure::{ContourMeasure, ContourMeasureIter},
        raw_path::RawPath,
    },
    refcnt::Rcp,
    shapes::paint::{
        effects_container::{self, EffectsContainer},
        shape_paint::{ShapePaint, ShapePaintPath, ShapePaintType},
        stroke_effect::{EffectPath, PathProvider, StrokeEffect, StrokeEffectState},
    },
};
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrimPathMode {
    Sequential = 1,
    Synchronized = 2,
}
pub struct TrimEffectPath {
    path: ShapePaintPath,
    contours: Vec<Rcp<ContourMeasure>>,
}
impl TrimEffectPath {
    pub fn new() -> Self {
        Self {
            path: ShapePaintPath::new(true),
            contours: Vec::new(),
        }
    }
}
impl EffectPath for TrimEffectPath {
    fn invalidate_effect(&mut self) {
        self.path.rewind();
        self.contours.clear();
    }
    fn path(&mut self) -> Option<&mut ShapePaintPath> {
        Some(&mut self.path)
    }
    fn as_trim_mut(&mut self) -> Option<&mut TrimEffectPath> {
        Some(self)
    }
}
pub struct TrimPath {
    pub base: TrimPathBase,
    stroke: StrokeEffectState,
}
impl TrimPath {
    pub fn mode(&self) -> TrimPathMode {
        TrimPathMode::from(self.base.mode_value())
    }
    pub fn on_added_clean(&mut self, _context: &mut CoreContext) -> StatusCode {
        let Some(container) = effects_container::from(self.base.parent_mut()) else {
            return StatusCode::InvalidObject;
        };
        container.add_stroke_effect(self as *mut _ as *mut dyn StrokeEffect);
        StatusCode::Ok
    }
    fn trim_path(
        &self,
        destination: &mut ShapePaintPath,
        contours: &mut Vec<Rcp<ContourMeasure>>,
        source: &RawPath,
        paint_type: ShapePaintType,
    ) {
        let raw = destination.mutable_raw_path();
        let render_offset = ((self.base.offset() % 1.0) + 1.0) % 1.0;
        let close_shape = paint_type == ShapePaintType::Fill;
        if contours.is_empty() {
            let mut iter = ContourMeasureIter::new(source);
            while let Some(measure) = iter.next() {
                contours.push(measure);
            }
        }
        match self.mode() {
            TrimPathMode::Sequential => {
                let total: f32 = contours.iter().map(|c| c.length()).sum();
                let mut start = total * (self.base.start() + render_offset);
                let mut end = total * (self.base.end() + render_offset);
                if end < start {
                    std::mem::swap(&mut start, &mut end);
                }
                if start > total {
                    start -= total;
                    end -= total;
                }
                let mut i = 0;
                let count = contours.len() as i32;
                let mut indices = Vec::new();
                let mut lengths = Vec::new();
                while end > 0.0 {
                    let current = i % count;
                    let length = contours[current as usize].length();
                    if start < length {
                        indices.push(current as usize);
                        lengths.push(start);
                        lengths.push(end);
                        end -= length;
                        start = 0.0;
                    } else {
                        start -= length;
                        end -= length;
                    }
                    i += 1;
                }
                let mut starting = 0i32;
                let mut index_count = 0usize;
                let mut previous = None;
                while index_count < indices.len() {
                    let index = ((if starting < 0 {
                        starting + indices.len() as i32
                    } else {
                        starting
                    }) % indices.len() as i32) as usize;
                    let contour_index = indices[index];
                    let contour = &contours[contour_index];
                    let length = contour.length();
                    let start = lengths[index * 2];
                    let end = lengths[index * 2 + 1];
                    contour.get_segment(
                        start,
                        end,
                        raw,
                        previous != Some(contour_index) || !contour.is_closed(),
                    );
                    if (start == 0.0 && end - start >= length && contour.is_closed()) || close_shape
                    {
                        raw.close();
                    }
                    previous = Some(contour_index);
                    index_count += 1;
                    starting -= 1;
                }
            }
            TrimPathMode::Synchronized => {
                for contour in contours.iter() {
                    let length = contour.length();
                    let mut start = length * (self.base.start() + render_offset);
                    let mut end = length * (self.base.end() + render_offset);
                    if end < start {
                        std::mem::swap(&mut start, &mut end);
                    }
                    if start >= length {
                        start -= length;
                        end -= length;
                    }
                    contour.get_segment(start, end, raw, true);
                    while end > length {
                        start = 0.0;
                        end -= length;
                        contour.get_segment(start, end, raw, !contour.is_closed());
                    }
                    if (self.base.start() == 0.0 && self.base.end() == 1.0 && contour.is_closed())
                        || close_shape
                    {
                        raw.close();
                    }
                }
            }
        }
    }
    pub fn start_changed(&mut self) {
        StrokeEffect::invalidate_effect_from_local(self);
    }
    pub fn end_changed(&mut self) {
        StrokeEffect::invalidate_effect_from_local(self);
    }
    pub fn offset_changed(&mut self) {
        StrokeEffect::invalidate_effect_from_local(self);
    }
    pub fn mode_value_changed(&mut self) {
        StrokeEffect::invalidate_effect_from_local(self);
    }
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        match self.mode() {
            TrimPathMode::Sequential | TrimPathMode::Synchronized => StatusCode::Ok,
        }
    }
    pub fn update_effect(
        &mut self,
        provider: &mut PathProvider,
        source: &ShapePaintPath,
        paint: &ShapePaint,
    ) {
        let Some(effect) = self
            .stroke
            .effect_paths
            .get_mut(&(provider as *mut _))
            .and_then(|path| path.as_trim_mut())
        else {
            return;
        };
        if effect.path.has_render_path() {
            return;
        }
        effect.path.rewind_as(source.is_local(), source.fill_rule());
        self.trim_path(
            &mut effect.path,
            &mut effect.contours,
            source.raw_path(),
            paint.paint_type(),
        );
    }
}
impl StrokeEffect for TrimPath {
    fn stroke_effect_state(&mut self) -> &mut StrokeEffectState {
        &mut self.stroke
    }
    fn update_effect(&mut self, p: &mut PathProvider, s: &ShapePaintPath, paint: &ShapePaint) {
        TrimPath::update_effect(self, p, s, paint);
    }
    fn parent_paint(&mut self) -> Option<&mut dyn EffectsContainer> {
        effects_container::from(self.base.parent_mut())
    }
    fn create_effect_path(&mut self) -> Box<dyn EffectPath> {
        Box::new(TrimEffectPath::new())
    }
}
