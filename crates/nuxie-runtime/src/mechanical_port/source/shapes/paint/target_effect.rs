use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::target_effect_base::TargetEffectBase,
    shapes::paint::{
        effects_container::{self, EffectsContainer},
        group_effect::GroupEffect,
        shape_paint::{ShapePaint, ShapePaintPath},
        stroke_effect::{EffectPath, PathProvider, StrokeEffect, StrokeEffectState},
    },
};
pub struct TargetEffectPath {
    proxy: PathProvider,
}
impl TargetEffectPath {
    pub fn new() -> Self {
        Self {
            proxy: PathProvider::default(),
        }
    }
    pub fn path_provider_proxy(&mut self) -> &mut PathProvider {
        &mut self.proxy
    }
}
impl EffectPath for TargetEffectPath {
    fn as_target_mut(&mut self) -> Option<&mut TargetEffectPath> {
        Some(self)
    }
}
pub struct TargetEffect {
    pub base: TargetEffectBase,
    stroke: StrokeEffectState,
    group_effect: Option<*mut GroupEffect>,
}
impl TargetEffect {
    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        let Some(container) = effects_container::from(self.base.parent_mut()) else {
            return StatusCode::InvalidObject;
        };
        container.add_stroke_effect(self as *mut _ as *mut dyn StrokeEffect);
        let Some(group) = context
            .resolve_mut(self.base.target_id())
            .and_then(|object| object.as_mut::<GroupEffect>())
        else {
            return StatusCode::MissingObject;
        };
        self.group_effect = Some(group);
        group.add_target_effect(self);
        for path in self.stroke.effect_paths.values_mut() {
            group.add_path_provider(path.as_target_mut().unwrap().path_provider_proxy());
        }
        StatusCode::Ok
    }
    pub fn update_effect(
        &mut self,
        provider: &mut PathProvider,
        source: &ShapePaintPath,
        paint: &ShapePaint,
    ) {
        let Some(group) = self.group_effect else {
            return;
        };
        if let Some(path) = self.stroke.effect_paths.get_mut(&(provider as *mut _)) {
            unsafe {
                (*group).update_effect(
                    path.as_target_mut().unwrap().path_provider_proxy(),
                    source,
                    paint,
                );
            }
        }
    }
    pub fn effect_path(&mut self, provider: &mut PathProvider) -> Option<&mut ShapePaintPath> {
        let group = self.group_effect?;
        let path = self.stroke.effect_paths.get_mut(&(provider as *mut _))?;
        unsafe { (*group).last_effect_path(path.as_target_mut().unwrap().path_provider_proxy()) }
    }
    pub fn add_path_provider(&mut self, provider: &mut PathProvider) {
        StrokeEffect::add_path_provider(self, provider);
        if let Some(path) = self.stroke.effect_paths.get_mut(&(provider as *mut _)) {
            if let Some(group) = self.group_effect {
                unsafe {
                    (*group).add_path_provider(path.as_target_mut().unwrap().path_provider_proxy());
                }
            }
        }
    }
    pub fn parent_paint_direct(&mut self) -> Option<&mut dyn EffectsContainer> {
        effects_container::from(self.base.parent_mut())
    }
    pub fn create_effect_path_direct(&mut self) -> Box<dyn EffectPath> {
        Box::new(TargetEffectPath::new())
    }
    pub fn invalidate_effect_direct(&mut self, provider: Option<&mut PathProvider>) {
        let Some(group) = self.group_effect else {
            return;
        };
        if let Some(provider) = provider {
            if let Some(path) = self.stroke.effect_paths.get_mut(&(provider as *mut _)) {
                unsafe {
                    (*group).invalidate_effect(Some(
                        path.as_target_mut().unwrap().path_provider_proxy(),
                    ));
                }
            }
        } else {
            for path in self.stroke.effect_paths.values_mut() {
                unsafe {
                    (*group).invalidate_effect(Some(
                        path.as_target_mut().unwrap().path_provider_proxy(),
                    ));
                }
            }
        }
    }
    pub fn invalidate_effect_from_local(&mut self) {
        StrokeEffect::invalidate_effect_from_local(self);
    }
}
impl StrokeEffect for TargetEffect {
    fn stroke_effect_state(&mut self) -> &mut StrokeEffectState {
        &mut self.stroke
    }
    fn update_effect(&mut self, p: &mut PathProvider, s: &ShapePaintPath, paint: &ShapePaint) {
        TargetEffect::update_effect(self, p, s, paint);
    }
    fn effect_path(&mut self, p: &mut PathProvider) -> Option<&mut ShapePaintPath> {
        TargetEffect::effect_path(self, p)
    }
    fn parent_paint(&mut self) -> Option<&mut dyn EffectsContainer> {
        self.parent_paint_direct()
    }
    fn add_path_provider(&mut self, p: &mut PathProvider) {
        TargetEffect::add_path_provider(self, p);
    }
    fn create_effect_path(&mut self) -> Box<dyn EffectPath> {
        self.create_effect_path_direct()
    }
    fn invalidate_effect(&mut self, p: Option<&mut PathProvider>) {
        self.invalidate_effect_direct(p);
    }
}
