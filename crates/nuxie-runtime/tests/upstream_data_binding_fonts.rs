//! Direct ports of both pinned `data_binding_fonts_test.cpp` cases.
use std::{path::PathBuf, rc::Rc};

use nuxie_render_api::{Factory, PersistentFactory, SerializingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    math::vec2d::Vec2D,
    text::font_hb::HbFont,
    text_engine::FontRef,
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_asset_font::ViewModelInstanceAssetFont,
    },
};
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};

use nuxie_sriv as sriv;

fn pinned_path(relative: &str) -> PathBuf {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    PathBuf::from(root).join("tests/unit_tests").join(relative)
}
fn pinned_fixture(name: &str) -> Vec<u8> {
    let path = pinned_path(&format!("assets/{name}"));
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

struct Fixture {
    _file: RuntimeFileHandle,
    silver: PersistentFactory<SerializingFactory>,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    view_model: CoreHandle,
}
fn fixture() -> Fixture {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut silver).expect("retained factory");
    let file = File::import(
        &pinned_fixture("data_bind_font_test.riv"),
        retained,
        None,
        None,
        None,
    )
    .expect("native font binding import");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default view-model instance");
    Fixture {
        _file: file,
        silver,
        artboard,
        machine,
        view_model,
    }
}
impl Fixture {
    fn bind(&self) {
        self.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(self.view_model.clone()));
    }
    fn advance(&self, seconds: f32) {
        self.machine.advance_and_apply(seconds);
    }
    fn property(&self) -> CoreHandle {
        let property = self
            .view_model
            .with_downcast::<ViewModelInstance, _>(|instance| {
                instance.property_value_named("fontProperty")
            })
            .flatten()
            .expect("fontProperty");
        assert!(
            property
                .with_downcast::<ViewModelInstanceAssetFont, _>(|_| ())
                .is_some()
        );
        property
    }
}
fn set_font(property: &CoreHandle, font: Option<FontRef>) {
    property
        .with_downcast_mut::<ViewModelInstanceAssetFont, _>(|property| property.set_value(font))
        .expect("native font property");
}
fn stored_font(property: &CoreHandle) -> Option<FontRef> {
    property
        .with_downcast::<ViewModelInstanceAssetFont, _>(|property| property.asset().font())
        .expect("native backing FontAsset")
}
fn source_bytes(font: &FontRef) -> std::sync::Arc<[u8]> {
    font.as_any()
        .downcast_ref::<HbFont>()
        .expect("approved native font backend")
        .source_bytes()
}

#[test]
fn data_bind_font() {
    let fixture = fixture();
    let (width, height) = fixture
        .artboard
        .with_artboard(|artboard| (artboard.width(), artboard.height()));
    fixture
        .silver
        .borrow_mut()
        .frame_size(width as u32, height as u32);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.bind();
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    // The same factory retained by native File admits the font bytes. HbFont
    // then owns those bytes at the approved Rust-native shaping boundary.
    let bytes = pinned_fixture("kablammo.ttf");
    let decoded = fixture
        .silver
        .borrow_mut()
        .decode_font(&bytes)
        .expect("factory decoded font");
    let font = HbFont::decode(decoded.bytes()).expect("native decoded font");
    set_font(&fixture.property(), Some(font));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    fixture.machine.with_instance_mut(|machine| {
        machine.pointer_down(Vec2D::new(490.0, 490.0), 0);
        machine.pointer_up(Vec2D::new(490.0, 490.0), 0);
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    fixture.machine.with_instance_mut(|machine| {
        machine.pointer_down(Vec2D::new(490.0, 20.0), 0);
        machine.pointer_up(Vec2D::new(490.0, 20.0), 0);
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);

    let expected =
        std::fs::read(pinned_path("silvers/data_bind_font_test.sriv")).expect("pinned font silver");
    let actual = fixture.silver.borrow().bytes().to_vec();
    assert_eq!(actual.len(), expected.len(), "pinned SRIV byte length");
    let expected = sriv::parse_sriv(&expected).expect("valid pinned SRIV");
    let actual = sriv::parse_sriv(&actual).expect("valid native SRIV");
    sriv::compare_sriv(&expected, &actual).expect("pinned font silver");
}

#[test]
fn font_data_bind_stores_and_clears_the_font_on_the_property() {
    let fixture = fixture();
    fixture.bind();
    fixture.advance(0.0);
    let property = fixture.property();

    let kablammo = pinned_fixture("kablammo.ttf");
    let font = HbFont::decode(&kablammo).expect("kablammo decoded");
    set_font(&property, Some(font.clone()));
    fixture.advance(0.0);
    let installed_kablammo = stored_font(&property).expect("backing FontAsset retains kablammo");
    assert!(Rc::ptr_eq(&installed_kablammo, &font));
    assert_eq!(
        source_bytes(&installed_kablammo).as_ref(),
        kablammo.as_slice()
    );

    let nabla = pinned_fixture("nabla.ttf");
    let font2 = HbFont::decode(&nabla).expect("nabla decoded");
    set_font(&property, Some(font2.clone()));
    fixture.advance(0.0);
    let installed_nabla = stored_font(&property).expect("backing FontAsset retains nabla");
    assert!(Rc::ptr_eq(&installed_nabla, &font2));
    assert_eq!(source_bytes(&installed_nabla).as_ref(), nabla.as_slice());
    assert!(!Rc::ptr_eq(&installed_kablammo, &installed_nabla));

    set_font(&property, None);
    fixture.advance(0.0);
    assert!(stored_font(&property).is_none());
}
