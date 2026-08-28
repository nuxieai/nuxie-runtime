use crate::mechanical_port::source::{
    animation::listener_invocation::{ListenerInvocation, ListenerInvocationKind},
    core::CoreHandle,
    generated::scripted::scripted_drawable_base::ScriptedDrawableBase,
    input::focusable::{Key, KeyModifiers},
    scripted::scripted_object::{ScriptProtocol, ScriptedObject},
};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HitResult {
    #[default]
    None,
    Hit,
    HitOpaque,
}
pub struct ScriptedDrawable {
    pub base: ScriptedDrawableBase,
    pub scripted: ScriptedObject,
    pub properties: Vec<CoreHandle>,
    pub children: Vec<CoreHandle>,
    pub inverse_world: [f32; 6],
    pub hidden: bool,
    pub collapsed: bool,
    pub opacity: f32,
    is_advance_active: bool,
    needs_update: bool,
    paint_dirty: bool,
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
            children: Vec::new(),
            inverse_world: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            hidden: false,
            collapsed: false,
            opacity: 1.0,
            is_advance_active: true,
            needs_update: false,
            paint_dirty: false,
        }
    }
}
impl ScriptedDrawable {
    pub fn asset_id(&self) -> u32 {
        self.base.script_asset_id()
    }

    pub fn did_hydrate_script_inputs(&mut self) {
        self.is_advance_active = true;
        self.paint_dirty = true;
    }
    pub fn draw(&mut self) {
        if self.scripted.draws() && self.scripted.self_ref() != 0 {
            self.scripted.script_draw()
        }
    }
    pub fn update(&mut self) {
        self.scripted.script_update();
        self.is_advance_active = true;
        self.needs_update = false
    }
    pub fn will_draw(&self) -> bool {
        !self.hidden
            && !self.collapsed
            && self.opacity > 0.0
            && self.scripted.self_ref() != 0
            && self.scripted.draws()
    }
    pub fn advance_component(&mut self, mut e: f32, advance_nested: bool) -> bool {
        if e == 0.0 || !self.is_advance_active || self.collapsed {
            return false;
        }
        self.is_advance_active = false;
        if !advance_nested {
            e = 0.0;
        }
        let advanced = self.scripted.script_advance(e);
        if advanced {
            self.is_advance_active = true;
            self.paint_dirty = true;
        }
        advanced
    }
    pub fn add_scripted_dirt(&mut self) {
        self.mark_needs_update()
    }
    pub fn add_property(&mut self, p: CoreHandle) {
        self.properties.push(p)
    }
    pub fn remove_property(&mut self, property: &CoreHandle) {
        self.properties.retain(|item| item != property)
    }
    pub fn mark_needs_update(&mut self) {
        if self.scripted.in_update_phase() {
            return;
        }
        self.needs_update = true
    }
    pub fn world_to_local(&self, w: Vec2) -> Option<Vec2> {
        let m = self.inverse_world;
        let det = m[0] * m[3] - m[2] * m[1];
        if det == 0.0 {
            return None;
        }
        let inverse_det = 1.0 / det;
        Some(Vec2 {
            x: (m[3] * w.x - m[2] * w.y + m[2] * m[5] - m[3] * m[4]) * inverse_det,
            y: (-m[1] * w.x + m[0] * w.y + m[1] * m[4] - m[0] * m[5]) * inverse_det,
        })
    }
    pub fn key_input(&mut self, k: Key, m: KeyModifiers, p: bool, r: bool) -> bool {
        if !self.scripted.wants_keyboard_input() {
            return false;
        }
        let handled = self.scripted.call_boolean(
            "keyboardEvent",
            &[k as u16 as f32, m.0 as f32, p as u8 as f32, r as u8 as f32],
        );
        if handled.is_some() {
            self.wake_advance();
        }
        handled.unwrap_or(false)
    }
    pub fn text_input(&mut self, text: &str) -> bool {
        if !self.scripted.wants_text_input() {
            return false;
        }
        let handled = self.scripted.call_boolean_with_string("textEvent", text);
        if handled.is_some() {
            self.wake_advance();
        }
        handled.unwrap_or(false)
    }
    pub fn gamepad_dispatch(&mut self, invocation: &ListenerInvocation) -> bool {
        let method = match invocation.kind() {
            ListenerInvocationKind::GamepadConnected => "gamepadConnected",
            ListenerInvocationKind::GamepadEvent => "gamepadEvent",
            ListenerInvocationKind::GamepadDisconnected => "gamepadDisconnected",
            _ => return false,
        };
        if !self.scripted.call_gamepad(method, invocation) {
            return false;
        }
        self.wake_advance();
        true
    }
    pub fn wake_advance(&mut self) {
        self.is_advance_active = true;
        self.paint_dirty = true
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Node
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
