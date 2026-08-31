use crate::mechanical_port::source::core::CoreHandle;
use crate::mechanical_port::source::shapes::paint::{
    dash_path::DashEffectPath,
    effects_container::{self, EffectsContainer},
    shape_paint::ShapePaint,
    shape_paint_path::ShapePaintPath,
    target_effect::TargetEffectPath,
    trim_path::TrimEffectPath,
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
impl StrokeEffectState {
    pub fn add_path_provider(&mut self, provider: &PathProvider, path: Box<dyn EffectPath>) {
        self.effect_paths.insert(provider.identity(), path);
    }

    pub fn invalidate_effect(&mut self, provider: Option<&PathProvider>) {
        if let Some(provider) = provider {
            if let Some(path) = self.effect_paths.get_mut(&provider.identity()) {
                path.invalidate_effect();
            }
        } else {
            for path in self.effect_paths.values_mut() {
                path.invalidate_effect();
            }
        }
    }
}
pub trait StrokeEffect:
    crate::mechanical_port::source::generated::core_registry::CoreCapabilities
{
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
            .add_path_provider(component, path);
    }
    fn invalidate_effect(&mut self, provider: Option<&PathProvider>) {
        self.stroke_effect_state().invalidate_effect(provider);
    }
    fn effect_path(&mut self, provider: &PathProvider) -> Option<Rc<RefCell<ShapePaintPath>>> {
        self.stroke_effect_state()
            .effect_paths
            .get_mut(&provider.identity())
            .and_then(|path| path.path())
    }
    fn invalidate_effect_from_local(&mut self)
    where
        Self: Sized,
    {
        let invalidating = self.stroke_effect_handle();
        for path in self.stroke_effect_state().effect_paths.values_mut() {
            path.invalidate_effect();
        }
        if let (Some(parent), Some(invalidating)) =
            (self.parent_paint_handle(), invalidating.as_ref())
        {
            let mut active = Some(effects_container::ActiveStrokeEffect::new(
                invalidating.clone(),
                self,
            ));
            effects_container::invalidate_effects_handle_with_active(
                &parent,
                Some(invalidating.clone()),
                &mut active,
            );
        }
    }
}
