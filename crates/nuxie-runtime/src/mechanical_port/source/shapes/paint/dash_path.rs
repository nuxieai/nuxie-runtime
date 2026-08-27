use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::dash_path_base::DashPathBase,
    math::{path_measure::PathMeasure, raw_path::RawPath},
    shapes::paint::{
        dash::Dash,
        effects_container::{self, EffectsContainer},
        shape_paint::{ShapePaint, ShapePaintPath, ShapePaintType},
        stroke_effect::{EffectPath, PathProvider, StrokeEffect, StrokeEffectState},
    },
};
pub struct DashEffectPath {
    path: ShapePaintPath,
    path_measure: PathMeasure,
}
impl DashEffectPath {
    pub fn new() -> Self {
        Self {
            path: ShapePaintPath::new(true),
            path_measure: PathMeasure::default(),
        }
    }
    pub fn create_path_measure(&mut self, source: &RawPath) {
        self.path_measure = PathMeasure::new(source);
    }
}
impl EffectPath for DashEffectPath {
    fn invalidate_effect(&mut self) {
        self.path.rewind();
    }
    fn path(&mut self) -> Option<&mut ShapePaintPath> {
        Some(&mut self.path)
    }
    fn as_dash_mut(&mut self) -> Option<&mut DashEffectPath> {
        Some(self)
    }
}
pub trait PathDasher {
    fn invalidate_dash(&mut self) {}
    fn dash<'a>(
        &mut self,
        destination: &'a mut ShapePaintPath,
        source: &RawPath,
        measure: &mut PathMeasure,
        offset: &Dash,
        dashes: &mut [*mut Dash],
    ) -> &'a mut ShapePaintPath {
        if destination.has_render_path() {
            return destination;
        }
        destination.rewind();
        Self::apply_dash(destination, source, measure, offset, dashes)
    }
    fn apply_dash<'a>(
        destination: &'a mut ShapePaintPath,
        _source: &RawPath,
        measure: &mut PathMeasure,
        offset: &Dash,
        dashes: &mut [*mut Dash],
    ) -> &'a mut ShapePaintPath {
        let valid = dashes
            .iter()
            .copied()
            .any(|dash| unsafe { (*dash).normalized_length(measure.length(), false) > 0.0 });
        if valid {
            let mut dash_index = 0usize;
            let raw = destination.mutable_raw_path();
            let mut dashed = 0.0;
            let mut distance = offset.normalized_length(measure.length(), true);
            let mut draw = true;
            while dashed < measure.length() {
                let dash = unsafe { &*dashes[dash_index % dashes.len()] };
                dash_index += 1;
                let mut length = dash.normalized_length(measure.length(), false);
                if length > measure.length() {
                    length = measure.length();
                }
                let mut end = distance + length;
                if end > measure.length() {
                    end -= measure.length();
                    if draw {
                        if distance < measure.length() {
                            measure.get_segment(distance, measure.length(), raw, true);
                            measure.get_segment(0.0, end, raw, !measure.is_closed());
                        } else {
                            measure.get_segment(0.0, end, raw, true);
                        }
                    }
                    distance = end - length;
                } else if draw {
                    measure.get_segment(distance, end, raw, true);
                }
                distance += length;
                dashed += length;
                draw = !draw;
            }
        }
        destination
    }
}
pub struct DashPath {
    pub base: DashPathBase,
    stroke: StrokeEffectState,
    dashes: Vec<*mut Dash>,
}
impl DashPath {
    pub fn on_added_clean(&mut self, _context: &mut CoreContext) -> StatusCode {
        let Some(container) = effects_container::from(self.base.parent_mut()) else {
            return StatusCode::InvalidObject;
        };
        container.add_stroke_effect(self as *mut _ as *mut dyn StrokeEffect);
        self.dashes.clear();
        for child in self.base.children_mut() {
            if let Some(dash) = child.as_mut::<Dash>() {
                self.dashes.push(dash);
            }
        }
        StatusCode::Ok
    }
    pub fn offset_changed(&mut self) {
        StrokeEffect::invalidate_effect_from_local(self);
    }
    pub fn offset_is_percentage_changed(&mut self) {
        StrokeEffect::invalidate_effect_from_local(self);
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
            .and_then(|path| path.as_dash_mut())
        else {
            return;
        };
        if effect.path.has_render_path() {
            return;
        }
        effect.path.rewind_local(source.is_local());
        if paint.paint_type() == ShapePaintType::Fill {
            effect.path.add_shape_paint_path(source, None);
        } else {
            let offset = Dash::with_value(
                Default::default(),
                self.base.offset(),
                self.base.offset_is_percentage(),
            );
            effect.create_path_measure(source.raw_path());
            Self::apply_dash(
                &mut effect.path,
                source.raw_path(),
                &mut effect.path_measure,
                &offset,
                &mut self.dashes,
            );
        }
    }
    pub fn invalidate_dash(&mut self) {
        StrokeEffect::invalidate_effect_from_local(self);
    }
}
impl PathDasher for DashPath {
    fn invalidate_dash(&mut self) {
        DashPath::invalidate_dash(self);
    }
}
impl StrokeEffect for DashPath {
    fn stroke_effect_state(&mut self) -> &mut StrokeEffectState {
        &mut self.stroke
    }
    fn update_effect(&mut self, p: &mut PathProvider, s: &ShapePaintPath, paint: &ShapePaint) {
        DashPath::update_effect(self, p, s, paint);
    }
    fn parent_paint(&mut self) -> Option<&mut dyn EffectsContainer> {
        effects_container::from(self.base.parent_mut())
    }
    fn create_effect_path(&mut self) -> Box<dyn EffectPath> {
        Box::new(DashEffectPath::new())
    }
}
