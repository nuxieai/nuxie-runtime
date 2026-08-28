use crate::mechanical_port::source::core::CoreHandle;
use crate::mechanical_port::source::shapes::paint::{
    dash_path::DashEffectPath, effects_container::EffectsContainer, shape_paint::ShapePaint,
    shape_paint_path::ShapePaintPath, target_effect::TargetEffectPath, trim_path::TrimEffectPath,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_PATH_PROVIDER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy)]
pub struct PathProvider {
    identity: u64,
}
impl Default for PathProvider {
    fn default() -> Self {
        Self {
            identity: NEXT_PATH_PROVIDER_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}
impl PathProvider {
    pub fn with_identity(identity: u64) -> Self {
        Self { identity }
    }

    pub fn identity(&self) -> u64 {
        self.identity
    }
}
pub trait EffectPath {
    fn invalidate_effect(&mut self) {}
    fn path(&mut self) -> Option<Rc<RefCell<ShapePaintPath>>> {
        None
    }
    fn as_target_mut(&mut self) -> Option<&mut TargetEffectPath> {
        None
    }
    fn as_dash_mut(&mut self) -> Option<&mut DashEffectPath> {
        None
    }
    fn as_trim_mut(&mut self) -> Option<&mut TrimEffectPath> {
        None
    }
}
pub struct EmptyEffectPath;
impl EffectPath for EmptyEffectPath {}
#[derive(Default)]
pub struct StrokeEffectState {
    pub effect_paths: HashMap<u64, Box<dyn EffectPath>>,
}
pub trait StrokeEffect {
    fn stroke_effect_state(&mut self) -> &mut StrokeEffectState;
    fn stroke_effect_handle(&self) -> Option<CoreHandle>;
    fn update_effect(
        &mut self,
        provider: &PathProvider,
        source: &ShapePaintPath,
        paint: &ShapePaint,
    );
    fn parent_paint_handle(&self) -> Option<CoreHandle>;
    fn create_effect_path(&mut self) -> Box<dyn EffectPath> {
        Box::new(EmptyEffectPath)
    }
    fn add_path_provider(&mut self, component: &PathProvider) {
        let path = self.create_effect_path();
        self.stroke_effect_state()
            .effect_paths
            .insert(component.identity(), path);
    }
    fn invalidate_effect(&mut self, provider: Option<&PathProvider>) {
        if let Some(provider) = provider {
            if let Some(path) = self
                .stroke_effect_state()
                .effect_paths
                .get_mut(&provider.identity())
            {
                path.invalidate_effect();
            }
        } else {
            for path in self.stroke_effect_state().effect_paths.values_mut() {
                path.invalidate_effect();
            }
        }
    }
    fn effect_path(&mut self, provider: &PathProvider) -> Option<Rc<RefCell<ShapePaintPath>>> {
        self.stroke_effect_state()
            .effect_paths
            .get_mut(&provider.identity())
            .and_then(|path| path.path())
    }
    fn invalidate_effect_from_local(&mut self) {
        let invalidating = self.stroke_effect_handle();
        for path in self.stroke_effect_state().effect_paths.values_mut() {
            path.invalidate_effect();
        }
        if let (Some(parent), Some(invalidating)) =
            (self.parent_paint_handle(), invalidating.as_ref())
        {
            parent.with_mut(|parent| {
                if let Some(parent) = parent.as_effects_container_mut() {
                    parent.invalidate_effects(Some(invalidating));
                }
            });
        }
    }
}
