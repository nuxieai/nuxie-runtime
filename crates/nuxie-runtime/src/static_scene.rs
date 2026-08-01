use crate::ArtboardInstance;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneLoop {
    OneShot,
    Loop,
    PingPong,
}

/// Narrow artboard surface consumed by [`StaticScene`].
///
/// `ArtboardInstance` implements this production seam. Keeping the delegation
/// explicit also makes the otherwise-untested upstream helper's API contract
/// executable without synthesizing animation behavior into a static scene.
pub trait StaticSceneArtboard {
    fn scene_name(&self) -> &str;
    fn scene_is_translucent(&self) -> bool;
    fn advance_artboard(&mut self, seconds: f32) -> bool;
}

impl StaticSceneArtboard for ArtboardInstance {
    fn scene_name(&self) -> &str {
        self.runtime_graph()
            .and_then(|graph| graph.name.as_deref())
            .unwrap_or("")
    }

    fn scene_is_translucent(&self) -> bool {
        self.runtime_is_translucent()
    }

    fn advance_artboard(&mut self, seconds: f32) -> bool {
        self.advance(seconds).unwrap_or(false)
    }
}

/// Non-animated scene wrapper around one artboard occurrence.
pub struct StaticScene<'a, A: StaticSceneArtboard + ?Sized = ArtboardInstance> {
    artboard: &'a mut A,
}

impl<'a, A: StaticSceneArtboard + ?Sized> StaticScene<'a, A> {
    pub fn new(artboard: &'a mut A) -> Self {
        Self { artboard }
    }

    pub fn is_translucent(&self) -> bool {
        self.artboard.scene_is_translucent()
    }

    pub fn name(&self) -> &str {
        self.artboard.scene_name()
    }

    pub fn loop_kind(&self) -> SceneLoop {
        SceneLoop::OneShot
    }

    pub fn duration_seconds(&self) -> f32 {
        0.0
    }

    pub fn advance_and_apply(&mut self, _seconds: f32) -> bool {
        let _ = self.artboard.advance_artboard(0.0);
        true
    }
}
