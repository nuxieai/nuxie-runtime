use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    component_dirt::{ComponentDirt, has_dirt},
    core::CoreHandle,
    core_context::CoreContext,
    data_bind::data_context::RuntimeDataContextHandle,
    generated::scripted::scripted_path_effect_base::ScriptedPathEffectBase,
    importers::import_stack::ImportStack,
    renderer::{from_render_raw_path, to_render_raw_path},
    scripted::scripted_object::{ScriptProtocol, ScriptUpdateRequestHost, ScriptedObject},
    shapes::paint::{
        shape_paint::ShapePaint,
        shape_paint_path::ShapePaintPath,
        stroke_effect::{EffectPath, PathProvider, StrokeEffect, StrokeEffectState},
    },
    status_code::StatusCode,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
pub struct ScriptedEffectPath {
    path: Rc<RefCell<ShapePaintPath>>,
}
impl EffectPath for ScriptedEffectPath {
    fn invalidate_effect(&mut self) {
        self.path.borrow_mut().rewind();
    }
    fn path(&mut self) -> Option<Rc<RefCell<ShapePaintPath>>> {
        Some(self.path.clone())
    }
}

pub struct ScriptedPathEffect {
    pub base: ScriptedPathEffectBase,
    pub scripted: ScriptedObject,
    pub properties: Vec<CoreHandle>,
    effects: StrokeEffectState,
    advance_active: bool,
}
impl Default for ScriptedPathEffect {
    fn default() -> Self {
        Self {
            base: ScriptedPathEffectBase::default(),
            scripted: ScriptedObject::default(),
            properties: Vec::new(),
            effects: StrokeEffectState::default(),
            advance_active: true,
        }
    }
}
impl ScriptedPathEffect {
    pub fn asset_id(&self) -> u32 {
        self.base.script_asset_id()
    }
    pub fn did_hydrate_script_inputs(&mut self) {
        self.advance_active = true;
        self.add_scripted_dirt(ComponentDirt::PAINT, true);
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let this = self.base.handle().expect("attached scripted path effect");
        self.base
            .with_artboard_mut(|artboard| artboard.add_scripted_object(this));
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        let Some(parent) = self.base.parent_handle() else {
            return StatusCode::InvalidObject;
        };
        let this = self.base.handle().expect("attached scripted path effect");
        parent
            .with_mut(|parent| {
                let Some(parent) = parent.as_effects_container_mut() else {
                    return StatusCode::InvalidObject;
                };
                parent.add_stroke_effect(this);
                StatusCode::Ok
            })
            .unwrap_or(StatusCode::InvalidObject)
    }
    pub fn advance_component(&mut self, mut elapsed: f32, flags: AdvanceFlags) -> bool {
        if elapsed == 0.0 || !self.advance_active {
            return false;
        }
        self.advance_active = false;
        if flags.0 & AdvanceFlags::ADVANCE_NESTED.0 == 0 {
            elapsed = 0.0;
        }
        let advanced = self.scripted.script_advance(elapsed);
        if advanced {
            self.advance_active = true;
        }
        advanced
    }
    pub fn add_scripted_dirt(&mut self, value: ComponentDirt, recurse: bool) -> bool {
        self.base.add_dirt(value, recurse)
    }
    pub fn add_property(&mut self, property: CoreHandle) {
        let this = self.base.handle();
        property.with_mut(|property| {
            property.script_input_set_scripted_object(this);
        });
        if !self.properties.contains(&property) {
            self.properties.push(property);
        }
    }
    pub fn remove_property(&mut self, property: &CoreHandle) {
        if let Some(index) = self.properties.iter().position(|item| item == property) {
            self.properties.remove(index);
        }
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        let code = self.scripted.register_referencer(this, stack);
        if code != StatusCode::Ok {
            return code;
        }
        self.base.import(stack)
    }
    pub fn clone_definition(&self) -> Self {
        let mut clone = Self::default();
        let mut base = std::mem::take(&mut clone.base);
        base.copy(&self.base, &mut clone);
        clone.base = base;
        clone
            .scripted
            .file_asset_referencer_mut()
            .set_asset_unattached(self.scripted.script_asset());
        clone
    }
    pub fn mark_needs_update(&mut self) {
        if !self.scripted.in_update_phase() {
            self.add_scripted_dirt(ComponentDirt::SCRIPT_UPDATE, false);
        }
    }
    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) {
            parent.with_mut(|parent| {
                if let Some(parent) = parent.as_component_mut() {
                    parent.add_dependent(this);
                }
            });
        }
    }
    pub fn update(&mut self, value: ComponentDirt) {
        self.base.update(value);
        if has_dirt(value, ComponentDirt::SCRIPT_UPDATE) {
            self.invalidate_effect_from_local();
            self.advance_active = true;
        }
    }
    pub fn data_context(&self) -> Option<RuntimeDataContextHandle> {
        self.base
            .with_artboard(|artboard| artboard.data_context())
            .flatten()
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::PathEffect
    }
}
impl StrokeEffect for ScriptedPathEffect {
    fn stroke_effect_state(&mut self) -> &mut StrokeEffectState {
        &mut self.effects
    }
    fn stroke_effect_handle(&self) -> Option<CoreHandle> {
        self.base.handle()
    }
    fn parent_paint_handle(&self) -> Option<CoreHandle> {
        self.base.parent_handle().filter(|parent| {
            parent
                .with_mut(|parent| parent.as_effects_container_mut().is_some())
                .unwrap_or(false)
        })
    }
    fn create_effect_path(&mut self) -> Box<dyn EffectPath> {
        Box::new(ScriptedEffectPath::default())
    }
    fn update_effect(
        &mut self,
        provider: &PathProvider,
        source: &ShapePaintPath,
        paint: &ShapePaint,
    ) {
        if !self.scripted.updates() {
            return;
        }
        let Some(path) = self.effect_path(provider) else {
            return;
        };
        if path.borrow().has_render_path() {
            return;
        }
        path.borrow_mut()
            .rewind_as(source.is_local(), source.fill_rule());
        let Some(instance) = self.scripted.runtime_instance() else {
            return;
        };
        self.scripted.set_in_update_phase(true);
        let node = crate::scripting::ScriptNode::from_path_effect(paint);
        let mut host = ScriptUpdateRequestHost::default();
        match instance.borrow_mut().call_path_effect_update(
            to_render_raw_path(source.raw_path()),
            node,
            &mut host,
        ) {
            Ok(output) => {
                // mutableRawPath()->addPath does not prune or flatten the result.
                path.borrow_mut()
                    .mutable_raw_path()
                    .add_path(&from_render_raw_path(&output), None);
            }
            Err(_) => eprintln!("update function failed"),
        }
        if host.take_requested() {
            // Callback is still in update phase: upstream suppresses this request.
            self.mark_needs_update();
        }
        self.scripted.set_in_update_phase(false);
    }
}
impl std::ops::Deref for ScriptedPathEffect {
    type Target = ScriptedPathEffectBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ScriptedPathEffect {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
