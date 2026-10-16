//! Supplemental public AngularGradient integration coverage for 16332c8e.
//! Upstream added no tests; this synthetic file uses the existing RivBytes harness.
#![cfg(feature = "tools")]

#[path = "support/riv_bytes.rs"]
mod riv_bytes;

use nuxie_render_api as render;
use nuxie_runtime::{
    File, RuntimeFactoryHandle,
    source::{
        component_dirt::ComponentDirt,
        generated::core_registry::CoreRegistry,
        shapes::paint::{
            angular_gradient::AngularGradient, gradient_stop::GradientStop,
            shape_paint_mutator::ShapePaintMutator,
        },
        shapes::path_flags::PathFlags,
    },
};
use render::Factory;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, PartialEq)]
enum Call {
    Angular(Vec<render::ColorInt>, Vec<f32>),
    Shader,
    Transform(render::Mat2D),
}
type Calls = Rc<RefCell<Vec<Call>>>;
struct AngularFactory(Calls);
impl Factory for AngularFactory {
    fn make_render_buffer(
        &mut self,
        ty: render::RenderBufferType,
        flags: render::RenderBufferFlags,
        size: usize,
    ) -> Box<dyn render::RenderBuffer> {
        render::NullFactory.make_render_buffer(ty, flags, size)
    }
    fn make_linear_gradient(
        &mut self,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: &[render::ColorInt],
        _: &[f32],
    ) -> Box<dyn render::RenderShader> {
        panic!("AngularGradient must not dispatch to linear");
    }
    fn make_radial_gradient(
        &mut self,
        _: f32,
        _: f32,
        _: f32,
        _: &[render::ColorInt],
        _: &[f32],
    ) -> Box<dyn render::RenderShader> {
        panic!("AngularGradient must not dispatch to radial");
    }
    fn make_angular_gradient(
        &mut self,
        colors: &[render::ColorInt],
        stops: &[f32],
    ) -> Box<dyn render::RenderShader> {
        self.0
            .borrow_mut()
            .push(Call::Angular(colors.to_vec(), stops.to_vec()));
        render::NullFactory.make_linear_gradient(0.0, 0.0, 1.0, 0.0, colors, stops)
    }
    fn make_render_path(
        &mut self,
        path: render::RawPath,
        rule: render::FillRule,
    ) -> Box<dyn render::RenderPath> {
        render::NullFactory.make_render_path(path, rule)
    }
    fn make_empty_render_path(&mut self) -> Box<dyn render::RenderPath> {
        render::NullFactory.make_empty_render_path()
    }
    fn make_render_paint(&mut self) -> Box<dyn render::RenderPaint> {
        Box::new(AngularPaint(self.0.clone()))
    }
    fn decode_image(
        &mut self,
        data: &[u8],
    ) -> Result<Box<dyn render::RenderImage>, render::ImageDecodeError> {
        render::NullFactory.decode_image(data)
    }
}
struct AngularPaint(Calls);
impl render::RenderPaint for AngularPaint {
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
    fn additiveness(&mut self, _: f32) {}
    fn shader(&mut self, shader: Option<&dyn render::RenderShader>) {
        assert!(shader.is_some());
        self.0.borrow_mut().push(Call::Shader);
    }
    fn shader_transform(&mut self, transform: render::Mat2D) {
        self.0.borrow_mut().push(Call::Transform(transform));
    }
    fn invalidate_stroke(&mut self) {}
}

#[test]
fn angular_import_clone_registered_stops_and_render_dispatch() {
    let mut riv = riv_bytes::RivBytes::default();
    riv_bytes::write_artboard(&mut riv);
    riv.object(3); // Shape, object 1.
    riv.prop_uint(5, 0);
    riv.end();
    riv.object(20); // Fill, object 2.
    riv.prop_uint(5, 1);
    riv.end();
    riv.object(1089); // AngularGradient, object 3.
    riv.prop_uint(5, 2);
    riv.prop_float(1103, 12.0);
    riv.prop_float(1104, 34.0);
    riv.prop_float(1105, std::f32::consts::FRAC_PI_2);
    riv.prop_float(1106, 2.0);
    riv.prop_float(1107, 3.0);
    riv.end();
    for position in [0.0, 1.0] {
        riv.object(19); // GradientStop, objects 4 and 5.
        riv.prop_uint(5, 3);
        riv.prop_float(39, position);
        riv.end();
    }
    let calls = Calls::default();
    let mut factory = render::PersistentFactory::new(AngularFactory(calls.clone()));
    let file = File::import(
        &riv.bytes(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("synthetic angular gradient imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("cloned artboard");
    let gradient = artboard
        .with_artboard(|a| {
            a.objects()
                .iter()
                .flatten()
                .find(|o| o.is_type_of(1089))
                .cloned()
        })
        .expect("AngularGradient survives import and clone");
    assert!(gradient.is_type_of(22));
    let stops = gradient
        .with_downcast::<AngularGradient, _>(|g| {
            assert_eq!(
                (
                    g.transform_x(),
                    g.transform_y(),
                    g.transform_rotation(),
                    g.transform_scale_x(),
                    g.transform_scale_y()
                ),
                (12.0, 34.0, std::f32::consts::FRAC_PI_2, 2.0, 3.0)
            );
            g.stops().to_vec()
        })
        .unwrap();
    assert_eq!(stops.len(), 2);
    for (stop, position) in stops.iter().zip([0.0, 1.0]) {
        assert_eq!(
            stop.with_downcast::<GradientStop, _>(|s| s.position()),
            Some(position)
        );
        assert_eq!(
            stop.with(|s| s.component_parent_handle()).flatten(),
            Some(gradient.clone())
        );
    }

    gradient.with_mut(|g| g.as_component_mut().unwrap().set_dirt(ComponentDirt::NONE));
    assert!(CoreRegistry::set_double_handle(&gradient, 1103, 12.5));
    assert_eq!(
        gradient.with(|g| g.as_component().unwrap().dirt()),
        Some(ComponentDirt::NONE)
    );
    calls.borrow_mut().clear();
    let mut paint = AngularPaint(calls.clone());
    gradient
        .with_downcast_mut::<AngularGradient, _>(|g| {
            g.apply_to(&mut paint, 1.0, PathFlags::NONE);
        })
        .unwrap();
    let recorded = calls.borrow();
    assert_eq!(recorded.len(), 3);
    let Call::Angular(colors, positions) = &recorded[0] else {
        panic!("angular factory dispatch");
    };
    assert_eq!(colors.len(), 2);
    assert_eq!(positions, &[0.0, 1.0]);
    assert_eq!(recorded[1], Call::Shader);
    let Call::Transform(matrix) = &recorded[2] else {
        panic!("shader transform follows shader");
    };
    let s = std::f32::consts::FRAC_PI_2.sin();
    let c = std::f32::consts::FRAC_PI_2.cos();
    assert_eq!(matrix.0, [c * 2.0, s * 2.0, -s * 3.0, c * 3.0, 12.5, 34.0]);
}
