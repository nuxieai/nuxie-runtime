//! Shared literal fixtures from paint_artboard_test.cpp at fa2cd6e3.
#![allow(dead_code)]
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    core::CoreArena,
    generated::{
        component_base::ComponentBase,
        core_registry::CoreRegistry,
        layout_component_base::LayoutComponentBase,
        node_base::NodeBase,
        shapes::{
            paint::{paint_image_base::PaintImageBase, solid_color_base::SolidColorBase},
            parametric_path_base::ParametricPathBase,
        },
    },
    shapes::{
        paint::{fill::Fill, paint_image::PaintImage, solid_color::SolidColor},
        rectangle::Rectangle,
        shape::Shape,
    },
    status_code::StatusCode,
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_number::ViewModelInstanceNumber,
    },
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
pub fn number(h: &CoreHandle, k: u16, v: f32) {
    assert!(CoreRegistry::set_double_handle(h, k.into(), v));
}
pub fn uint(h: &CoreHandle, k: u16, v: u32) {
    assert!(CoreRegistry::set_uint_handle(h, k.into(), v));
}
pub fn approx(a: f32, b: f32) {
    assert!(
        (f64::from(a) - f64::from(b)).abs() <= f64::from(100.0 * f32::EPSILON) * f64::from(b).abs(),
        "{a} != {b}"
    );
}
pub fn advance(a: &RuntimeArtboardInstanceHandle, dt: f32) {
    a.advance(
        dt,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}
pub struct Scene {
    pub arena: CoreArena,
    pub artboard: CoreHandle,
    pub shape: CoreHandle,
    pub paint: Option<CoreHandle>,
}
impl Scene {
    pub fn new(
        factory: RuntimeFactoryHandle,
        width: f32,
        height: f32,
        source: Option<Option<CoreHandle>>,
        mode: u32,
    ) -> Self {
        let arena = CoreArena::default();
        let artboard = arena.insert(Artboard::with_factory(factory));
        artboard
            .with_downcast_mut::<Artboard, _>(|a| {
                a.set_core_arena(arena.clone());
                a.add_object(Some(artboard.clone()));
            })
            .unwrap();
        let add = |h: &CoreHandle, parent: &CoreHandle| {
            let id = artboard
                .with_downcast::<Artboard, _>(|a| a.id_of(parent))
                .unwrap();
            uint(h, ComponentBase::PARENT_ID_PROPERTY_KEY, id);
            artboard
                .with_downcast_mut::<Artboard, _>(|a| a.add_object(Some(h.clone())))
                .unwrap();
        };
        let shape = arena.insert(Shape::default());
        add(&shape, &artboard);
        let rect = arena.insert(Rectangle::default());
        number(&rect, ParametricPathBase::WIDTH_PROPERTY_KEY, width);
        number(&rect, ParametricPathBase::HEIGHT_PROPERTY_KEY, height);
        add(&rect, &shape);
        let fill = arena.insert(Fill::default());
        add(&fill, &shape);
        let color = arena.insert(SolidColor::default());
        assert!(CoreRegistry::set_color_handle(
            &color,
            SolidColorBase::COLOR_VALUE_PROPERTY_KEY.into(),
            0xffffffffu32 as i32
        ));
        add(&color, &fill);
        let paint = source.as_ref().map(|_| {
            let p = arena.insert(PaintImage::default());
            uint(&p, PaintImageBase::IMAGE_SIZE_MODE_PROPERTY_KEY, mode);
            add(&p, &fill);
            p
        });
        assert_eq!(Artboard::initialize_handle(&artboard), StatusCode::Ok);
        if let Some(source) = source {
            paint
                .as_ref()
                .unwrap()
                .with_downcast_mut::<PaintImage, _>(|p| {
                    p.artboard_referencer.set_referenced_artboard(source)
                })
                .unwrap();
        } else {
            number(&artboard, LayoutComponentBase::WIDTH_PROPERTY_KEY, width);
            number(&artboard, LayoutComponentBase::HEIGHT_PROPERTY_KEY, height);
            number(&shape, NodeBase::X_PROPERTY_KEY, width * 0.5);
            number(&shape, NodeBase::Y_PROPERTY_KEY, height * 0.5);
            Artboard::advance_handle(
                &artboard,
                0.0,
                AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
            );
        }
        Self {
            arena,
            artboard,
            shape,
            paint,
        }
    }
    pub fn source(f: RuntimeFactoryHandle, w: f32, h: f32) -> Self {
        Self::new(f, w, h, None, 0)
    }
    pub fn host(f: RuntimeFactoryHandle, s: Option<CoreHandle>, w: f32, h: f32, m: u32) -> Self {
        Self::new(f, w, h, Some(s), m)
    }
    pub fn instance(&self) -> RuntimeArtboardInstanceHandle {
        let a = Artboard::instance_from_handle(&self.artboard).unwrap();
        advance(&a, 0.0);
        a
    }
}
pub fn paint_of(a: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    a.with_artboard(|a| {
        a.objects()
            .iter()
            .flatten()
            .find(|h| h.with_downcast::<PaintImage, _>(|_| ()).is_some())
            .unwrap()
            .clone()
    })
}
pub fn inner(p: &CoreHandle) -> RuntimeArtboardInstanceHandle {
    p.with_downcast::<PaintImage, _>(|p| p.source.as_ref().unwrap().artboard())
        .unwrap()
}
pub fn first_shape(a: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    a.with_artboard(|a| {
        a.objects()
            .iter()
            .flatten()
            .find(|h| h.with_downcast::<Shape, _>(|_| ()).is_some())
            .unwrap()
            .clone()
    })
}
pub fn x(s: &CoreHandle) -> f32 {
    CoreRegistry::get_double_handle(s, NodeBase::X_PROPERTY_KEY.into()).unwrap()
}
pub fn set_x(s: &CoreHandle, v: f32) {
    number(s, NodeBase::X_PROPERTY_KEY, v);
}
pub fn bound_shape(p: &CoreHandle) -> CoreHandle {
    inner(p).with_artboard(|a| {
        a.objects()
            .iter()
            .flatten()
            .find(|h| h.with_downcast::<Shape, _>(|_| ()).is_some() && x(h) == 25.0)
            .unwrap()
            .clone()
    })
}
pub fn read_file(factory: RuntimeFactoryHandle) -> RuntimeFileHandle {
    let bytes = include_bytes!("../../../../fixtures/sync/paint_artboard_bind.riv");
    File::import(bytes, factory, None, None, None).unwrap()
}
pub fn bind_instance(
    file: &RuntimeFileHandle,
    _host: &RuntimeArtboardInstanceHandle,
) -> CoreHandle {
    file.with_file(|f| f.create_default_view_model_instance(f.view_model_named("vm").unwrap()))
        .unwrap()
}
pub fn red_x(vm: &CoreHandle, value: f32) {
    let p = vm
        .with_downcast::<ViewModelInstance, _>(|v| v.property_value_named("redX"))
        .flatten()
        .unwrap();
    assert!(ViewModelInstanceNumber::set_value_handle(&p, value));
}
