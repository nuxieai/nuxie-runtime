use crate::mechanical_port::source::{
    animation::{keyed_callback_reporter::KeyedCallbackReporter, loop_::Loop},
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::field_types::core_callback_type::CallbackContext,
    scene::{Scene, SceneBehavior},
};

pub struct StaticScene {
    pub scene: Scene,
}

impl StaticScene {
    pub fn new(artboard_instance: RuntimeArtboardInstanceWeakHandle) -> Self {
        Self {
            scene: Scene::new(artboard_instance),
        }
    }

    pub fn is_translucent(&self) -> bool {
        self.scene
            .artboard_instance()
            .with_artboard(|artboard| artboard.is_translucent())
            .expect("StaticScene outlived its ArtboardInstance")
    }

    pub fn name(&self) -> String {
        self.scene
            .artboard_instance()
            .with_artboard(|artboard| artboard.name().to_owned())
            .expect("StaticScene outlived its ArtboardInstance")
    }

    pub fn loop_(&self) -> Loop {
        Loop::OneShot
    }

    pub fn duration_seconds(&self) -> f32 {
        0.0
    }

    pub fn advance_and_apply(&mut self, _seconds: f32) -> bool {
        self.scene
            .artboard_instance()
            .with_artboard_mut(|artboard| artboard.advance(0.0))
            .expect("StaticScene outlived its ArtboardInstance");
        true
    }
}

impl CallbackContext for StaticScene {}

impl KeyedCallbackReporter for StaticScene {
    fn report_keyed_callback(&mut self, object_id: u32, property_key: u32, elapsed_seconds: f32) {
        self.scene
            .report_keyed_callback(object_id, property_key, elapsed_seconds);
    }
}

impl SceneBehavior for StaticScene {
    fn scene(&self) -> &Scene {
        &self.scene
    }

    fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    fn name(&self) -> String {
        StaticScene::name(self)
    }

    fn loop_(&self) -> Loop {
        StaticScene::loop_(self)
    }

    fn is_translucent(&self) -> bool {
        StaticScene::is_translucent(self)
    }

    fn duration_seconds(&self) -> f32 {
        StaticScene::duration_seconds(self)
    }

    fn advance_and_apply(&mut self, elapsed_seconds: f32) -> bool {
        StaticScene::advance_and_apply(self, elapsed_seconds)
    }
}
