use crate::mechanical_port::source::{
    core::CoreHandle,
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::target_effect_base::TargetEffectBase,
    shapes::paint::{
        effects_container::{self, EffectsContainer},
        group_effect::GroupEffect,
        shape_paint::ShapePaint,
        shape_paint_path::ShapePaintPath,
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
    pub fn path_provider_proxy(&self) -> &PathProvider {
        &self.proxy
    }
}
impl EffectPath for TargetEffectPath {
    fn as_target_mut(&mut self) -> Option<&mut TargetEffectPath> {
        Some(self)
    }
}
impl std::ops::Deref for TargetEffect {
    type Target = TargetEffectBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TargetEffect {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TargetEffect {
    pub const TYPE_KEY: u16 = TargetEffectBase::TYPE_KEY;
}

pub struct TargetEffect {
    pub base: TargetEffectBase,
    stroke: StrokeEffectState,
    group_effect: Option<CoreHandle>,
}
impl Default for TargetEffect {
    fn default() -> Self {
        Self {
            base: TargetEffectBase::default(),
            stroke: StrokeEffectState::default(),
            group_effect: None,
        }
    }
}
impl TargetEffect {
    pub(crate) fn invalidate_effect_handle_with_active(
        handle: &CoreHandle,
        provider: Option<PathProvider>,
        active: &mut Option<effects_container::ActiveStrokeEffect<'_>>,
    ) {
        let (group, proxies) = handle
            .with_downcast_mut::<TargetEffect, _>(|target| {
                let group = target.group_effect.clone();
                let proxies = if let Some(provider) = provider {
                    target
                        .stroke
                        .effect_paths
                        .get_mut(&provider.identity())
                        .and_then(|path| path.as_target_mut())
                        .map(|path| vec![*path.path_provider_proxy()])
                        .unwrap_or_default()
                } else {
                    target
                        .stroke
                        .effect_paths
                        .values_mut()
                        .filter_map(|path| path.as_target_mut())
                        .map(|path| *path.path_provider_proxy())
                        .collect()
                };
                (group, proxies)
            })
            .unwrap_or_default();
        let Some(group) = group else {
            return;
        };
        for proxy in proxies {
            GroupEffect::invalidate_effect_handle_with_active(&group, Some(proxy), active);
        }
    }

    pub(crate) fn invalidate_effect_from_handle_with_active(
        handle: &CoreHandle,
        active: &mut Option<effects_container::ActiveStrokeEffect<'_>>,
    ) {
        let (parent, invalidating) = handle
            .with_downcast_mut::<TargetEffect, _>(|target| {
                target.stroke.invalidate_effect(None);
                (target.base.parent_handle(), target.base.handle())
            })
            .unwrap_or_default();
        if let (Some(parent), Some(invalidating)) = (parent, invalidating) {
            effects_container::invalidate_effects_handle_with_active(
                &parent,
                Some(invalidating),
                active,
            );
        }
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
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
        let Some(group) = context
            .resolve_handle(self.base.target_id())
            .filter(|group| group.with_downcast::<GroupEffect, _>(|_| ()).is_some())
        else {
            return StatusCode::MissingObject;
        };
        self.group_effect = Some(group.clone());
        group.with_downcast_mut::<GroupEffect, _>(|group| {
            group.add_target_effect(this);
            for path in self.stroke.effect_paths.values_mut() {
                group.add_path_provider(path.as_target_mut().unwrap().path_provider_proxy());
            }
        });
        StatusCode::Ok
    }
    pub fn update_effect(
        &mut self,
        provider: &PathProvider,
        source: &ShapePaintPath,
        paint: &ShapePaint,
    ) {
        let Some(group) = self.group_effect.clone() else {
            return;
        };
        if let Some(path) = self.stroke.effect_paths.get_mut(&provider.identity()) {
            group.with_downcast_mut::<GroupEffect, _>(|group| {
                group.update_effect(
                    path.as_target_mut().unwrap().path_provider_proxy(),
                    source,
                    paint,
                );
            });
        }
    }
    pub fn effect_path(
        &mut self,
        provider: &PathProvider,
    ) -> Option<std::rc::Rc<std::cell::RefCell<ShapePaintPath>>> {
        let group = self.group_effect.clone()?;
        let path = self.stroke.effect_paths.get_mut(&provider.identity())?;
        group
            .with_downcast_mut::<GroupEffect, _>(|group| {
                group.last_effect_path(path.as_target_mut().unwrap().path_provider_proxy())
            })
            .flatten()
    }
    pub fn add_path_provider(&mut self, provider: &PathProvider) {
        let path = self.create_effect_path();
        self.stroke.add_path_provider(provider, path);
        if let Some(path) = self.stroke.effect_paths.get_mut(&provider.identity()) {
            if let Some(group) = self.group_effect.as_ref() {
                group.with_downcast_mut::<GroupEffect, _>(|group| {
                    group.add_path_provider(path.as_target_mut().unwrap().path_provider_proxy());
                });
            }
        }
    }
    pub fn create_effect_path_direct(&mut self) -> Box<dyn EffectPath> {
        Box::new(TargetEffectPath::new())
    }
    pub fn invalidate_effect_direct(&mut self, provider: Option<&PathProvider>) {
        let Some(group) = self.group_effect.clone() else {
            return;
        };
        if let Some(provider) = provider {
            if let Some(path) = self.stroke.effect_paths.get_mut(&provider.identity()) {
                group.with_downcast_mut::<GroupEffect, _>(|group| {
                    group.invalidate_effect(Some(
                        path.as_target_mut().unwrap().path_provider_proxy(),
                    ));
                });
            }
        } else {
            for path in self.stroke.effect_paths.values_mut() {
                group.with_downcast_mut::<GroupEffect, _>(|group| {
                    group.invalidate_effect(Some(
                        path.as_target_mut().unwrap().path_provider_proxy(),
                    ));
                });
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
    fn stroke_effect_handle(&self) -> Option<CoreHandle> {
        self.base.handle()
    }
    fn update_effect(&mut self, p: &PathProvider, s: &ShapePaintPath, paint: &ShapePaint) {
        TargetEffect::update_effect(self, p, s, paint);
    }
    fn effect_path(
        &mut self,
        p: &PathProvider,
    ) -> Option<std::rc::Rc<std::cell::RefCell<ShapePaintPath>>> {
        TargetEffect::effect_path(self, p)
    }
    fn parent_paint_handle(&self) -> Option<CoreHandle> {
        self.base.parent_handle()
    }
    fn add_path_provider(&mut self, p: &PathProvider) {
        TargetEffect::add_path_provider(self, p);
    }
    fn create_effect_path(&mut self) -> Box<dyn EffectPath> {
        self.create_effect_path_direct()
    }
    fn invalidate_effect(&mut self, p: Option<&PathProvider>) {
        self.invalidate_effect_direct(p);
    }
}
