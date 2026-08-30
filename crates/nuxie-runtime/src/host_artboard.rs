//! Public host adapter. All live scene state belongs to the translated owners.

use std::collections::HashSet;

use anyhow::{Context, Result, ensure};
use nuxie_render_api::{Aabb, Mat2D, Renderer};

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    component::ComponentOccurrenceHandle,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    file::RuntimeFileHandle,
    generated::{component_base::ComponentBase, core_registry::CoreRegistry},
    scripted::scripted_data_converter::ScriptedDataConverter,
    text::text_value_run::TextValueRun,
};
use crate::{
    host_animation::{LinearAnimationInstance, RuntimeLinearAnimationAdvanceResult},
    host_state_machine::StateMachineInstance,
    scripting::{ScriptError, ScriptValue},
};

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
    /// Index of the source ViewModel declared by this Artboard, or `None` for
    /// the upstream `UINT32_MAX` sentinel.
    pub fn view_model_index(&self) -> Option<usize> {
        let index = self
            .native
            .with_artboard(|artboard| artboard.base.view_model_id());
        (index != u32::MAX).then(|| index as usize)
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
            (bounds.left(), bounds.top(), bounds.width(), bounds.height())
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
        let height = self.native.with_artboard(|artboard| artboard.height());
        self.set_artboard_dimensions(value, height);
    }
    pub fn set_height(&mut self, value: f32) {
        let width = self.native.with_artboard(|artboard| artboard.width());
        self.set_artboard_dimensions(width, value);
    }
    pub fn set_artboard_dimensions(&mut self, width: f32, height: f32) -> bool {
        let unchanged = self
            .native
            .with_artboard(|artboard| artboard.width() == width && artboard.height() == height);
        if unchanged {
            return false;
        }
        self.native.set_size(width, height);
        true
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
    /// Return the authored root text run's current text. Nested paths remain
    /// the responsibility of the translated Artboard APIs; this projection is
    /// intentionally the exact root-name lookup used by the C host surface.
    pub fn root_text_value_run_text(&self, name: &str) -> Option<String> {
        let run = self
            .native
            .with_artboard(|artboard| artboard.base.find_handle::<TextValueRun>(name))?;
        run.with_downcast::<TextValueRun, _>(|run| run.base.text().to_owned())
    }
    /// Set one authored root text run through the translated TextValueRun
    /// mutation path. `None` means no root run has the exact authored name.
    pub fn set_root_text_value_run(&mut self, name: &str, value: String) -> Option<bool> {
        let run = self
            .native
            .with_artboard(|artboard| artboard.base.find_handle::<TextValueRun>(name))?;
        run.with_downcast_mut::<TextValueRun, _>(|run| {
            let changed = run.base.text() != value;
            if changed {
                run.set_bound_text(value);
            }
            changed
        })
    }

    /// Set one primitive input on every live occurrence of an authored
    /// scripted object in this exact root, including nested artboards and
    /// component-list rows.
    ///
    /// `None` means no initialized occurrence of the exact source Artboard and
    /// local object is currently retained. `Some(false)` means all retained
    /// occurrences already expose the requested backend value. The source
    /// Artboard index and its local object id are the same two coordinates the
    /// upstream runtime uses; binary-global ids are never compared with Rust
    /// arena slots.
    pub fn set_script_input_for_source_occurrences_if_changed(
        &mut self,
        source_artboard_index: usize,
        source_local_id: usize,
        name: &str,
        value: ScriptValue,
    ) -> std::result::Result<Option<bool>, ScriptError> {
        let Some(source_artboard) = self
            .file
            .with_file(|file| file.artboard_at_source(source_artboard_index))
        else {
            return Ok(None);
        };
        let mut visited = HashSet::new();
        set_script_input_in_occurrence_tree(
            &self.native,
            &source_artboard,
            source_local_id,
            name,
            &value,
            &mut visited,
        )
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
    ) -> Result<RuntimeLinearAnimationAdvanceResult> {
        ensure!(
            self.native.core_handle() == animation.native_artboard().core_handle(),
            "animation belongs to another Artboard"
        );
        Ok(animation.advance_and_apply_with_observed_events(seconds))
    }
    pub fn apply_linear_animation_instance_at(
        &mut self,
        animation: &mut LinearAnimationInstance,
        time: f32,
        mix: f32,
    ) -> Result<bool> {
        ensure!(
            self.native.core_handle() == animation.native_artboard().core_handle(),
            "animation belongs to another Artboard"
        );
        Ok(animation.apply_at_and_settle(time, mix))
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

fn set_script_input_in_occurrence_tree(
    artboard: &RuntimeArtboardInstanceHandle,
    source_artboard: &CoreHandle,
    source_local_id: usize,
    name: &str,
    value: &ScriptValue,
    visited: &mut HashSet<(usize, usize, u64)>,
) -> std::result::Result<Option<bool>, ScriptError> {
    if !visited.insert(artboard.core_handle().identity_key()) {
        return Ok(None);
    }
    let (objects, nested_hosts, component_lists, source) = artboard.with_artboard(|artboard| {
        (
            artboard.base.objects().to_vec(),
            artboard.base.nested_artboards(),
            artboard.base.artboard_component_lists(),
            artboard.base.artboard_source_handle(),
        )
    });
    let mut found = false;
    let mut changed = false;
    let matches_source_artboard = source.as_ref() == Some(source_artboard);
    if matches_source_artboard
        && let Some(object) = objects.get(source_local_id).and_then(Option::as_ref)
    {
        let instance = object
            .with(|object| {
                object
                    .as_scripted_object()
                    .and_then(|scripted| scripted.runtime_instance())
            })
            .flatten();
        if let Some(instance) = instance {
            found = true;
            let current = instance.borrow_mut().get_input(name)?;
            if !script_input_values_equivalent(&current, value) {
                instance.borrow_mut().set_input(name, value.clone())?;
                mark_script_input_dirt(object);
                changed = true;
            }
        }
    }

    let nested_instances = nested_hosts
        .into_iter()
        .filter_map(|host| {
            host.with(|host| host.nested_artboard_instance_handle())
                .flatten()
        })
        .chain(component_lists.into_iter().flat_map(|list| {
            list.with(|list| {
                let host = list.as_artboard_host()?;
                Some(
                    (0..host.artboard_count())
                        .filter_map(|index| host.artboard_instance(index as i32))
                        .collect::<Vec<_>>(),
                )
            })
            .flatten()
            .unwrap_or_default()
        }));
    for nested in nested_instances {
        if let Some(nested_changed) = set_script_input_in_occurrence_tree(
            &nested,
            source_artboard,
            source_local_id,
            name,
            value,
            visited,
        )? {
            found = true;
            changed |= nested_changed;
        }
    }
    Ok(found.then_some(changed))
}

fn script_input_values_equivalent(current: &ScriptValue, requested: &ScriptValue) -> bool {
    current == requested
        || matches!(
            (current, requested),
            (ScriptValue::Number(current), ScriptValue::Color(requested))
                if *current == f64::from(*requested)
        )
        || matches!(
            (current, requested),
            (ScriptValue::CoreString(current), ScriptValue::String(requested))
                if current.as_bytes() == requested.as_bytes()
        )
}

fn mark_script_input_dirt(owner: &CoreHandle) {
    let is_component = owner
        .with(|owner| owner.as_component().is_some())
        .unwrap_or(false);
    if is_component {
        ComponentOccurrenceHandle::Authored(owner.clone())
            .add_dirt(ComponentDirt::SCRIPT_UPDATE, false);
    } else {
        owner.with_downcast_mut::<ScriptedDataConverter, _>(|converter| {
            converter.add_scripted_dirt(u32::from(ComponentDirt::SCRIPT_UPDATE.0), false);
        });
    }
}
