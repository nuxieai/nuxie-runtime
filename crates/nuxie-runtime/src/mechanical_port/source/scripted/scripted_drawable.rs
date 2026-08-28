use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    animation::listener_invocation::{ListenerInvocation, ListenerInvocationKind},
    component_dirt::ComponentDirt,
    core::{CoreHandle, CoreObject},
    core_context::CoreContext,
    generated::{
        core_registry::CoreCapabilities, scripted::scripted_drawable_base::ScriptedDrawableBase,
    },
    hit_info::HitInfo,
    importers::import_stack::ImportStack,
    input::focusable::{Key, KeyModifiers},
    math::{mat2d::Mat2D, vec2d::Vec2D},
    renderer::Renderer,
    scripted::scripted_object::{ScriptProtocol, ScriptUpdateRequestHost, ScriptedObject},
    status_code::StatusCode,
};
use crate::scripting::{ScriptMethod, ScriptedDrawableInputResult};

pub struct ScriptedDrawable {
    pub base: ScriptedDrawableBase,
    pub scripted: ScriptedObject,
    pub properties: Vec<CoreHandle>,
    is_advance_active: bool,
}

impl std::ops::Deref for ScriptedDrawable {
    type Target = ScriptedDrawableBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ScriptedDrawable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl Default for ScriptedDrawable {
    fn default() -> Self {
        Self {
            base: ScriptedDrawableBase::default(),
            scripted: ScriptedObject::default(),
            properties: Vec::new(),
            is_advance_active: true,
        }
    }
}
impl ScriptedDrawable {
    pub fn asset_id(&self) -> u32 {
        self.base.script_asset_id()
    }

    pub fn did_hydrate_script_inputs(&mut self) {
        self.is_advance_active = true;
        self.add_scripted_dirt(ComponentDirt::PAINT, false);
    }

    pub fn draw_occurrence(owner: &CoreHandle, renderer: &mut Renderer) {
        let Some((instance, opacity, save, transform, artboard)) = owner
            .with(|owner| {
                let drawable = owner.as_scripted_drawable()?;
                if !drawable.scripted.draws() {
                    return None;
                }
                Some((
                    drawable.scripted.runtime_instance()?,
                    owner.as_transform_component()?.render_opacity(),
                    drawable.base.needs_save_operation(),
                    *owner.as_world_transform_component()?.world_transform(),
                    owner.as_component()?.artboard_handle(),
                ))
            })
            .flatten()
        else {
            return;
        };
        let factory = artboard
            .and_then(|artboard| {
                artboard
                    .with(|artboard| {
                        artboard
                            .as_artboard()
                            .and_then(|artboard| artboard.factory())
                    })
                    .flatten()
            })
            .expect("an imported scripted drawable retains its Artboard factory");
        let opacity_save = opacity != 1.0;
        if save || opacity_save {
            renderer.save();
        }
        if opacity_save {
            renderer.modulate_opacity(opacity);
        }
        renderer.transform(nuxie_render_api::Mat2D(*transform.values()));
        let mut host = ScriptUpdateRequestHost::default();
        factory.with_factory_mut(|factory| {
            let _ = instance
                .borrow_mut()
                .call_draw(factory, renderer, &mut host);
        });
        if save || opacity_save {
            renderer.restore();
        }
        if host.take_requested() {
            ScriptedObject::apply_update_request(owner);
        }
    }

    /// Called after the inherited TransformComponent update has completed.
    pub fn update_after_super_occurrence(owner: &CoreHandle, dirt: ComponentDirt) {
        if !dirt.contains(ComponentDirt::SCRIPT_UPDATE) {
            return;
        }
        let instance = owner
            .with_mut(|owner| {
                let drawable = owner.as_scripted_drawable_mut()?;
                if !drawable.scripted.updates() {
                    return None;
                }
                let instance = drawable.scripted.runtime_instance()?;
                drawable.scripted.set_in_update_phase(true);
                Some(instance)
            })
            .flatten();
        if let Some(instance) = instance {
            let mut host = ScriptUpdateRequestHost::default();
            let _ =
                instance
                    .borrow_mut()
                    .call_optional_method(ScriptMethod::Update, &[], &mut host);
            // markNeedsUpdate is ignored inside the upstream update phase.
        }
        owner.with_mut(|owner| {
            let drawable = owner
                .as_scripted_drawable_mut()
                .expect("scripted update owner");
            drawable.scripted.set_in_update_phase(false);
            drawable.is_advance_active = true;
        });
    }

    pub fn will_draw(&self) -> bool {
        self.base.base.will_draw()
            && self.scripted.runtime_instance().is_some()
            && self.scripted.draws()
    }

    pub fn advance_occurrence(
        owner: &CoreHandle,
        elapsed_seconds: f32,
        flags: AdvanceFlags,
    ) -> bool {
        if elapsed_seconds == 0.0 {
            return false;
        }
        let instance = owner
            .with_mut(|owner| {
                if owner.as_component()?.is_collapsed() {
                    return None;
                }
                let drawable = owner.as_scripted_drawable_mut()?;
                if !drawable.is_advance_active {
                    return None;
                }
                drawable.is_advance_active = false;
                if !drawable.scripted.advances() {
                    return None;
                }
                drawable.scripted.runtime_instance()
            })
            .flatten();
        let Some(instance) = instance else {
            return false;
        };
        let elapsed = if flags.0 & AdvanceFlags::ADVANCE_NESTED.0 == 0 {
            0.0
        } else {
            elapsed_seconds
        };
        let mut host = ScriptUpdateRequestHost::default();
        let advanced = instance
            .borrow_mut()
            .call_advance_truthy(elapsed, &mut host)
            .unwrap_or(false);
        if host.take_requested() {
            ScriptedObject::apply_update_request(owner);
        }
        if advanced {
            owner.with_mut(|owner| {
                owner
                    .as_scripted_drawable_mut()
                    .expect("scripted advance owner")
                    .wake_advance()
            });
        }
        advanced
    }

    pub fn add_scripted_dirt(&mut self, value: ComponentDirt, recurse: bool) -> bool {
        CoreCapabilities::component_add_dirt(self, value, recurse)
    }

    pub fn add_property(&mut self, property: CoreHandle) {
        let owner = CoreObject::core(self).handle();
        property.with_mut(|property| property.script_input_set_scripted_object(owner));
        self.properties.push(property);
    }
    pub fn remove_property(&mut self, property: &CoreHandle) {
        self.properties.retain(|item| item != property)
    }
    pub fn mark_needs_update(&mut self) {
        if self.scripted.in_update_phase() {
            return;
        }
        self.add_scripted_dirt(ComponentDirt::SCRIPT_UPDATE, false);
    }

    pub fn world_to_local(&self, world: Vec2D) -> Option<Vec2D> {
        let world_transform =
            CoreCapabilities::as_world_transform_component(self)?.world_transform();
        let mut inverse = Mat2D::default();
        world_transform
            .invert(&mut inverse)
            .then(|| inverse * world)
    }

    fn dispatch_input_occurrence(
        owner: &CoreHandle,
        invocation: &ListenerInvocation,
    ) -> ScriptedDrawableInputResult {
        let instance = owner
            .with(|owner| {
                let scripted = &owner.as_scripted_drawable()?.scripted;
                match invocation.kind() {
                    ListenerInvocationKind::Keyboard if !scripted.wants_keyboard_input() => {
                        return None;
                    }
                    ListenerInvocationKind::TextInput if !scripted.wants_text_input() => {
                        return None;
                    }
                    _ => {}
                }
                scripted.runtime_instance()
            })
            .flatten();
        let Some(instance) = instance else {
            return ScriptedDrawableInputResult::default();
        };
        let mut host = ScriptUpdateRequestHost::default();
        let result = instance
            .borrow_mut()
            .call_scripted_drawable_input(&invocation.to_script_invocation(), &mut host)
            .unwrap_or_default();
        if host.take_requested() {
            ScriptedObject::apply_update_request(owner);
        }
        if result.invoked {
            owner.with_mut(|owner| {
                owner
                    .as_scripted_drawable_mut()
                    .expect("scripted input owner")
                    .wake_advance()
            });
        }
        result
    }

    pub fn key_input_occurrence(
        owner: &CoreHandle,
        key: Key,
        modifiers: KeyModifiers,
        pressed: bool,
        repeat: bool,
    ) -> bool {
        Self::dispatch_input_occurrence(
            owner,
            &ListenerInvocation::keyboard(key.raw(), modifiers.bits(), pressed, repeat),
        )
        .handled
    }

    pub fn text_input_occurrence(owner: &CoreHandle, text: &str) -> bool {
        Self::dispatch_input_occurrence(owner, &ListenerInvocation::text_input(text.to_owned()))
            .handled
    }

    pub fn gamepad_dispatch_occurrence(
        owner: &CoreHandle,
        invocation: &ListenerInvocation,
    ) -> bool {
        match invocation.kind() {
            ListenerInvocationKind::GamepadConnected
            | ListenerInvocationKind::GamepadEvent
            | ListenerInvocationKind::GamepadDisconnected => {}
            _ => return false,
        }
        Self::dispatch_input_occurrence(owner, invocation).invoked
    }

    pub fn wake_advance(&mut self) {
        self.is_advance_active = true;
        self.add_scripted_dirt(ComponentDirt::PAINT, false);
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Node
    }

    pub fn hit_test(&mut self, _info: &mut HitInfo, _transform: &Mat2D) -> Option<CoreHandle> {
        None
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let status = self.base.base.on_added_dirty(context);
        if status != StatusCode::Ok {
            return status;
        }
        let owner = CoreObject::core(self)
            .handle()
            .expect("attached scripted drawable");
        let artboard = CoreCapabilities::as_component(self)
            .and_then(|component| component.artboard_handle())
            .expect("scripted drawable artboard");
        artboard.with_mut(|artboard| {
            artboard
                .as_artboard_mut()
                .expect("scripted drawable artboard")
                .add_scripted_object(owner)
        });
        StatusCode::Ok
    }

    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(owner) = CoreObject::core(self).handle() else {
            return StatusCode::MissingObject;
        };
        let result = self.scripted.register_referencer(owner, stack);
        if result != StatusCode::Ok {
            return result;
        }
        CoreCapabilities::as_component_mut(self)
            .expect("scripted drawable component")
            .import(stack)
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
}
pub struct HitScriptedDrawable {
    drawable: CoreHandle,
}
impl HitScriptedDrawable {
    pub fn new(drawable: CoreHandle) -> Self {
        Self { drawable }
    }
}
impl crate::mechanical_port::source::animation::state_machine_instance::HitComponent
    for HitScriptedDrawable
{
    fn component(&self) -> crate::mechanical_port::source::drawable::RuntimeDrawableOccurrence {
        crate::mechanical_port::source::drawable::RuntimeDrawableOccurrence::Authored(
            self.drawable.clone(),
        )
    }
    fn hit_test(&self, _position: crate::mechanical_port::source::math::vec2d::Vec2D) -> bool {
        true
    }
    fn prepare_event(
        &self,
        _position: crate::mechanical_port::source::math::vec2d::Vec2D,
        _hit_type: crate::mechanical_port::source::listener_type::ListenerType,
        _pointer_id: i32,
    ) {
    }
    fn process_event(
        &self,
        machine: &mut crate::mechanical_port::source::animation::state_machine_instance::StateMachineInstance,
        position: crate::mechanical_port::source::math::vec2d::Vec2D,
        hit_type: crate::mechanical_port::source::listener_type::ListenerType,
        can_hit: bool,
        _timestamp: f32,
        pointer_id: i32,
    ) -> crate::mechanical_port::source::hit_result::HitResult {
        machine.perform_scripted_pointer(&self.drawable, hit_type, can_hit, position, pointer_id)
    }
    fn process_gamepad_invocation(
        &self,
        _invocation: &ListenerInvocation,
        _already_dispatched: Option<&CoreHandle>,
    ) -> crate::mechanical_port::source::hit_result::HitResult {
        crate::mechanical_port::source::hit_result::HitResult::None
    }
}
