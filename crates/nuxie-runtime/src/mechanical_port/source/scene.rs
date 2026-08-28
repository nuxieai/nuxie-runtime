use crate::mechanical_port::source::{
    animation::{keyed_callback_reporter::KeyedCallbackReporter, loop_::Loop},
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::field_types::core_callback_type::{CallbackContext, CallbackData},
    generated::core_registry::CoreRegistry,
    hit_result::HitResult,
    math::{aabb::Aabb, vec2d::Vec2D},
    renderer::Renderer,
    state_machine::{SmiBool, SmiInput, SmiNumber, SmiTrigger},
    viewmodel::runtime::viewmodel_instance_runtime::RuntimeViewModelInstanceHandle,
};

#[derive(Clone)]
pub struct Scene {
    artboard_instance: RuntimeArtboardInstanceWeakHandle,
}

impl Scene {
    pub fn new(artboard_instance: RuntimeArtboardInstanceWeakHandle) -> Self {
        assert!(
            artboard_instance
                .with_artboard(|artboard| artboard.is_instance())
                .unwrap_or(false)
        );
        Self { artboard_instance }
    }

    pub fn artboard_instance(&self) -> RuntimeArtboardInstanceWeakHandle {
        self.artboard_instance.clone()
    }

    pub fn width(&self) -> f32 {
        self.artboard_instance
            .with_artboard(|artboard| artboard.width())
            .expect("Scene outlived its ArtboardInstance")
    }

    pub fn height(&self) -> f32 {
        self.artboard_instance
            .with_artboard(|artboard| artboard.height())
            .expect("Scene outlived its ArtboardInstance")
    }

    pub fn bounds(&self) -> Aabb {
        Aabb::new(0.0, 0.0, self.width(), self.height())
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        self.artboard_instance
            .with_artboard_mut(|artboard| artboard.draw(renderer))
            .expect("Scene outlived its ArtboardInstance");
    }

    pub fn report_keyed_callback(
        &mut self,
        object_id: u32,
        property_key: u32,
        elapsed_seconds: f32,
    ) {
        let core_object = self
            .artboard_instance
            .with_artboard(|artboard| artboard.resolve_handle(object_id))
            .flatten();
        if let Some(core_object) = core_object {
            let data = CallbackData::new(Some(self), elapsed_seconds);
            CoreRegistry::set_callback_handle(&core_object, property_key as i32, data);
        }
    }
}

impl CallbackContext for Scene {}

impl KeyedCallbackReporter for Scene {
    fn report_keyed_callback(&mut self, object_id: u32, property_key: u32, elapsed_seconds: f32) {
        Scene::report_keyed_callback(self, object_id, property_key, elapsed_seconds);
    }
}

pub trait SceneBehavior: KeyedCallbackReporter + CallbackContext {
    fn scene(&self) -> &Scene;
    fn scene_mut(&mut self) -> &mut Scene;
    fn name(&self) -> String;
    fn loop_(&self) -> Loop;
    fn is_translucent(&self) -> bool;
    fn duration_seconds(&self) -> f32;
    fn advance_and_apply(&mut self, elapsed_seconds: f32) -> bool;

    fn bind_view_model_instance(&mut self, _view_model_instance: RuntimeViewModelInstanceHandle) {}

    fn pointer_down(&mut self, _position: Vec2D, _pointer_id: i32) -> HitResult {
        HitResult::None
    }

    fn pointer_move(&mut self, _position: Vec2D, _time_stamp: f32, _pointer_id: i32) -> HitResult {
        HitResult::None
    }

    fn pointer_up(&mut self, _position: Vec2D, _pointer_id: i32) -> HitResult {
        HitResult::None
    }

    fn pointer_exit(&mut self, _position: Vec2D, _pointer_id: i32) -> HitResult {
        HitResult::None
    }

    fn input_count(&self) -> usize {
        0
    }

    fn input(&self, _index: usize) -> Option<&SmiInput> {
        None
    }

    fn get_bool(&self, _name: &str) -> Option<&SmiBool> {
        None
    }

    fn get_number(&self, _name: &str) -> Option<&SmiNumber> {
        None
    }

    fn get_trigger(&self, _name: &str) -> Option<&SmiTrigger> {
        None
    }
}
