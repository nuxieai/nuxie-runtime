use crate::mechanical_port::source::{
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
#[derive(Default)]
pub struct ScriptedDrawable {
    pub scripted: ScriptedObject,
    pub properties: Vec<usize>,
    pub children: Vec<usize>,
    pub inverse_world: [f32; 6],
    pub hidden: bool,
    pub collapsed: bool,
    pub opacity: f32,
    is_advance_active: bool,
    needs_update: bool,
}
impl ScriptedDrawable {
    pub fn did_hydrate_script_inputs(&mut self) {
        self.wake_advance();
        self.mark_needs_update()
    }
    pub fn draw(&mut self) {
        self.scripted.script_draw_canvas()
    }
    pub fn update(&mut self) {
        self.scripted.script_update();
        self.needs_update = false
    }
    pub fn will_draw(&self) -> bool {
        !self.hidden && !self.collapsed && self.opacity > 0.0
    }
    pub fn advance_component(&mut self, e: f32, animate: bool) -> bool {
        if !animate || !self.is_advance_active {
            return false;
        }
        self.is_advance_active = self.scripted.script_advance(e);
        self.is_advance_active
    }
    pub fn add_scripted_dirt(&mut self) {
        self.mark_needs_update()
    }
    pub fn add_property(&mut self, p: usize) {
        self.properties.push(p)
    }
    pub fn mark_needs_update(&mut self) {
        self.needs_update = true
    }
    pub fn world_to_local(&self, w: Vec2) -> Option<Vec2> {
        let m = self.inverse_world;
        let det = m[0] * m[3] - m[2] * m[1];
        if det == 0.0 {
            return None;
        }
        Some(Vec2 {
            x: m[0] * w.x + m[2] * w.y + m[4],
            y: m[1] * w.x + m[3] * w.y + m[5],
        })
    }
    pub fn key_input(&mut self, k: Key, m: KeyModifiers, p: bool, r: bool) -> bool {
        let method = if p { "keyDown" } else { "keyUp" };
        let handled = self
            .scripted
            .call_number(method, &[k as u16 as f32, m.0 as f32, r as u8 as f32])
            .is_some();
        if handled {
            self.wake_advance()
        }
        handled
    }
    pub fn text_input(&mut self, text: &str) -> bool {
        self.scripted.set_string_input("text".into(), text.into());
        let handled = self.scripted.call_number("textInput", &[]).is_some();
        if handled {
            self.wake_advance()
        }
        handled
    }
    pub fn wake_advance(&mut self) {
        self.is_advance_active = true
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Node
    }
}
pub struct HitScriptedDrawable<'a> {
    pub drawable: &'a mut ScriptedDrawable,
}
impl HitScriptedDrawable<'_> {
    pub fn hit_test(&self, _p: Vec2) -> bool {
        true
    }
    pub fn process_event(&mut self, p: Vec2, event: &str, can_hit: bool) -> HitResult {
        if !can_hit {
            return HitResult::None;
        }
        let Some(local) = self.drawable.world_to_local(p) else {
            return HitResult::None;
        };
        if self
            .drawable
            .scripted
            .call_number(event, &[local.x, local.y])
            .is_some()
        {
            self.drawable.wake_advance();
            HitResult::Hit
        } else {
            HitResult::None
        }
    }
}
