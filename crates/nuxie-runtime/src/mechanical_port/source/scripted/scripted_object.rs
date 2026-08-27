use std::{collections::HashMap, rc::Rc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptProtocol {
    Node,
    Converter,
    Interpolator,
    Layout,
    PathEffect,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptValue {
    Artboard(usize),
    Boolean(bool),
    Integer(i32),
    Number(f32),
    String(String),
    ViewModel(usize),
    Trigger,
}
pub trait ScriptRuntime {
    fn initialize(&mut self, asset_id: u32, protocol: ScriptProtocol) -> Option<(i32, i32)>;
    fn set_input(&mut self, self_ref: i32, name: &str, value: &ScriptValue);
    fn advance(&mut self, self_ref: i32, elapsed: f32) -> bool;
    fn draw(&mut self, self_ref: i32);
    fn update(&mut self, self_ref: i32);
    fn dispose(&mut self, self_ref: i32, context_ref: i32);
    fn call_number(&mut self, self_ref: i32, method: &str, args: &[f32]) -> Option<f32>;
}
pub struct ScriptedObject {
    self_ref: i32,
    context_ref: i32,
    runtime: Option<Box<dyn ScriptRuntime>>,
    asset_id: u32,
    asset: Option<Rc<[u8]>>,
    inputs: HashMap<String, ScriptValue>,
    tracked_properties: Vec<usize>,
    data_context: Option<usize>,
    in_update_phase: bool,
    user_init_done: bool,
    disposed: bool,
}
impl Default for ScriptedObject {
    fn default() -> Self {
        Self {
            self_ref: 0,
            context_ref: 0,
            runtime: None,
            asset_id: 0,
            asset: None,
            inputs: HashMap::new(),
            tracked_properties: Vec::new(),
            data_context: None,
            in_update_phase: false,
            user_init_done: false,
            disposed: false,
        }
    }
}
impl ScriptedObject {
    pub fn set_runtime(&mut self, r: Box<dyn ScriptRuntime>) {
        self.runtime = Some(r)
    }
    pub fn ensure_script_initialized(&mut self, protocol: ScriptProtocol) -> bool {
        if self.self_ref != 0 {
            return true;
        }
        let Some(runtime) = &mut self.runtime else {
            return false;
        };
        if let Some((s, c)) = runtime.initialize(self.asset_id, protocol) {
            self.self_ref = s;
            self.context_ref = c;
            true
        } else {
            false
        }
    }
    pub fn hydrate_script_inputs(&mut self) -> bool {
        let Some(runtime) = &mut self.runtime else {
            return true;
        };
        if self.self_ref == 0 {
            return false;
        }
        for (n, v) in &self.inputs {
            runtime.set_input(self.self_ref, n, v)
        }
        self.user_init_done = true;
        true
    }
    fn set(&mut self, n: String, v: ScriptValue) {
        self.inputs.insert(n.clone(), v.clone());
        if let Some(r) = &mut self.runtime {
            if self.self_ref != 0 {
                r.set_input(self.self_ref, &n, &v)
            }
        }
    }
    pub fn set_artboard_input(&mut self, n: String, v: usize) {
        self.set(n, ScriptValue::Artboard(v))
    }
    pub fn set_boolean_input(&mut self, n: String, v: bool) {
        self.set(n, ScriptValue::Boolean(v))
    }
    pub fn set_integer_input(&mut self, n: String, v: i32) {
        self.set(n, ScriptValue::Integer(v))
    }
    pub fn set_number_input(&mut self, n: String, v: f32) {
        self.set(n, ScriptValue::Number(v))
    }
    pub fn set_string_input(&mut self, n: String, v: String) {
        self.set(n, ScriptValue::String(v))
    }
    pub fn set_view_model_input(&mut self, n: String, v: usize) {
        self.set(n, ScriptValue::ViewModel(v))
    }
    pub fn trigger(&mut self, n: String) {
        self.set(n, ScriptValue::Trigger)
    }
    pub fn script_advance(&mut self, e: f32) -> bool {
        self.runtime
            .as_mut()
            .is_some_and(|r| r.advance(self.self_ref, e))
    }
    pub fn script_draw_canvas(&mut self) {
        if let Some(r) = &mut self.runtime {
            r.draw(self.self_ref)
        }
    }
    pub fn script_update(&mut self) {
        self.in_update_phase = true;
        if let Some(r) = &mut self.runtime {
            r.update(self.self_ref)
        }
        self.in_update_phase = false
    }
    pub fn script_dispose(&mut self) {
        if self.disposed {
            return;
        }
        if let Some(r) = &mut self.runtime {
            r.dispose(self.self_ref, self.context_ref)
        }
        self.self_ref = 0;
        self.context_ref = 0;
        self.inputs.clear();
        self.tracked_properties.clear();
        self.disposed = true
    }
    pub fn reinit(&mut self) {
        self.script_dispose();
        self.disposed = false;
        self.user_init_done = false
    }
    pub fn set_asset(&mut self, id: u32, a: Option<Rc<[u8]>>) {
        self.script_dispose();
        self.asset_id = id;
        self.asset = a;
        self.disposed = false
    }
    pub fn add_tracked_property(&mut self, p: usize) {
        if p != 0 {
            self.tracked_properties.push(p)
        }
    }
    pub fn remove_tracked_property(&mut self, p: usize) {
        self.tracked_properties.retain(|v| *v != p)
    }
    pub fn tracked_properties(&self) -> &[usize] {
        &self.tracked_properties
    }
    pub fn self_ref(&self) -> i32 {
        self.self_ref
    }
    pub fn data_context(&self) -> Option<usize> {
        self.data_context
    }
    pub fn set_data_context(&mut self, v: Option<usize>) {
        self.data_context = v
    }
    pub fn in_update_phase(&self) -> bool {
        self.in_update_phase
    }
    pub fn user_lua_init_done(&self) -> bool {
        self.user_init_done
    }
    pub fn call_number(&mut self, m: &str, args: &[f32]) -> Option<f32> {
        self.runtime.as_mut()?.call_number(self.self_ref, m, args)
    }
}
impl Drop for ScriptedObject {
    fn drop(&mut self) {
        self.script_dispose()
    }
}
