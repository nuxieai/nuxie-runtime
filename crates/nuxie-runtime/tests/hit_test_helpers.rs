use nuxie_render_api::FillRule;
use nuxie_runtime::{
    HitTestArea, HitTestCommandPath, Mat2D, SceneLoop, StaticScene, StaticSceneArtboard,
};

fn add_rectangle(path: &mut HitTestCommandPath, left: f32, top: f32, right: f32, bottom: f32) {
    path.move_to(left, top);
    path.line_to(right, top);
    path.line_to(right, bottom);
    path.line_to(left, bottom);
    path.close();
}

#[test]
fn hittest_basics_direct_port() {
    let mut path = HitTestCommandPath::new(HitTestArea::new(10, 10, 12, 12));
    add_rectangle(&mut path, 0.0, 0.0, 20.0, 20.0);
    assert!(path.was_hit());

    let mut path = HitTestCommandPath::new(HitTestArea::new(81, 156, 84, 159));
    add_rectangle(&mut path, 29.9785, 32.5261, 231.102, 269.898);
    assert!(path.was_hit());
}

#[test]
fn hit_test_command_path_preserves_fill_rule_and_transform_contracts() {
    let mut path = HitTestCommandPath::new(HitTestArea::new(4, 4, 6, 6));
    path.fill_rule(FillRule::EvenOdd);
    add_rectangle(&mut path, 0.0, 0.0, 10.0, 10.0);
    add_rectangle(&mut path, 0.0, 0.0, 10.0, 10.0);
    assert!(
        !path.was_hit(),
        "two identical contours cancel for even-odd"
    );

    path.rewind();
    path.fill_rule(FillRule::NonZero);
    add_rectangle(&mut path, 0.0, 0.0, 10.0, 10.0);
    add_rectangle(&mut path, 0.0, 0.0, 10.0, 10.0);
    assert!(
        path.was_hit(),
        "two identical contours retain non-zero winding"
    );

    path.rewind();
    path.set_transform(Mat2D([2.0, 0.0, 0.0, 2.0, 20.0, 30.0]));
    add_rectangle(&mut path, -10.0, -15.0, -5.0, -10.0);
    assert!(path.was_hit());
}

#[derive(Debug)]
struct StubArtboard {
    name: String,
    translucent: bool,
    advances: Vec<f32>,
}

impl StaticSceneArtboard for StubArtboard {
    fn scene_name(&self) -> &str {
        &self.name
    }

    fn scene_is_translucent(&self) -> bool {
        self.translucent
    }

    fn advance_artboard(&mut self, seconds: f32) -> bool {
        self.advances.push(seconds);
        false
    }
}

#[test]
fn static_scene_matches_the_pinned_cpp_api_contract() {
    let mut artboard = StubArtboard {
        name: "still life".to_owned(),
        translucent: true,
        advances: Vec::new(),
    };
    {
        let mut scene = StaticScene::new(&mut artboard);
        assert_eq!(scene.name(), "still life");
        assert!(scene.is_translucent());
        assert_eq!(scene.loop_kind(), SceneLoop::OneShot);
        assert_eq!(scene.duration_seconds(), 0.0);
        assert!(
            scene.advance_and_apply(12.5),
            "StaticScene ignores the artboard's false advance result"
        );
    }
    assert_eq!(
        artboard.advances,
        [0.0],
        "StaticScene ignores elapsed seconds and advances its artboard at zero"
    );
}
