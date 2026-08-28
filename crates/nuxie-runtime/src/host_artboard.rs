//! Public host adapter. All live scene state belongs to the translated owners.

use anyhow::{Context, Result, ensure};
use nuxie_render_api::{Aabb, Mat2D, Renderer};

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    component_dirt::ComponentDirt,
    core::CoreHandle,
    file::RuntimeFileHandle,
    generated::{component_base::ComponentBase, core_registry::CoreRegistry},
};
use crate::{host_animation::LinearAnimationInstance, host_state_machine::StateMachineInstance};

pub struct ArtboardInstance {
    native: RuntimeArtboardInstanceHandle,
    file: RuntimeFileHandle,
    index: usize,
}

impl std::fmt::Debug for ArtboardInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtboardInstance")
            .field("index", &self.index)
            .field("identity", &self.native.core_handle().identity_key())
            .finish()
    }
}

/// An observation of a component, not a second mutable component owner.
#[derive(Clone, Debug)]
pub struct RuntimeComponent {
    pub local_id: usize,
    pub handle: CoreHandle,
    pub name: String,
    pub dirt: ComponentDirt,
    pub collapsed: bool,
}

pub type RuntimeComponents = Vec<RuntimeComponent>;

impl ArtboardInstance {
    pub fn from_native(file: RuntimeFileHandle, index: usize) -> Result<Self> {
        let source = file
            .with_file(|file| file.artboard_at_source(index))
            .with_context(|| format!("no artboard at index {index}"))?;
        let native = Artboard::instance_from_handle(&source)
            .context("translated Artboard initialization failed")?;
        Ok(Self {
            native,
            file,
            index,
        })
    }

    pub fn from_native_handle(
        file: RuntimeFileHandle,
        index: usize,
        native: RuntimeArtboardInstanceHandle,
    ) -> Self {
        Self {
            native,
            file,
            index,
        }
    }

    pub fn native_handle(&self) -> RuntimeArtboardInstanceHandle {
        self.native.clone()
    }
    pub fn native_file(&self) -> RuntimeFileHandle {
        self.file.clone()
    }
    pub fn artboard_index(&self) -> usize {
        self.index
    }
    pub fn name(&self) -> String {
        CoreRegistry::get_string_handle(
            &self.native.core_handle(),
            i32::from(ComponentBase::NAME_PROPERTY_KEY),
        )
        .unwrap_or_default()
    }
    pub fn artboard_dimensions(&self) -> (f32, f32) {
        self.native
            .with_artboard(|artboard| (artboard.base.layout_width(), artboard.base.layout_height()))
    }
    pub fn artboard_bounds(&self) -> (f32, f32, f32, f32) {
        self.native.with_artboard(|artboard| {
            let bounds = artboard.base.bounds();
            (bounds.left, bounds.top, bounds.width(), bounds.height())
        })
    }
    pub fn bounds(&self) -> Aabb {
        let (left, top, width, height) = self.artboard_bounds();
        Aabb::new(left, top, left + width, top + height)
    }
    pub fn frame_origin(&self) -> bool {
        self.native.with_artboard(|a| a.base.frame_origin())
    }
    pub fn set_frame_origin(&mut self, value: bool) {
        self.native
            .with_artboard_mut(|a| a.base.set_frame_origin(value));
    }
    pub fn set_width(&mut self, value: f32) {
        self.native.with_artboard_mut(|a| a.base.set_width(value));
    }
    pub fn set_height(&mut self, value: f32) {
        self.native.with_artboard_mut(|a| a.base.set_height(value));
    }
    pub fn object_handle(&self, local_id: usize) -> Option<CoreHandle> {
        let id = u32::try_from(local_id).ok()?;
        self.native
            .with_artboard(|artboard| artboard.base.resolve_handle(id))
    }
    pub fn object_count(&self) -> usize {
        self.native.with_artboard(|a| a.base.objects().len())
    }
    pub fn object_index(&self, object: &CoreHandle) -> Option<usize> {
        self.native
            .with_artboard(|a| usize::try_from(a.base.object_index(object)).ok())
    }
    pub fn components(&self) -> RuntimeComponents {
        let objects = self.native.with_artboard(|a| a.base.objects().to_vec());
        objects
            .into_iter()
            .enumerate()
            .filter_map(|(local_id, object)| {
                let handle = object?;
                let (dirt, collapsed) = handle
                    .with(|object| object.as_component().map(|c| (c.dirt(), c.is_collapsed())))
                    .flatten()?;
                let name = CoreRegistry::get_string_handle(
                    &handle,
                    i32::from(ComponentBase::NAME_PROPERTY_KEY),
                )
                .unwrap_or_default();
                Some(RuntimeComponent {
                    local_id,
                    handle,
                    name,
                    dirt,
                    collapsed,
                })
            })
            .collect()
    }
    pub fn double_property(&self, id: usize, key: u16) -> Option<f32> {
        CoreRegistry::get_double_handle(&self.object_handle(id)?, i32::from(key))
    }
    pub fn uint_property(&self, id: usize, key: u16) -> Option<u32> {
        CoreRegistry::get_uint_handle(&self.object_handle(id)?, i32::from(key))
    }
    pub fn bool_property(&self, id: usize, key: u16) -> Option<bool> {
        CoreRegistry::get_bool_handle(&self.object_handle(id)?, i32::from(key))
    }
    pub fn color_property(&self, id: usize, key: u16) -> Option<u32> {
        CoreRegistry::get_color_handle(&self.object_handle(id)?, i32::from(key))
            .map(|value| value as u32)
    }
    pub fn string_property(&self, id: usize, key: u16) -> Option<String> {
        CoreRegistry::get_string_handle(&self.object_handle(id)?, i32::from(key))
    }
    pub fn set_double_property(&mut self, id: usize, key: u16, value: f32) -> bool {
        self.object_handle(id)
            .is_some_and(|object| CoreRegistry::set_double_handle(&object, i32::from(key), value))
    }
    pub fn set_uint_property(&mut self, id: usize, key: u16, value: u32) -> bool {
        self.object_handle(id)
            .is_some_and(|object| CoreRegistry::set_uint_handle(&object, i32::from(key), value))
    }
    pub fn set_bool_property(&mut self, id: usize, key: u16, value: bool) -> bool {
        self.object_handle(id)
            .is_some_and(|object| CoreRegistry::set_bool_handle(&object, i32::from(key), value))
    }
    pub fn set_color_property(&mut self, id: usize, key: u16, value: u32) -> bool {
        self.object_handle(id).is_some_and(|object| {
            CoreRegistry::set_color_handle(&object, i32::from(key), value as i32)
        })
    }
    pub fn set_string_property(&mut self, id: usize, key: u16, value: impl Into<String>) -> bool {
        self.object_handle(id).is_some_and(|object| {
            CoreRegistry::set_string_handle(&object, i32::from(key), value.into())
        })
    }
    pub fn object_world_transform(&mut self, id: usize) -> Option<Mat2D> {
        self.native.update_pass(true);
        self.object_handle(id)?
            .with(|object| {
                object
                    .as_world_transform_component()
                    .map(|transform| Mat2D(*transform.world_transform().values()))
            })
            .flatten()
    }
    pub fn animation_count(&self) -> usize {
        self.native.with_artboard(|a| a.base.animation_count())
    }
    pub fn state_machine_count(&self) -> usize {
        self.native.with_artboard(|a| a.base.state_machine_count())
    }
    pub fn animation_name_at(&self, index: usize) -> String {
        self.native
            .with_artboard(|a| a.base.animation_name_at(index))
    }
    pub fn state_machine_name_at(&self, index: usize) -> String {
        self.native
            .with_artboard(|a| a.base.state_machine_name_at(index))
    }
    pub fn default_state_machine_index(&self) -> Option<usize> {
        self.native
            .with_artboard(|a| usize::try_from(a.base.default_state_machine_index()).ok())
    }
    pub fn default_state_machine_instance(&mut self) -> Option<StateMachineInstance> {
        self.state_machine_instance(self.default_state_machine_index()?)
    }
    pub fn linear_animation_instance(&self, index: usize) -> Option<LinearAnimationInstance> {
        Some(LinearAnimationInstance::from_native(
            self.file.clone(),
            self.native.clone(),
            index,
            self.native.animation_at(index)?,
        ))
    }
    pub fn linear_animation_instance_named(&self, name: &str) -> Option<LinearAnimationInstance> {
        let index =
            (0..self.animation_count()).find(|&index| self.animation_name_at(index) == name)?;
        self.linear_animation_instance(index)
    }
    pub fn state_machine_instance(&mut self, index: usize) -> Option<StateMachineInstance> {
        Some(StateMachineInstance::from_native(
            self.file.clone(),
            self.native.clone(),
            index,
            self.native.state_machine_instance_handle(index)?,
        ))
    }
    pub fn state_machine_instance_named(&mut self, name: &str) -> Option<StateMachineInstance> {
        let index = (0..self.state_machine_count())
            .find(|&index| self.state_machine_name_at(index) == name)?;
        self.state_machine_instance(index)
    }
    pub fn advance(&mut self, seconds: f32) -> Result<bool> {
        Ok(self.native.advance_default(seconds))
    }
    pub fn update_components(&mut self) -> bool {
        self.native.update_pass(true)
    }
    pub fn advance_nested_artboards(&mut self, seconds: f32) -> bool {
        self.native
            .advance_internal(seconds, AdvanceFlags::ADVANCE_NESTED)
    }
    pub fn advance_linear_animation_instance(
        &mut self,
        animation: &mut LinearAnimationInstance,
        seconds: f32,
    ) -> bool {
        assert_eq!(
            self.native.core_handle(),
            animation.native_artboard().core_handle(),
            "animation belongs to another Artboard"
        );
        animation.advance_and_apply(seconds)
    }
    pub fn advance_state_machine_instance(
        &mut self,
        machine: &mut StateMachineInstance,
        seconds: f32,
    ) -> Result<bool> {
        ensure!(
            self.native.core_handle() == machine.native_artboard().core_handle(),
            "state machine belongs to another Artboard"
        );
        Ok(machine.advance_and_apply(seconds))
    }
    pub fn advance_state_machine_instances(
        &mut self,
        machines: &mut [StateMachineInstance],
        seconds: f32,
        advance_view_models: bool,
    ) -> Result<crate::host_state_machine::RuntimeStateMachineAdvanceResult> {
        StateMachineInstance::advance_and_apply_batch(self, machines, seconds, advance_view_models)
    }
    pub fn draw(&self, renderer: &mut dyn Renderer) {
        self.native.draw(renderer);
    }
    pub fn validate_renderer_factory(
        &self,
        factory: &mut dyn nuxie_render_api::Factory,
    ) -> Result<()> {
        let candidate = factory
            .persistent_context()
            .context("renderer factory must retain a persistent context")?;
        let current = self
            .native
            .with_artboard(|a| a.base.factory())
            .context("Artboard has no renderer factory")?;
        ensure!(
            candidate.identity() == current.persistent_context().identity(),
            "Artboard was imported with a different renderer factory"
        );
        Ok(())
    }
    pub fn audio_engine(
        &self,
    ) -> Option<crate::mechanical_port::source::audio::audio_engine::AudioEngineRef> {
        self.native.with_artboard(|a| a.base.audio_engine())
    }
    pub fn has_audio(&self) -> bool {
        self.native.with_artboard_mut(|a| a.base.has_audio())
    }
    pub fn set_audio_engine(
        &mut self,
        value: Option<crate::mechanical_port::source::audio::audio_engine::AudioEngineRef>,
    ) {
        self.native
            .with_artboard_mut(|a| a.base.set_audio_engine(value));
    }
    pub fn bind_native_view_model(&mut self, value: Option<CoreHandle>) {
        self.native.bind_view_model_instance(value);
    }
    pub fn bind_owned_view_model_handle(&mut self, value: crate::RuntimeOwnedViewModelHandle) {
        self.bind_native_view_model(Some(value.native_handle()));
    }
    pub fn volume(&self) -> f32 {
        self.native.with_artboard(|a| a.base.volume())
    }
    pub fn set_volume(&mut self, value: f32) {
        self.native.with_artboard_mut(|a| a.base.set_volume(value));
    }
}
