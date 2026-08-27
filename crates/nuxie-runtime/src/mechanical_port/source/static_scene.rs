use crate::mechanical_port::source::{
    animation::loop_::Loop, artboard::ArtboardInstance, scene::Scene,
};

pub struct StaticScene {
    pub scene: Scene,
}

impl StaticScene {
    pub fn new(artboard_instance: Box<ArtboardInstance>) -> Self {
        Self {
            scene: Scene::new(artboard_instance),
        }
    }

    pub fn is_translucent(&self) -> bool {
        self.scene.artboard_instance().is_translucent()
    }

    pub fn name(&self) -> String {
        self.scene.artboard_instance().name().to_owned()
    }

    pub fn loop_(&self) -> Loop {
        Loop::OneShot
    }

    pub fn duration_seconds(&self) -> f32 {
        0.0
    }

    pub fn advance_and_apply(&mut self, _seconds: f32) -> bool {
        self.scene.artboard_instance_mut().advance(0.0);
        true
    }
}
