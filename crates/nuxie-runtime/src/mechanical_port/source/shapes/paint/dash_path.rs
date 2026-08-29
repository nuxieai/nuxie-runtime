use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    core::CoreHandle,
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::dash_path_base::DashPathBase,
    math::{path_measure::PathMeasure, raw_path::RawPath},
    shapes::paint::{
        dash::Dash,
        effects_container::{self, EffectsContainer},
        shape_paint::{ShapePaint, ShapePaintType},
        shape_paint_path::ShapePaintPath,
        stroke_effect::{EffectPath, PathProvider, StrokeEffect, StrokeEffectState},
    },
};
pub struct DashEffectPath {
    path: Rc<RefCell<ShapePaintPath>>,
    path_measure: PathMeasure,
}
impl DashEffectPath {
    pub fn new() -> Self {
        Self {
            path: Rc::new(RefCell::new(ShapePaintPath::new(true))),
            path_measure: PathMeasure::default(),
        }
    }
    pub fn create_path_measure(&mut self, source: &RawPath) {
        self.path_measure = PathMeasure::from_path_default(source);
    }
}
impl EffectPath for DashEffectPath {
    fn invalidate_effect(&mut self) {
        self.path.borrow_mut().rewind();
    }
    fn path(&mut self) -> Option<Rc<RefCell<ShapePaintPath>>> {
        Some(self.path.clone())
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
        dashes: &[CoreHandle],
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
        dashes: &[CoreHandle],
    ) -> &'a mut ShapePaintPath {
        let valid = dashes.iter().any(|dash| {
            dash.with_downcast::<Dash, _>(|dash| {
                dash.normalized_length(measure.length(), false) > 0.0
            })
            .unwrap_or(false)
        });
        if valid {
            let mut dash_index = 0usize;
            let raw = destination.mutable_raw_path();
            let mut dashed = 0.0;
            let mut distance = offset.normalized_length(measure.length(), true);
            let mut draw = true;
            while dashed < measure.length() {
                let dash = &dashes[dash_index % dashes.len()];
                dash_index += 1;
                let mut length = dash
                    .with_downcast::<Dash, _>(|dash| {
                        dash.normalized_length(measure.length(), false)
                    })
                    .unwrap_or_default();
                if length > measure.length() {
                    length = measure.length();
                }
                let mut end = distance + length;
                if end > measure.length() {
                    end -= measure.length();
                    if draw {
                        if distance < measure.length() {
                            measure.get_segment(distance, measure.length(), Some(&mut *raw), true);
                            measure.get_segment(0.0, end, Some(&mut *raw), !measure.is_closed());
                        } else {
                            measure.get_segment(0.0, end, Some(&mut *raw), true);
                        }
                    }
                    distance = end - length;
                } else if draw {
                    measure.get_segment(distance, end, Some(&mut *raw), true);
                }
                distance += length;
                dashed += length;
                draw = !draw;
            }
        }
        destination
    }
}
impl std::ops::Deref for DashPath {
    type Target = DashPathBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DashPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl DashPath {
    pub const TYPE_KEY: u16 = DashPathBase::TYPE_KEY;
}

pub struct DashPath {
    pub base: DashPathBase,
    stroke: StrokeEffectState,
    dashes: Vec<CoreHandle>,
}

impl Default for DashPath {
    fn default() -> Self {
        Self {
            base: DashPathBase::default(),
            stroke: StrokeEffectState::default(),
            dashes: Vec::new(),
        }
    }
}
impl DashPath {
    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
            return StatusCode::InvalidObject;
        };
        let added = parent
            .with_mut(|parent| {
                parent
                    .as_effects_container_mut()
                    .map(|container| container.add_stroke_effect(this.clone(), self))
            })
            .flatten()
            .is_some();
        if !added {
            return StatusCode::InvalidObject;
        }
        self.dashes.clear();
        for child in self.base.children() {
            if child
                .with(|child| child.as_dash().is_some())
                .unwrap_or(false)
            {
                self.dashes.push(child.clone());
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
        provider: &PathProvider,
        source: &ShapePaintPath,
        paint: &ShapePaint,
    ) {
        let Some(effect) = self
            .stroke
            .effect_paths
            .get_mut(&provider.identity())
            .and_then(|path| path.as_dash_mut())
        else {
            return;
        };
        if effect.path.borrow().has_render_path() {
            return;
        }
        effect.path.borrow_mut().rewind_local(source.is_local());
        if paint.paint_type() == ShapePaintType::Fill {
            effect.path.borrow_mut().add_shape_paint_path(source, None);
        } else {
            let offset = Dash::with_value(
                Default::default(),
                self.base.offset(),
                self.base.offset_is_percentage(),
            );
            effect.create_path_measure(source.raw_path());
            Self::apply_dash(
                &mut effect.path.borrow_mut(),
                source.raw_path(),
                &mut effect.path_measure,
                &offset,
                &self.dashes,
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
    fn stroke_effect_handle(&self) -> Option<CoreHandle> {
        self.base.handle()
    }
    fn update_effect(&mut self, p: &PathProvider, s: &ShapePaintPath, paint: &ShapePaint) {
        DashPath::update_effect(self, p, s, paint);
    }
    fn parent_paint_handle(&self) -> Option<CoreHandle> {
        self.base.parent_handle()
    }
    fn create_effect_path(&mut self) -> Box<dyn EffectPath> {
        Box::new(DashEffectPath::new())
    }
}
