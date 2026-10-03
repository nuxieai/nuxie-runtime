//! Direct port of pinned `tests/unit_tests/runtime/stroke_test.cpp`.

use std::path::PathBuf;

use nuxie::{File, PersistentFactory, RuntimeFactoryHandle};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::source::{
    generated::shapes::paint::solid_color_base::SolidColorBase,
    shapes::paint::{solid_color::SolidColor, stroke::Stroke},
};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

#[test]
fn stroke_can_be_looked_up_at_runtime() {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let file = File::import(
        &pinned_fixture("stroke_name_test.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("import fixture");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let stroke = artboard
        .with_artboard(|artboard| artboard.base.find_handle::<Stroke>("white_stroke"))
        .expect("named stroke");
    let paint = stroke
        .with_downcast::<Stroke, _>(|stroke| stroke.base.paint())
        .flatten()
        .expect("stroke paint");
    assert!(paint.is_type_of(SolidColorBase::TYPE_KEY));
    paint
        .with_downcast_mut::<SolidColor, _>(|paint| paint.set_color_value(0xff00_ffffu32 as i32))
        .expect("solid-color owner");
}

use nuxie_render_api as render;
use nuxie_runtime::source::{
    animation::keyframe_uint::KeyFrameUint,
    artboard::RuntimeArtboardInstanceHandle,
    core::CoreHandle,
    generated::{core_registry::CoreRegistry, shapes::paint::stroke_base::StrokeBase},
};

struct StrokePositionPaint {
    position: render::StrokePosition,
    writes: usize,
}
impl Default for StrokePositionPaint {
    fn default() -> Self {
        Self {
            position: render::StrokePosition::Center,
            writes: 0,
        }
    }
}
impl render::RenderPaint for StrokePositionPaint {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn style(&mut self, _: render::RenderPaintStyle) {}
    fn color(&mut self, _: render::ColorInt) {}
    fn thickness(&mut self, _: f32) {}
    fn join(&mut self, _: render::StrokeJoin) {}
    fn cap(&mut self, _: render::StrokeCap) {}
    fn feather(&mut self, _: f32) {}
    fn blend_mode(&mut self, _: render::BlendMode) {}
    fn shader(&mut self, _: Option<&dyn render::RenderShader>) {}
    fn invalidate_stroke(&mut self) {}
    fn stroke_position(&mut self, value: render::StrokePosition) {
        self.position = value;
        self.writes += 1;
    }
}
#[derive(Default)]
struct StrokePositionFactory(render::NullFactory);
impl render::Factory for StrokePositionFactory {
    fn make_render_buffer(
        &mut self,
        kind: render::RenderBufferType,
        flags: render::RenderBufferFlags,
        size: usize,
    ) -> Box<dyn render::RenderBuffer> {
        self.0.make_render_buffer(kind, flags, size)
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[render::ColorInt],
        stops: &[f32],
    ) -> Box<dyn render::RenderShader> {
        self.0.make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }
    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[render::ColorInt],
        stops: &[f32],
    ) -> Box<dyn render::RenderShader> {
        self.0.make_radial_gradient(cx, cy, radius, colors, stops)
    }
    fn make_render_path(
        &mut self,
        path: render::RawPath,
        rule: render::FillRule,
    ) -> Box<dyn render::RenderPath> {
        self.0.make_render_path(path, rule)
    }
    fn make_empty_render_path(&mut self) -> Box<dyn render::RenderPath> {
        self.0.make_empty_render_path()
    }
    fn make_render_paint(&mut self) -> Box<dyn render::RenderPaint> {
        Box::new(StrokePositionPaint::default())
    }
    fn decode_image(
        &mut self,
        bytes: &[u8],
    ) -> Result<Box<dyn render::RenderImage>, render::ImageDecodeError> {
        self.0.decode_image(bytes)
    }
}
fn position_fixture() -> (RuntimeArtboardInstanceHandle, CoreHandle) {
    let mut factory = PersistentFactory::new(StrokePositionFactory::default());
    let file = File::import(
        &pinned_fixture("stroke_name_test.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let source = file.with_file(File::artboard).unwrap();
    let artboard = nuxie_runtime::Artboard::instance_from_handle(&source).unwrap();
    let stroke = artboard
        .with_artboard(|a| a.find_handle::<Stroke>("white_stroke"))
        .unwrap();
    (artboard, stroke)
}
fn paint_position(stroke: &CoreHandle) -> (render::StrokePosition, usize) {
    let paint = stroke
        .with_downcast::<Stroke, _>(|s| s.render_paint_handle())
        .flatten()
        .unwrap();
    let paint = paint.borrow();
    let paint = paint
        .as_any()
        .downcast_ref::<StrokePositionPaint>()
        .unwrap();
    (paint.position, paint.writes)
}
fn set_position(stroke: &CoreHandle, position: u8) {
    stroke
        .with_downcast_mut::<Stroke, _>(|s| s.set_position(position))
        .unwrap();
}
#[test]
fn stroke_position_defaults_to_center() {
    let (_artboard, stroke) = position_fixture();
    assert_eq!(stroke.with_downcast::<Stroke, _>(|s| s.position()), Some(1));
    assert_eq!(
        stroke.with_downcast::<Stroke, _>(Stroke::stroke_position),
        Some(render::StrokePosition::Center)
    );
    let (position, writes) = paint_position(&stroke);
    assert!(writes > 0);
    assert_eq!(position, render::StrokePosition::Center);
}
#[test]
fn changing_stroke_position_reaches_render_paint() {
    let (artboard, stroke) = position_fixture();
    for (value, expected) in [
        (0, render::StrokePosition::Inside),
        (2, render::StrokePosition::Outside),
        (7, render::StrokePosition::Center),
    ] {
        set_position(&stroke, value);
        artboard.advance(
            0.0,
            nuxie_runtime::AdvanceFlags::ADVANCE_NESTED
                | nuxie_runtime::AdvanceFlags::ANIMATE
                | nuxie_runtime::AdvanceFlags::NEW_FRAME,
        );
        assert_eq!(paint_position(&stroke).0, expected);
    }
    assert_eq!(
        stroke.with_downcast::<Stroke, _>(Stroke::stroke_position),
        Some(render::StrokePosition::Center)
    );
}
#[test]
fn stroke_apply_to_carries_position() {
    let (_artboard, stroke) = position_fixture();
    set_position(&stroke, 2);
    let mut target = StrokePositionPaint::default();
    stroke
        .with_downcast_mut::<Stroke, _>(|s| s.apply_to(&mut target, 1.0))
        .unwrap();
    assert_eq!(target.position, render::StrokePosition::Outside);
}
#[test]
fn keyframes_and_binds_drive_stroke_position() {
    let (artboard, stroke) = position_fixture();
    let mut keyframe = KeyFrameUint::default();
    CoreRegistry::set_uint(&mut keyframe, 631, 2);
    stroke
        .with_mut(|s| keyframe.apply(s, i32::from(StrokeBase::POSITION_PROPERTY_KEY), 1.0, None))
        .unwrap();
    artboard.advance(
        0.0,
        nuxie_runtime::AdvanceFlags::ADVANCE_NESTED
            | nuxie_runtime::AdvanceFlags::ANIMATE
            | nuxie_runtime::AdvanceFlags::NEW_FRAME,
    );
    assert_eq!(paint_position(&stroke).0, render::StrokePosition::Outside);
    CoreRegistry::set_uint_handle(&stroke, i32::from(StrokeBase::POSITION_PROPERTY_KEY), 0);
    artboard.advance(
        0.0,
        nuxie_runtime::AdvanceFlags::ADVANCE_NESTED
            | nuxie_runtime::AdvanceFlags::ANIMATE
            | nuxie_runtime::AdvanceFlags::NEW_FRAME,
    );
    assert_eq!(paint_position(&stroke).0, render::StrokePosition::Inside);
}
#[test]
fn stroke_position_holds_between_keyframes() {
    assert!(!CoreRegistry::is_interpolatable_uint(u32::from(
        StrokeBase::POSITION_PROPERTY_KEY
    )));
    let (_artboard, stroke) = position_fixture();
    let mut keyframe = KeyFrameUint::default();
    CoreRegistry::set_uint(&mut keyframe, 631, 0);
    stroke
        .with_mut(|s| keyframe.apply(s, i32::from(StrokeBase::POSITION_PROPERTY_KEY), 0.5, None))
        .unwrap();
    assert_eq!(stroke.with_downcast::<Stroke, _>(|s| s.position()), Some(0));
}
