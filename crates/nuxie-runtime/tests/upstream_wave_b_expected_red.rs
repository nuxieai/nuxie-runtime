//! Native converter/cycle tests followed by preserved Wave B C++ cases.
//! Remaining ignored entries retain their complete upstream action/assertion bodies.

use nuxie_render_api::{Factory, PersistentFactory, RecordingFactory, SerializingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    core::{CoreArena, CoreType},
    math::vec2d::Vec2D,
    nested_artboard::NestedArtboard,
    shapes::rectangle::Rectangle,
    text::text_value_run::TextValueRun,
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_color::ViewModelInstanceColor,
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_number::ViewModelInstanceNumber,
    },
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
use std::path::PathBuf;

use nuxie_sriv as sriv;

fn binding_path(relative: &str) -> PathBuf {
    PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests")
    .join(relative)
}

fn binding_file<F: Factory + 'static>(
    asset: &str,
    factory: &mut PersistentFactory<F>,
) -> RuntimeFileHandle {
    let path = binding_path(&format!("assets/{asset}"));
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let retained = RuntimeFactoryHandle::from_factory(factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("{asset} imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    file
}

fn binding_property(instance: &CoreHandle, name: &str) -> CoreHandle {
    instance
        .with_downcast::<ViewModelInstance, _>(|instance| instance.property_value_named(name))
        .flatten()
        .unwrap_or_else(|| panic!("view model property {name}"))
}

fn binding_set_number(property: &CoreHandle, value: f32) {
    property
        .with_downcast_mut::<ViewModelInstanceNumber, _>(|property| property.set_value(value))
        .expect("number property");
}

fn binding_set_color(property: &CoreHandle, value: u32) {
    property
        .with_downcast_mut::<ViewModelInstanceColor, _>(|property| property.set_value(value as i32))
        .expect("color property");
}

fn binding_find<T: CoreType>(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    artboard
        .with_artboard(|artboard| artboard.find_handle::<T>(name))
        .unwrap_or_else(|| panic!("authored object {name}"))
}

fn binding_child(
    artboard: &RuntimeArtboardInstanceHandle,
    name: &str,
) -> RuntimeArtboardInstanceHandle {
    let nested = binding_find::<NestedArtboard>(artboard, name);
    nested
        .with(|nested| nested.nested_artboard_instance_handle())
        .flatten()
        .expect("mounted nested artboard")
}

fn binding_width(rect: &CoreHandle) -> f32 {
    rect.with_downcast::<Rectangle, _>(|rect| rect.base.width())
        .expect("Rectangle")
}

fn binding_text(run: &CoreHandle) -> String {
    run.with_downcast::<TextValueRun, _>(|run| run.base.text().to_owned())
        .expect("TextValueRun")
}

struct BindingSilver {
    machine: RuntimeStateMachineInstanceHandle,
    artboard: RuntimeArtboardInstanceHandle,
    file: RuntimeFileHandle,
    silver: PersistentFactory<SerializingFactory>,
}

impl BindingSilver {
    fn new(asset: &str) -> Self {
        let mut silver = PersistentFactory::new(SerializingFactory::new());
        let file = binding_file(asset, &mut silver);
        let artboard = file
            .with_file(File::artboard_default)
            .expect("default artboard");
        let (width, height) =
            artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
        silver.borrow_mut().frame_size(width as u32, height as u32);
        let machine = artboard.state_machine_at(0).expect("state machine 0");
        Self {
            machine,
            artboard,
            file,
            silver,
        }
    }
    fn instance(&self) -> CoreHandle {
        let id = self
            .artboard
            .with_artboard(|artboard| artboard.base.view_model_id());
        self.file
            .with_file_mut(|file| {
                if id == u32::MAX {
                    file.create_view_model_instance_for_artboard(self.artboard.core_handle())
                } else {
                    file.create_view_model_instance_at(id as usize, 0)
                }
            })
            .expect("view model instance")
    }
    fn advance(&self, elapsed: f32) {
        self.machine.advance_and_apply(elapsed);
    }
    fn matches(&self, name: &str) {
        let expected =
            std::fs::read(binding_path(&format!("silvers/{name}.sriv"))).expect("pinned silver");
        let actual = self.silver.borrow().bytes().to_vec();
        let expected_sriv = sriv::parse_sriv(&expected).expect("valid pinned SRIV");
        let actual_sriv = sriv::parse_sriv(&actual).expect("valid native SRIV");
        sriv::compare_sriv(&expected_sriv, &actual_sriv).expect("pinned converter silver");
        assert_eq!(actual.len(), expected.len(), "pinned SRIV byte length");
    }
}

#[test]
fn wave_b_data_binding_converters_test_001_direct_port_expected_red() {
    let fixture = BindingSilver::new("list_to_length_test.riv");
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance.clone()));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let list = binding_property(&instance, "lis");
    let child_model = fixture
        .file
        .with_file(|file| file.view_model_named("child"))
        .expect("child model");
    for _ in 0..4 {
        fixture.silver.borrow_mut().add_frame();
        let child = fixture
            .file
            .with_file_mut(|file| file.create_default_view_model_instance(child_model.clone()));
        if let Some(child) = child {
            let mut item = ViewModelInstanceListItem::default();
            item.set_view_model_instance(Some(child));
            let item = list.insert_sibling(item).expect("native list item");
            list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item(item))
                .expect("list property");
        }
        fixture.advance(0.1);
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("list_to_length_test");
}

#[test]
fn wave_b_data_binding_converters_test_002_direct_port_expected_red() {
    let fixture = BindingSilver::new("data_converter_interpolator_reset.riv");
    let mut renderer = fixture.silver.borrow().make_renderer();
    {
        let instance = fixture.instance();
        let number = binding_property(&instance, "xPos");
        let color = binding_property(&instance, "col");
        binding_set_number(&number, 250.0);
        binding_set_color(&color, (255_u32 << 24) | (255 << 16));
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        binding_set_color(&color, (255_u32 << 24) | (255 << 8));
        binding_set_number(&number, 500.0);
        for _ in 0..(1.0_f32 / 0.016_f32) as i32 {
            fixture.silver.borrow_mut().add_frame();
            fixture.advance(0.016);
            fixture.artboard.draw(&mut renderer);
        }
    }
    {
        fixture.silver.borrow_mut().add_frame();
        let instance = fixture.instance();
        let number = binding_property(&instance, "xPos");
        let color = binding_property(&instance, "col");
        binding_set_number(&number, 250.0);
        binding_set_color(&color, (255_u32 << 24) | (255 << 16));
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        binding_set_color(&color, (255_u32 << 24) | 255);
        binding_set_number(&number, 0.0);
        for _ in 0..(1.0_f32 / 0.016_f32) as i32 {
            fixture.silver.borrow_mut().add_frame();
            fixture.advance(0.016);
            fixture.artboard.draw(&mut renderer);
        }
    }
    fixture.matches("data_converter_interpolator_reset");
}

#[test]
fn wave_b_data_binding_converters_test_003_direct_port_expected_red() {
    let fixture = BindingSilver::new("interpolation_zero_duration.riv");
    let instance = fixture.instance();
    let object_x = binding_property(&instance, "objectX");
    let interp_value = binding_property(&instance, "interpValue");
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    binding_set_number(&object_x, 200.0);
    let frames = (1.5_f32 / 0.1_f32) as i32;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
    }
    binding_set_number(&interp_value, 0.0);
    fixture.advance(0.016);
    binding_set_number(&object_x, 400.0);
    fixture.advance(0.016);
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
    }
    binding_set_number(&interp_value, 1.0);
    fixture.advance(0.016);
    binding_set_number(&object_x, 200.0);
    fixture.advance(0.016);
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("interpolation_zero_duration");
}

struct BindingCycle {
    machine: RuntimeStateMachineInstanceHandle,
    artboard: RuntimeArtboardInstanceHandle,
    _file: RuntimeFileHandle,
}

impl BindingCycle {
    fn new(name: &str) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = binding_file("data_binding_test_3.riv", &mut factory);
        let source = file
            .with_file(|file| file.artboard_named_source(name))
            .expect("named source artboard");
        let artboard = Artboard::instance_from_handle(&source).expect("artboard instance");
        let instance = file
            .with_file_mut(|file| {
                file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .expect("default view model instance");
        let machine = artboard
            .default_state_machine_handle()
            .expect("default state machine");
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        machine.advance_and_apply(0.0);
        Self {
            machine,
            artboard,
            _file: file,
        }
    }
    fn advance(&self, elapsed: f32) {
        self.machine.advance_and_apply(elapsed);
    }
    fn click(&self, point: Vec2D) {
        self.machine
            .with_instance_mut(|machine| machine.pointer_down(point, 0));
        self.machine
            .with_instance_mut(|machine| machine.pointer_up(point, 0));
    }
}

#[test]
fn wave_b_data_binding_cycle_test_001_direct_port_expected_red() {
    let fixture = BindingCycle::new("main-1");
    let rect = binding_find::<Rectangle>(&fixture.artboard, "sized-rect-path");
    assert_eq!(binding_width(&rect), 100.0);
    fixture.click(Vec2D::new(75.0, 75.0));
    fixture.advance(0.0);
    assert_eq!(binding_width(&rect), 200.0);
}

#[test]
fn wave_b_data_binding_cycle_test_002_direct_port_expected_red() {
    let fixture = BindingCycle::new("main-2");
    let child = binding_child(&fixture.artboard, "child-2");
    let rect = binding_find::<Rectangle>(&child, "child-rect-path");
    assert_eq!(binding_width(&rect), 100.0);
    fixture.click(Vec2D::new(250.0, 250.0));
    fixture.advance(0.0);
    assert_eq!(binding_width(&rect), 200.0);
}

#[test]
fn wave_b_data_binding_cycle_test_003_direct_port_expected_red() {
    let fixture = BindingCycle::new("main-3");
    let rect = binding_find::<Rectangle>(&fixture.artboard, "sized-rect-path");
    assert_eq!(binding_width(&rect), 100.0);
    fixture.advance(0.5);
    assert_eq!(binding_width(&rect), 100.0);
    fixture.advance(0.0);
    assert_eq!(binding_width(&rect), 200.0);
}

#[test]
fn wave_b_data_binding_cycle_test_004_direct_port_expected_red() {
    let fixture = BindingCycle::new("main-4");
    let child = binding_child(&fixture.artboard, "child-4");
    let rect = binding_find::<Rectangle>(&child, "child-rect-path");
    assert_eq!(binding_width(&rect), 100.0);
    fixture.advance(0.5);
    assert_eq!(binding_width(&rect), 100.0);
    fixture.advance(0.0);
    assert_eq!(binding_width(&rect), 200.0);
}

#[test]
fn wave_b_data_binding_cycle_test_005_direct_port_expected_red() {
    let fixture = BindingCycle::new("main-5");
    let run = binding_find::<TextValueRun>(&fixture.artboard, "text-run-test");
    assert_eq!(binding_text(&run), "before");
    fixture.advance(0.5);
    assert_eq!(binding_text(&run), "after");
}

#[test]
fn wave_b_data_binding_cycle_test_006_direct_port_expected_red() {
    let fixture = BindingCycle::new("main-6");
    let child = binding_child(&fixture.artboard, "child-6");
    let run = binding_find::<TextValueRun>(&child, "child-text-run");
    assert_eq!(binding_text(&run), "parent-before");
    fixture.advance(0.5);
    assert_eq!(binding_text(&run), "parent-after");
}

#[test]
fn wave_b_data_binding_cycle_test_007_direct_port_expected_red() {
    let fixture = BindingCycle::new("main-7");
    let main_run = binding_find::<TextValueRun>(&fixture.artboard, "main-run");
    let child = binding_child(&fixture.artboard, "child-7");
    let child_run = binding_find::<TextValueRun>(&child, "child-run");
    let grandchild = binding_child(&child, "grand-child-7");
    let grandchild_run = binding_find::<TextValueRun>(&grandchild, "grand-child-run");
    for (elapsed, expected) in [
        (0.5, "main-test-2"),
        (1.5, "child-text-1"),
        (0.5, "child-text-2"),
        (1.5, "grand-child-text-1"),
        (0.5, "grand-child-text-2"),
    ] {
        fixture.advance(elapsed);
        assert_eq!(binding_text(&main_run), expected);
        assert_eq!(binding_text(&child_run), expected);
        assert_eq!(binding_text(&grandchild_run), expected);
    }
}

// Native asset-binding observations and fixture construction. These retain the
// actual File/artboard/SMI owners; SRIV comparison is the shared pinned oracle.
use nuxie_runtime::source::{
    generated::{
        core_registry::CoreRegistry, shapes::image_base::ImageBase,
        viewmodel::viewmodel_instance_asset_base::ViewModelInstanceAssetBase,
    },
    layout::Fit,
    layout_component::LayoutComponent,
    shapes::image::Image,
    text::{
        font_hb::HbFont,
        text_engine::{Coord, Font, FontRef, TextRun, with_host_fallback_proc},
    },
    viewmodel::{
        viewmodel_instance_asset_font::ViewModelInstanceAssetFont,
        viewmodel_instance_asset_image::ViewModelInstanceAssetImage,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};
use std::rc::Rc;

fn binding_asset_silver(asset: &str, artboard_name: Option<&str>) -> BindingSilver {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let file = binding_file(asset, &mut silver);
    let artboard = file
        .with_file(|file| match artboard_name {
            Some(name) => file.artboard_named(name),
            None => file.artboard_default(),
        })
        .expect("authored artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    BindingSilver {
        machine,
        artboard,
        file,
        silver,
    }
}

fn binding_default_instance(
    file: &RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
) -> CoreHandle {
    file.with_file_mut(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    })
    .expect("default view model instance")
}

fn binding_set_font(property: &CoreHandle, font: Option<FontRef>) {
    property
        .with_downcast_mut::<ViewModelInstanceAssetFont, _>(|property| property.set_value(font))
        .expect("font property");
}

fn binding_set_image(property: &CoreHandle, image: Option<Rc<dyn nuxie_render_api::RenderImage>>) {
    property
        .with_downcast_mut::<ViewModelInstanceAssetImage, _>(|property| property.set_value(image))
        .expect("image property");
}

fn binding_set_asset_index(property: &CoreHandle, index: u32) {
    CoreRegistry::set_uint_handle(
        property,
        ViewModelInstanceAssetBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
        index,
    );
}

fn binding_asset_index(property: &CoreHandle) -> usize {
    property
        .with_downcast::<ViewModelInstanceAssetImage, _>(|property| {
            property.base.property_value() as usize
        })
        .expect("image property")
}

fn binding_click(machine: &RuntimeStateMachineInstanceHandle, point: Vec2D) {
    machine.with_instance_mut(|machine| machine.pointer_down(point, 0));
    machine.with_instance_mut(|machine| machine.pointer_up(point, 0));
}

fn binding_images(artboard: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
    artboard.with_artboard(|artboard| artboard.objects_typed::<Image>().iter().collect())
}

fn binding_image_fit(image: &CoreHandle) -> u32 {
    image
        .with_downcast::<Image, _>(|image| image.base.fit())
        .expect("Image")
}

fn binding_image_alignment(image: &CoreHandle) -> (f32, f32) {
    image
        .with_downcast::<Image, _>(|image| (image.base.alignment_x(), image.base.alignment_y()))
        .expect("Image")
}

fn binding_set_image_fit_alignment(image: &CoreHandle, fit: u32, x: f32, y: f32) {
    CoreRegistry::set_uint_handle(image, ImageBase::FIT_PROPERTY_KEY.into(), fit);
    CoreRegistry::set_double_handle(image, ImageBase::ALIGNMENT_X_PROPERTY_KEY.into(), x);
    CoreRegistry::set_double_handle(image, ImageBase::ALIGNMENT_Y_PROPERTY_KEY.into(), y);
}

// Pinned Catch Approx defaults, including the float operands' double promotion.
fn binding_asset_approx(actual: f32, expected: f32) -> bool {
    let actual = f64::from(actual);
    let expected = f64::from(expected);
    let margin = f64::from(f32::EPSILON) * 100.0 * expected.abs();
    (expected >= actual && actual >= expected)
        || (expected + margin >= actual && actual + margin >= expected)
}

#[test]
fn wave_b_data_binding_fonts_test_001_direct_port_expected_red() {
    let fixture = binding_asset_silver("data_bind_font_test.riv", None);
    let vmi = binding_default_instance(&fixture.file, &fixture.artboard);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(vmi.clone()));
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    let font_bytes = std::fs::read(binding_path("assets/kablammo.ttf")).expect("kablammo.ttf");
    let decoded = fixture
        .silver
        .borrow_mut()
        .decode_font(&font_bytes)
        .expect("factory font decode");
    let font = HbFont::decode(decoded.bytes()).expect("native font");
    let font_property = binding_property(&vmi, "fontProperty");
    assert!(font_property.is_type_of(ViewModelInstanceAssetFont::TYPE_KEY));
    binding_set_font(&font_property, Some(font));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    binding_click(&fixture.machine, Vec2D::new(490.0, 490.0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_click(&fixture.machine, Vec2D::new(490.0, 20.0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("data_bind_font_test");
}

#[test]
fn wave_b_data_binding_fonts_test_002_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("data_bind_font_test.riv", &mut factory);
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let machine = artboard.state_machine_at(0).expect("state machine");
    let vmi = binding_default_instance(&file, &artboard);
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(vmi.clone()));
    machine.advance_and_apply(0.0);
    let property = binding_property(&vmi, "fontProperty");
    assert!(property.is_type_of(ViewModelInstanceAssetFont::TYPE_KEY));
    let asset = property
        .with_downcast::<ViewModelInstanceAssetFont, _>(|property| property.asset())
        .expect("backing FontAsset");

    let bytes = std::fs::read(binding_path("assets/kablammo.ttf")).expect("kablammo.ttf");
    let font = HbFont::decode(&bytes).expect("native font");
    binding_set_font(&property, Some(font.clone()));
    machine.advance_and_apply(0.0);
    assert!(Rc::ptr_eq(&asset.font().expect("stored font"), &font));

    let bytes = std::fs::read(binding_path("assets/nabla.ttf")).expect("nabla.ttf");
    let font2 = HbFont::decode(&bytes).expect("second native font");
    binding_set_font(&property, Some(font2.clone()));
    machine.advance_and_apply(0.0);
    assert!(Rc::ptr_eq(&asset.font().expect("replaced font"), &font2));

    binding_set_font(&property, None);
    machine.advance_and_apply(0.0);
    assert!(asset.font().is_none());
}

#[test]
fn wave_b_data_binding_images_test_001_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("data_binding_images_test.riv", &mut factory);
    let artboard = file
        .with_file(|file| file.artboard_named("main"))
        .expect("main instance");
    let vmi = binding_default_instance(&file, &artboard);
    artboard.bind_view_model_instance(Some(vmi.clone()));
    artboard.advance_default(0.0);
    let main_property = binding_property(&vmi, "main_im");
    assert!(main_property.is_type_of(ViewModelInstanceAssetImage::TYPE_KEY));
    let sub_vmi_property = binding_property(&vmi, "sub_1");
    assert!(sub_vmi_property.is_type_of(ViewModelInstanceViewModel::TYPE_KEY));
    let referenced = sub_vmi_property
        .with_downcast::<ViewModelInstanceViewModel, _>(|property| {
            property.reference_view_model_instance()
        })
        .flatten()
        .expect("referenced view model");
    let sub_property = binding_property(&referenced, "sub_1_im");
    assert!(sub_property.is_type_of(ViewModelInstanceAssetImage::TYPE_KEY));
    let assets = file.with_file(|file| file.assets().to_vec());
    let root_image = binding_find::<Image>(&artboard, "root_img");
    let nested = binding_child(&artboard, "sub_1");
    let sub_image = binding_find::<Image>(&nested, "sub_1_img");
    let image_asset = root_image
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .expect("root image asset");
    let sub_image_asset = sub_image
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .expect("sub image asset");
    assert_eq!(image_asset, assets[binding_asset_index(&main_property)]);
    assert_eq!(sub_image_asset, assets[binding_asset_index(&sub_property)]);
    binding_set_asset_index(&main_property, 2);
    binding_set_asset_index(&sub_property, 6);
    artboard.advance_default(0.0);
    let updated_main = assets[binding_asset_index(&main_property)].clone();
    let updated_sub = assets[binding_asset_index(&sub_property)].clone();
    assert_ne!(image_asset, updated_main);
    assert_ne!(sub_image_asset, updated_sub);
    assert_eq!(
        root_image
            .with_downcast::<Image, _>(Image::image_asset)
            .flatten(),
        Some(updated_main)
    );
    assert_eq!(
        sub_image
            .with_downcast::<Image, _>(Image::image_asset)
            .flatten(),
        Some(updated_sub)
    );
}

#[test]
fn wave_b_data_binding_images_test_002_direct_port_expected_red() {
    let fixture = binding_asset_silver("viewmodel_image_reset.riv", None);
    let vmi = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(vmi.clone()));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let property = binding_property(&vmi, "img");
    binding_set_image(&property, None);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("viewmodel_image_reset");
}

#[test]
fn wave_b_data_binding_images_test_003_direct_port_expected_red() {
    let fixture = binding_asset_silver("viewmodel_based_condition.riv", None);
    let vmi = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(vmi));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(1.1);
    fixture.advance(0.1); // Pinned extra advance processes the event.
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(1.1);
    fixture.advance(0.1); // Pinned extra advance processes the event.
    fixture.artboard.draw(&mut renderer);
    fixture.matches("viewmodel_based_condition");
}

#[test]
fn wave_b_data_binding_images_test_004_direct_port_expected_red() {
    let fixture = binding_asset_silver("image_binding_with_listener.riv", Some("main"));
    let mut renderer = fixture.silver.borrow().make_renderer();
    let id = fixture
        .artboard
        .with_artboard(|artboard| artboard.base.view_model_id());
    let vmi = fixture
        .file
        .with_file_mut(|file| file.create_view_model_instance_at(id as usize, 0))
        .expect("authored view model instance");
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(vmi.clone()));
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_click(&fixture.machine, Vec2D::new(650.0, 650.0));
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    let bytes = std::fs::read(binding_path("assets/open_source.jpg")).expect("open_source.jpg");
    assert_eq!(bytes.len(), 8880);
    let image: Rc<dyn nuxie_render_api::RenderImage> = Rc::from(
        fixture
            .silver
            .borrow_mut()
            .decode_image(&bytes)
            .expect("factory image decode"),
    );
    let property = binding_property(&vmi, "image1");
    binding_set_image(&property, Some(image));
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_click(&fixture.machine, Vec2D::new(650.0, 650.0));
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    binding_set_image(&property, None);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_click(&fixture.machine, Vec2D::new(650.0, 650.0));
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("image_binding_with_listener");
}

#[test]
fn wave_b_data_binding_images_test_005_direct_port_expected_red() {
    let fixture = binding_asset_silver("image_fit_alignment.riv", Some("Main"));
    let vmi = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(vmi.clone()));
    fixture.advance(0.1);
    let image = binding_images(&fixture.artboard)
        .into_iter()
        .next()
        .expect("first image");
    let original_fit = binding_image_fit(&image);
    let (original_x, original_y) = binding_image_alignment(&image);
    let test_fit = if original_fit == Fit::Contain as u32 {
        Fit::Cover as u32
    } else {
        Fit::Contain as u32
    };
    let test_x = if original_x == -1.0 { 1.0 } else { -1.0 };
    let test_y = if original_y == -1.0 { 1.0 } else { -1.0 };
    binding_set_image_fit_alignment(&image, test_fit, test_x, test_y);
    assert_eq!(binding_image_fit(&image), test_fit);
    assert_eq!(binding_image_alignment(&image).0, test_x);
    assert_eq!(binding_image_alignment(&image).1, test_y);
    binding_set_image_fit_alignment(&image, original_fit, original_x, original_y);
    assert_eq!(binding_image_fit(&image), original_fit);
    assert_eq!(binding_image_alignment(&image).0, original_x);
    assert_eq!(binding_image_alignment(&image).1, original_y);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);

    let property = binding_property(&vmi, "imageProperty");
    assert!(property.is_type_of(ViewModelInstanceAssetImage::TYPE_KEY));
    let no_scale_image = || {
        binding_images(&fixture.artboard)
            .into_iter()
            .find(|image| binding_image_fit(image) == Fit::None as u32)
    };
    let assets = fixture.file.with_file(|file| file.assets().to_vec());
    let find_asset_index = |name: &str| {
        assets
            .iter()
            .position(|asset| {
                asset
                    .with(|asset| {
                        asset
                            .as_file_asset()
                            .expect("FileAsset")
                            .file_asset_base()
                            .name()
                            == name
                    })
                    .expect("live asset")
            })
            .unwrap_or(assets.len())
    };
    let image1_index = find_asset_index("image1");
    let image2_index = find_asset_index("image2");
    let image3_index = find_asset_index("image3");
    assert_ne!(image1_index, assets.len());
    assert_ne!(image2_index, assets.len());
    assert_ne!(image3_index, assets.len());

    for _ in 0..20 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    binding_set_asset_index(&property, image2_index as u32);
    fixture.advance(0.0);
    let no_scale = no_scale_image().expect("unscaled image");
    let transform = no_scale
        .with_downcast::<Image, _>(|image| *image.base.transform())
        .unwrap();
    assert!(transform[4] < 0.0);
    assert!(transform[5] < 0.0);
    for _ in 0..20 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    binding_set_asset_index(&property, image3_index as u32);
    fixture.advance(0.0);
    let no_scale = no_scale_image().expect("unscaled image");
    let transform = no_scale
        .with_downcast::<Image, _>(|image| *image.base.transform())
        .unwrap();
    assert!(transform[4] < 0.0);
    assert!(transform[5] < 0.0);
    for _ in 0..20 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("image_fit_alignment");
}

#[test]
fn wave_b_data_binding_images_test_006_direct_port_expected_red() {
    let fixture = binding_asset_silver("image_fit_alignment_2.riv", Some("Main"));
    let vmi = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(vmi));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for _ in 0..60 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("image_fit_alignment_2");
}

#[test]
fn wave_b_data_binding_images_test_007_direct_port_expected_red() {
    let fixture = binding_asset_silver("image_fit_alignment_3.riv", Some("Artboard"));
    let vmi = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(vmi));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for _ in 0..60 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("image_fit_alignment_3");
}

#[test]
fn wave_b_data_binding_images_test_008_direct_port_expected_red() {
    let fixture = binding_asset_silver("image_fit_alignment_updated_test.riv", Some("Main"));
    let vmi = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(vmi));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for _ in 0..60 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("image_fit_alignment_updated_test");
}

#[test]
fn wave_b_data_binding_images_test_009_direct_port_expected_red() {
    let legacy_bytes =
        std::fs::read(binding_path("assets/image_fit_alignment.riv")).expect("fixture");
    assert!(legacy_bytes.len() > 6);
    assert_eq!(legacy_bytes[0], b'R');
    assert_eq!(legacy_bytes[1], b'I');
    assert_eq!(legacy_bytes[2], b'V');
    assert_eq!(legacy_bytes[3], b'E');
    assert_eq!(legacy_bytes[4], 7);
    assert!(legacy_bytes[5] < 2);
    let mut modern_bytes = legacy_bytes.clone();
    modern_bytes[5] = 2;

    struct Loaded {
        _file: RuntimeFileHandle,
        _artboard: RuntimeArtboardInstanceHandle,
        machine: RuntimeStateMachineInstanceHandle,
        images: Vec<CoreHandle>,
    }
    let load = |bytes: &[u8], factory: &mut PersistentFactory<SerializingFactory>| {
        let retained = RuntimeFactoryHandle::from_factory(factory).expect("retained factory");
        let mut result = ImportResult::Malformed;
        let file =
            File::import(bytes, retained, Some(&mut result), None, None).expect("native File");
        assert_eq!(result, ImportResult::Success);
        let artboard = file
            .with_file(|file| file.artboard_named("Main"))
            .expect("Main");
        let machine = artboard.state_machine_at(0).expect("state machine");
        let id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
        let vmi = file
            .with_file_mut(|file| {
                if id == u32::MAX {
                    file.create_view_model_instance_for_artboard(artboard.core_handle())
                } else {
                    file.create_view_model_instance_at(id as usize, 0)
                }
            })
            .expect("view model instance");
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(vmi));
        machine.advance_and_apply(0.1);
        let images = binding_images(&artboard)
            .into_iter()
            .filter(|image| {
                image
                    .with_downcast::<Image, _>(|image| image.base.parent_handle())
                    .flatten()
                    .is_some_and(|parent| parent.is_type_of(LayoutComponent::TYPE_KEY))
            })
            .collect();
        Loaded {
            _file: file,
            _artboard: artboard,
            machine,
            images,
        }
    };
    let mut legacy_factory = PersistentFactory::new(SerializingFactory::new());
    let mut modern_factory = PersistentFactory::new(SerializingFactory::new());
    let legacy = load(&legacy_bytes, &mut legacy_factory);
    let modern = load(&modern_bytes, &mut modern_factory);
    assert!(!legacy.images.is_empty());
    assert_eq!(legacy.images.len(), modern.images.len());
    let x_axis_scale = |image: &CoreHandle| {
        image
            .with_downcast::<Image, _>(|image| {
                let world = image.base.world_transform();
                Vec2D::new(world[0], world[1]).length()
            })
            .expect("native Image transform")
    };
    let mut pick = legacy.images.len();
    for _ in 0..120 {
        if pick != legacy.images.len() {
            break;
        }
        for (index, image) in legacy.images.iter().enumerate() {
            if x_axis_scale(image) > 1.0 {
                pick = index;
                break;
            }
        }
        if pick != legacy.images.len() {
            break;
        }
        legacy.machine.advance_and_apply(0.016);
        modern.machine.advance_and_apply(0.016);
    }
    assert_ne!(pick, legacy.images.len());
    let legacy_image = &legacy.images[pick];
    let modern_image = &modern.images[pick];
    let user_scale_x = modern_image
        .with_downcast::<Image, _>(|image| image.base.scale_x())
        .unwrap();
    assert!(!binding_asset_approx(user_scale_x, 1.0));
    let legacy_user_scale_x = legacy_image
        .with_downcast::<Image, _>(|image| image.base.scale_x())
        .unwrap();
    assert!(!binding_asset_approx(legacy_user_scale_x, user_scale_x));
    let legacy_scale = x_axis_scale(legacy_image);
    let modern_scale = x_axis_scale(modern_image);
    assert!(legacy_scale > 0.0);
    assert!(binding_asset_approx(
        modern_scale,
        legacy_scale * user_scale_x
    ));
}

#[test]
fn wave_b_data_binding_images_test_010_direct_port_expected_red() {
    let fixture = binding_asset_silver("stateful_component_image_test.riv", None);
    let mut renderer = fixture.silver.borrow().make_renderer();
    let id = fixture
        .artboard
        .with_artboard(|artboard| artboard.base.view_model_id());
    let vmi = fixture
        .file
        .with_file_mut(|file| file.create_view_model_instance_at(id as usize, 0))
        .expect("authored view model instance");
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(vmi.clone()));
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    let bytes = std::fs::read(binding_path("assets/open_source.jpg")).expect("open_source.jpg");
    assert_eq!(bytes.len(), 8880);
    let image: Rc<dyn nuxie_render_api::RenderImage> = Rc::from(
        fixture
            .silver
            .borrow_mut()
            .decode_image(&bytes)
            .expect("factory image decode"),
    );
    let property = binding_property(&vmi, "img");
    binding_set_image(&property, Some(image));
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("stateful_component_image_test");
}

use nuxie_runtime::source::{
    constraints::follow_path_constraint::FollowPathConstraint,
    custom_property_boolean::CustomPropertyBoolean,
    custom_property_number::CustomPropertyNumber,
    custom_property_string::CustomPropertyString,
    generated::viewmodel::viewmodel_instance_trigger_base::ViewModelInstanceTriggerBase,
    shapes::{
        paint::{fill::Fill, solid_color::SolidColor},
        shape::Shape,
    },
    viewmodel::{
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_enum::ViewModelInstanceEnum,
        viewmodel_instance_string::ViewModelInstanceString,
        viewmodel_instance_trigger::ViewModelInstanceTrigger,
    },
};

fn binding_authored_instance(
    asset: &str,
    name: &str,
) -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle, CoreHandle) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file(asset, &mut factory);
    let artboard = file
        .with_file(|file| file.artboard_named(name))
        .expect("authored artboard instance");
    let instance = binding_default_instance(&file, &artboard);
    (file, artboard, instance)
}

fn binding_default_machine(
    artboard: &RuntimeArtboardInstanceHandle,
    instance: &CoreHandle,
) -> RuntimeStateMachineInstanceHandle {
    let machine = artboard
        .default_state_machine_handle()
        .expect("default state machine");
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance.clone()));
    machine
}

fn binding_typed_property<T: CoreType>(instance: &CoreHandle, name: &str) -> CoreHandle {
    let property = binding_property(instance, name);
    assert!(
        property.is_type_of(T::TYPE_KEY),
        "{name} has the pinned property type"
    );
    property
}

fn binding_set_string(property: &CoreHandle, value: &str) {
    property
        .with_downcast_mut::<ViewModelInstanceString, _>(|property| property.set_value(value))
        .expect("string property");
}

fn binding_set_boolean(property: &CoreHandle, value: bool) {
    property
        .with_downcast_mut::<ViewModelInstanceBoolean, _>(|property| property.set_value(value))
        .expect("boolean property");
}

fn binding_set_trigger(property: &CoreHandle, value: u32) {
    CoreRegistry::set_uint_handle(
        property,
        ViewModelInstanceTriggerBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
        value,
    );
}

fn binding_shape_fill(shape: &CoreHandle) -> CoreHandle {
    let fill = shape
        .with_downcast::<Shape, _>(|shape| shape.base.children()[1].clone())
        .expect("Shape");
    assert!(fill.is_type_of(Fill::TYPE_KEY));
    fill
}

fn binding_fill_color(fill: &CoreHandle) -> u32 {
    let paint = fill
        .with_downcast::<Fill, _>(|fill| fill.base.paint())
        .flatten()
        .expect("fill paint");
    assert!(paint.is_type_of(SolidColor::TYPE_KEY));
    paint
        .with_downcast::<SolidColor, _>(|paint| paint.base.color_value() as u32)
        .expect("SolidColor")
}

fn binding_shape_position(shape: &CoreHandle) -> (f32, f32) {
    shape
        .with_downcast::<Shape, _>(|shape| (shape.base.x(), shape.base.y()))
        .expect("Shape")
}

fn binding_rotation(shape: &CoreHandle) -> f32 {
    shape
        .with_downcast::<Shape, _>(|shape| shape.base.rotation())
        .expect("Shape")
}

fn binding_custom_number(property: &CoreHandle) -> f32 {
    property
        .with_downcast::<CustomPropertyNumber, _>(|property| property.base.property_value())
        .expect("CustomPropertyNumber")
}

fn binding_custom_string(property: &CoreHandle) -> String {
    property
        .with_downcast::<CustomPropertyString, _>(|property| {
            property.base.property_value().to_owned()
        })
        .expect("CustomPropertyString")
}

fn binding_custom_boolean(property: &CoreHandle) -> bool {
    property
        .with_downcast::<CustomPropertyBoolean, _>(|property| property.base.property_value())
        .expect("CustomPropertyBoolean")
}

#[test]
fn wave_b_data_binding_test_001_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_authored_instance("data_binding_test.riv", "artboard-1");
    artboard.bind_view_model_instance(Some(instance.clone()));
    artboard.advance_default(0.0);
    let rect = binding_find::<Rectangle>(&artboard, "bound_rect");
    assert_eq!(binding_width(&rect), 100.0);
    let shape = binding_find::<Shape>(&artboard, "bound_rect_shape");
    assert!(binding_asset_approx(binding_rotation(&shape), 1.5708));
    let fill = binding_shape_fill(&shape);
    assert_eq!(binding_fill_color(&fill), 0xffff0000);
    let text = binding_find::<TextValueRun>(&artboard, "bound_text_run");
    assert_eq!(binding_text(&text), "bound text");
    let constraint = binding_find::<FollowPathConstraint>(&artboard, "");
    let orient = || {
        constraint
            .with_downcast::<FollowPathConstraint, _>(|constraint| constraint.base.orient())
            .expect("FollowPathConstraint")
    };
    assert!(!orient());
    let width_property = binding_typed_property::<ViewModelInstanceNumber>(&instance, "width");
    let rotation_property =
        binding_typed_property::<ViewModelInstanceNumber>(&instance, "rotation");
    let color_property = binding_typed_property::<ViewModelInstanceColor>(&instance, "color");
    let text_property = binding_typed_property::<ViewModelInstanceString>(&instance, "text");
    let orient_property = binding_typed_property::<ViewModelInstanceBoolean>(&instance, "orient");
    binding_set_number(&width_property, 200.0);
    binding_set_number(&rotation_property, 180.0);
    binding_set_color(&color_property, 0xff00ff00);
    binding_set_string(&text_property, "New text");
    binding_set_boolean(&orient_property, true);
    artboard.advance_default(0.0);
    assert_eq!(binding_width(&rect), 200.0);
    assert!(binding_asset_approx(binding_rotation(&shape), 3.14159));
    assert_eq!(binding_fill_color(&fill), 0xff00ff00);
    assert_eq!(binding_text(&text), "New text");
    assert!(orient());
}

#[test]
fn wave_b_data_binding_test_002_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_authored_instance("data_binding_test.riv", "artboard-2");
    let machine = binding_default_machine(&artboard, &instance);
    let shape = binding_find::<Shape>(&artboard, "color_rectangle");
    let fill = binding_shape_fill(&shape);
    assert_eq!(binding_shape_position(&shape).0, 250.0);
    assert_eq!(binding_shape_position(&shape).1, 250.0);
    assert_eq!(binding_fill_color(&fill), 0xff747474);
    let state = binding_typed_property::<ViewModelInstanceEnum>(&instance, "state");
    let trigger = binding_typed_property::<ViewModelInstanceTrigger>(&instance, "trigger-prop");
    machine.advance_and_apply(0.0);
    assert_eq!(binding_fill_color(&fill), 0xffff0000);
    state
        .with_downcast_mut::<ViewModelInstanceEnum, _>(|state| state.set_value_at(1))
        .expect("enum");
    machine.advance_and_apply(0.0);
    assert_eq!(binding_fill_color(&fill), 0xff00ff00);
    assert_eq!(binding_shape_position(&shape).0, 150.0);
    assert_eq!(binding_shape_position(&shape).1, 250.0);
    state
        .with_downcast_mut::<ViewModelInstanceEnum, _>(|state| state.set_value_named("state-blue"))
        .expect("enum");
    binding_set_trigger(&trigger, 1);
    machine.advance_and_apply(0.0);
    assert_eq!(binding_fill_color(&fill), 0xff0000ff);
    assert_eq!(binding_shape_position(&shape).0, 350.0);
    assert_eq!(binding_shape_position(&shape).1, 250.0);
    binding_set_trigger(&trigger, 1);
    machine.advance_and_apply(0.0);
    assert_eq!(binding_shape_position(&shape).0, 350.0);
    assert_eq!(binding_shape_position(&shape).1, 350.0);
}

#[test]
fn wave_b_data_binding_test_003_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_authored_instance("data_binding_test.riv", "artboard-2");
    let machine = binding_default_machine(&artboard, &instance);
    let trigger = binding_typed_property::<ViewModelInstanceTrigger>(&instance, "trigger-prop");
    machine.advance_and_apply(0.0);
    binding_set_trigger(&trigger, 1);
    machine.advance_and_apply_view_models(0.0, false);
    assert_eq!(
        trigger
            .with_downcast::<ViewModelInstanceTrigger, _>(|trigger| trigger.base.property_value())
            .unwrap(),
        1
    );
    machine.advance_and_apply_view_models(0.0, true);
    assert_eq!(
        trigger
            .with_downcast::<ViewModelInstanceTrigger, _>(|trigger| trigger.base.property_value())
            .unwrap(),
        0
    );
}

#[test]
fn wave_b_data_binding_test_004_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_authored_instance("data_binding_test.riv", "artboard-3");
    let machine = binding_default_machine(&artboard, &instance);
    let number = binding_find::<CustomPropertyNumber>(&artboard, "num_prop");
    assert_eq!(binding_custom_number(&number), 0.0);
    let text = binding_find::<TextValueRun>(&artboard, "text_run_bound");
    let property = binding_typed_property::<ViewModelInstanceNumber>(&instance, "num1");
    assert_eq!(binding_text(&text), "text");
    machine.advance_and_apply(0.0);
    assert_eq!(binding_custom_number(&number), 34.0);
    assert_eq!(binding_text(&text), "6");
    binding_set_number(&property, -10.0);
    machine.advance_and_apply(0.0);
    assert_eq!(binding_custom_number(&number), -20.0);
    assert_eq!(binding_text(&text), "-3");
}

#[test]
fn wave_b_data_binding_test_005_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_authored_instance("data_binding_test.riv", "artboard-3");
    let machine = binding_default_machine(&artboard, &instance);
    let both = binding_find::<TextValueRun>(&artboard, "second_text_run_trim_both");
    let start = binding_find::<TextValueRun>(&artboard, "second_text_run_trim_start");
    let end = binding_find::<TextValueRun>(&artboard, "second_text_run_trim_end");
    let none = binding_find::<TextValueRun>(&artboard, "second_text_run_no_trim");
    let property = binding_typed_property::<ViewModelInstanceString>(&instance, "text");
    assert_eq!(binding_text(&none), "text");
    assert_eq!(binding_text(&both), "text");
    assert_eq!(binding_text(&start), "text");
    assert_eq!(binding_text(&end), "text");
    machine.advance_and_apply(0.0);
    assert_eq!(binding_text(&both), "abc");
    assert_eq!(binding_text(&none), "     abc    ");
    assert_eq!(binding_text(&start), "abc    ");
    assert_eq!(binding_text(&end), "     abc");
    binding_set_string(&property, "a b c ");
    machine.advance_and_apply(0.0);
    assert_eq!(binding_text(&none), "a b c ");
    assert_eq!(binding_text(&both), "a b c");
    assert_eq!(binding_text(&start), "a b c ");
    assert_eq!(binding_text(&end), "a b c");
}

#[test]
fn wave_b_data_binding_test_006_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_authored_instance("data_binding_test.riv", "artboard-4");
    let machine = binding_default_machine(&artboard, &instance);
    let upper = binding_find::<TextValueRun>(&artboard, "RGBA_formatted_color_run");
    let lower = binding_find::<TextValueRun>(&artboard, "rgba_formatted_color_run");
    let hls = binding_find::<TextValueRun>(&artboard, "hls_formatted_color_run");
    let escaped = binding_find::<TextValueRun>(&artboard, "escaped_characters_run");
    let property = binding_typed_property::<ViewModelInstanceColor>(&instance, "col");
    assert_eq!(binding_text(&upper), "text");
    assert_eq!(binding_text(&lower), "text");
    assert_eq!(binding_text(&hls), "text");
    assert_eq!(binding_text(&escaped), "text");
    machine.advance_and_apply(0.0);
    assert_eq!(
        binding_text(&upper),
        "color: {red: 1E, green: 5A, blue: C8, alpha: FF}"
    );
    assert_eq!(
        binding_text(&lower),
        "color: {red: 30, green: 90, blue: 200, alpha: 255}"
    );
    assert_eq!(
        binding_text(&hls),
        "color: {hue: 219, luminance: 45, saturation: 74}"
    );
    assert_eq!(binding_text(&escaped), "%r %g %b %a \\a");
    binding_set_color(&property, 0x64c86432);
    machine.advance_and_apply(0.0);
    assert_eq!(
        binding_text(&upper),
        "color: {red: C8, green: 64, blue: 32, alpha: 64}"
    );
    assert_eq!(
        binding_text(&lower),
        "color: {red: 200, green: 100, blue: 50, alpha: 100}"
    );
    assert_eq!(
        binding_text(&hls),
        "color: {hue: 20, luminance: 49, saturation: 60}"
    );
    assert_eq!(binding_text(&escaped), "%r %g %b %a \\a");
    binding_set_color(&property, 0x64000a0f);
    machine.advance_and_apply(0.0);
    assert_eq!(
        binding_text(&upper),
        "color: {red: 00, green: 0A, blue: 0F, alpha: 64}"
    );
    assert_eq!(
        binding_text(&lower),
        "color: {red: 0, green: 10, blue: 15, alpha: 100}"
    );
    assert_eq!(
        binding_text(&hls),
        "color: {hue: 200, luminance: 3, saturation: 100}"
    );
    assert_eq!(binding_text(&escaped), "%r %g %b %a \\a");
}

#[test]
fn wave_b_data_binding_test_007_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_authored_instance("data_binding_test_2.riv", "artboard-2");
    let machine = binding_default_machine(&artboard, &instance);
    machine.advance_and_apply(0.0);
    let one = binding_find::<CustomPropertyNumber>(&artboard, "mapped-range-1");
    assert_eq!(binding_custom_number(&one), 6.0);
    let two = binding_find::<CustomPropertyNumber>(&artboard, "mapped-range-2");
    assert_eq!(binding_custom_number(&two), 3.0);
    let three = binding_find::<CustomPropertyNumber>(&artboard, "mapped-range-3");
    assert_eq!(binding_custom_number(&three), 2.0);
    let four = binding_find::<CustomPropertyNumber>(&artboard, "mapped-range-4");
    assert_eq!(binding_custom_number(&four), 2.0);
    let five = binding_find::<CustomPropertyNumber>(&artboard, "mapped-range-5");
    assert_eq!(binding_custom_number(&five), 2.0);
    let property = binding_typed_property::<ViewModelInstanceNumber>(&instance, "map-range-num");
    assert_eq!(
        property
            .with_downcast::<ViewModelInstanceNumber, _>(|property| property.base.property_value())
            .unwrap(),
        4.0
    );
    binding_set_number(&property, -1.0);
    machine.advance_and_apply(0.0);
    assert_eq!(binding_custom_number(&one), 1.0);
    assert_eq!(binding_custom_number(&two), 2.0);
    assert_eq!(binding_custom_number(&three), 2.0);
    assert_eq!(binding_custom_number(&four), 3.0);
    assert_eq!(binding_custom_number(&five), 2.0);
    binding_set_number(&property, 0.0);
    machine.advance_and_apply(0.0);
    assert_eq!(binding_custom_number(&one), 2.0);
    assert_eq!(binding_custom_number(&two), 2.0);
    assert_eq!(binding_custom_number(&three), 2.0);
    assert_eq!(binding_custom_number(&four), 3.0);
    assert_eq!(binding_custom_number(&five), 2.0);
    binding_set_number(&property, 0.25);
    machine.advance_and_apply(0.0);
    assert!(binding_asset_approx(binding_custom_number(&one), 2.12916));
    assert!(binding_asset_approx(binding_custom_number(&two), 2.12916));
    assert!(binding_asset_approx(binding_custom_number(&three), 2.12916));
    assert!(binding_asset_approx(binding_custom_number(&four), 2.87084));
    assert_eq!(binding_custom_number(&five), 2.0);
    binding_set_number(&property, 2.0);
    machine.advance_and_apply(0.0);
    assert_eq!(binding_custom_number(&one), 4.0);
    assert_eq!(binding_custom_number(&two), 3.0);
    assert_eq!(binding_custom_number(&three), 2.0);
    assert_eq!(binding_custom_number(&four), 2.0);
    assert_eq!(binding_custom_number(&five), 2.0);
    binding_set_number(&property, 2.25);
    machine.advance_and_apply(0.0);
    assert_eq!(binding_custom_number(&one), 4.25);
    assert_eq!(binding_custom_number(&two), 3.0);
    assert!(binding_asset_approx(binding_custom_number(&three), 2.12916));
    assert_eq!(binding_custom_number(&four), 2.0);
    assert_eq!(binding_custom_number(&five), 2.0);
}

#[test]
fn wave_b_data_binding_test_008_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_authored_instance("data_binding_test_2.riv", "artboard-3");
    let machine = binding_default_machine(&artboard, &instance);
    machine.advance_and_apply(0.0);
    let one = binding_find::<CustomPropertyString>(&artboard, "pad-string-1");
    assert_eq!(binding_custom_string(&one), "abcabcatext");
    let two = binding_find::<CustomPropertyString>(&artboard, "pad-string-2");
    assert_eq!(binding_custom_string(&two), "textabcabcab");
    let three = binding_find::<CustomPropertyString>(&artboard, "pad-string-3");
    assert_eq!(binding_custom_string(&three), "");
    let property = binding_typed_property::<ViewModelInstanceString>(&instance, "pad-string");
    assert_eq!(
        property
            .with_downcast::<ViewModelInstanceString, _>(|property| property
                .base
                .property_value()
                .to_owned())
            .unwrap(),
        "text"
    );
    binding_set_string(&property, "text-text-text");
    machine.advance_and_apply(0.0);
    assert_eq!(binding_custom_string(&one), "text-text-text");
    assert_eq!(binding_custom_string(&two), "text-text-text");
    assert_eq!(binding_custom_string(&three), "");
    binding_set_string(&property, "");
    machine.advance_and_apply(0.0);
    assert_eq!(binding_custom_string(&one), "abcabcabcab");
    assert_eq!(binding_custom_string(&two), "abcabcabcabc");
    assert_eq!(binding_custom_string(&three), "");
}

#[test]
fn wave_b_data_binding_test_009_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_authored_instance("data_binding_test_2.riv", "artboard-3");
    let machine = binding_default_machine(&artboard, &instance);
    machine.advance_and_apply(0.0);
    let custom = binding_find::<CustomPropertyBoolean>(&artboard, "negate-bool-1");
    assert!(binding_custom_boolean(&custom));
    let property = binding_typed_property::<ViewModelInstanceBoolean>(&instance, "bool-prop");
    assert!(
        !property
            .with_downcast::<ViewModelInstanceBoolean, _>(|property| property.base.property_value())
            .unwrap()
    );
    binding_set_boolean(&property, true);
    machine.advance_and_apply(0.0);
    assert!(!binding_custom_boolean(&custom));
}

#[test]
fn wave_b_data_binding_test_010_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_authored_instance("shared_viewmodel_instance.riv", "main");
    let machine = binding_default_machine(&artboard, &instance);
    machine.advance_and_apply(0.0);
    let child_property = binding_typed_property::<ViewModelInstanceViewModel>(&instance, "child1");
    let child_instance = child_property
        .with_downcast::<ViewModelInstanceViewModel, _>(|property| {
            property.reference_view_model_instance()
        })
        .flatten()
        .expect("referenced view model");
    let label = binding_typed_property::<ViewModelInstanceString>(&child_instance, "label");
    let child1 = binding_child(&artboard, "child1");
    let text1 = binding_find::<TextValueRun>(&child1, "text_run");
    assert_eq!(binding_text(&text1), "label-vmi-1");
    let child2 = binding_child(&artboard, "child2");
    let text2 = binding_find::<TextValueRun>(&child2, "text_run");
    assert_eq!(binding_text(&text2), "label-vmi-1");
    binding_set_string(&label, "label-update");
    machine.advance_and_apply(0.0);
    assert_eq!(binding_text(&text1), "label-update");
    assert_eq!(binding_text(&text2), "label-update");
}

use nuxie_runtime::source::{
    custom_property_trigger::CustomPropertyTrigger, data_bind::data_values::data_type::DataType,
};

// The fresh-instance cases deliberately do not clone an authored default.
fn binding_fresh_instance(
    asset: &str,
    name: &str,
) -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle, CoreHandle) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file(asset, &mut factory);
    let artboard = file
        .with_file(|file| file.artboard_named(name))
        .expect("authored artboard instance");
    let instance = file
        .with_file_mut(|file| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        .expect("new view model instance");
    (file, artboard, instance)
}

fn binding_reference_instance(property: &CoreHandle) -> CoreHandle {
    property
        .with_downcast::<ViewModelInstanceViewModel, _>(|property| {
            property.reference_view_model_instance()
        })
        .flatten()
        .expect("referenced view model instance")
}

fn binding_trigger(property: &CoreHandle) {
    property
        .with_downcast_mut::<ViewModelInstanceTrigger, _>(|property| property.trigger())
        .expect("trigger property");
}

fn binding_list_item(list: &CoreHandle, instance: Option<CoreHandle>) -> CoreHandle {
    let mut item = ViewModelInstanceListItem::default();
    item.set_view_model_instance(instance);
    list.insert_sibling(item)
        .expect("native list item allocation")
}

fn binding_list_add(list: &CoreHandle, item: CoreHandle) {
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item(item))
        .expect("list");
}

fn binding_list_add_at(list: &CoreHandle, item: CoreHandle, index: i32) {
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.add_item_at(item, index))
        .expect("list");
}

fn binding_new_named_instance(file: &RuntimeFileHandle, name: &str) -> CoreHandle {
    let model = file
        .with_file(|file| file.view_model_named(name))
        .expect("view model");
    file.with_file_mut(|file| file.create_view_model_instance(model))
        .expect("fresh named view model instance")
}

#[test]
fn wave_b_data_binding_test_011_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_authored_instance("shared_viewmodel_instance.riv", "main_2");
    let machine = binding_default_machine(&artboard, &instance);
    machine.advance_and_apply(0.0);
    let child = binding_typed_property::<ViewModelInstanceViewModel>(&instance, "vm_2_child1");
    let referenced = binding_reference_instance(&child);
    let label = binding_typed_property::<ViewModelInstanceString>(&referenced, "label");
    let child1 = binding_child(&artboard, "child1");
    let text1 = binding_find::<TextValueRun>(&child1, "text_run");
    assert_eq!(binding_text(&text1), "label-vmi-1");
    let child2 = binding_child(&artboard, "child2");
    let text2 = binding_find::<TextValueRun>(&child2, "text_run");
    assert_eq!(binding_text(&text2), "label-vmi-2");
    binding_set_string(&label, "label-update");
    machine.advance_and_apply(0.0);
    assert_eq!(binding_text(&text1), "label-update");
    assert_eq!(binding_text(&text2), "label-vmi-2");
}

#[test]
fn wave_b_data_binding_test_012_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_fresh_instance("shared_viewmodel_instance.riv", "main_2");
    let machine = binding_default_machine(&artboard, &instance);
    machine.advance_and_apply(0.0);
    let child = binding_typed_property::<ViewModelInstanceViewModel>(&instance, "vm_2_child1");
    let referenced = binding_reference_instance(&child);
    let label = binding_typed_property::<ViewModelInstanceString>(&referenced, "label");
    let child1 = binding_child(&artboard, "child1");
    let text1 = binding_find::<TextValueRun>(&child1, "text_run");
    assert_eq!(binding_text(&text1), "");
    let child2 = binding_child(&artboard, "child2");
    let text2 = binding_find::<TextValueRun>(&child2, "text_run");
    assert_eq!(binding_text(&text2), "");
    binding_set_string(&label, "label-update");
    machine.advance_and_apply(0.0);
    assert_eq!(binding_text(&text1), "label-update");
    assert_eq!(binding_text(&text2), "");
}

#[test]
fn wave_b_data_binding_test_013_direct_port_expected_red() {
    let (_file, artboard, instance) =
        binding_fresh_instance("data_binding_test_triggers.riv", "root");
    let machine = binding_default_machine(&artboard, &instance);
    machine.advance_and_apply(0.0);
    let shape = binding_find::<Shape>(&artboard, "main_rect");
    let fill = binding_shape_fill(&shape);
    assert_eq!(binding_fill_color(&fill), 0xffff0000);
    machine.advance_and_apply(0.7);
    machine.advance_and_apply(0.1);
    assert_eq!(binding_fill_color(&fill), 0xff00ff00);
}

#[test]
fn wave_b_data_binding_test_014_direct_port_expected_red() {
    let fixture = binding_asset_silver("transition_self_comparator_test.riv", None);
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance.clone()));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let number = binding_typed_property::<ViewModelInstanceNumber>(&instance, "num");
    let trigger = binding_typed_property::<ViewModelInstanceTrigger>(&instance, "tri");
    let color = binding_typed_property::<ViewModelInstanceColor>(&instance, "col");
    let boolean = binding_typed_property::<ViewModelInstanceBoolean>(&instance, "bol");
    let string = binding_typed_property::<ViewModelInstanceString>(&instance, "str");
    let list = binding_typed_property::<ViewModelInstanceList>(&instance, "lis");

    fixture.silver.borrow_mut().add_frame();
    binding_set_number(&number, 20.0);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_number(&number, 20.0);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_number(&number, 10.0);
    binding_set_number(&number, 20.0);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_number(&number, 10.0);
    binding_trigger(&trigger);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_color(&color, 0x64000a0f);
    binding_set_color(&color, 0x65000a0f);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_color(&color, 0x66000a0f);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_boolean(&boolean, true);
    binding_set_boolean(&boolean, false);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_boolean(&boolean, true);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_string(&string, "a");
    binding_set_string(&string, "b");
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_string(&string, "c");
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);

    fixture.silver.borrow_mut().add_frame();
    let item1 = binding_list_item(&list, None);
    binding_list_add(&list, item1);
    let item2 = binding_list_item(&list, None);
    binding_list_add(&list, item2);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    let item = binding_list_item(&list, None);
    binding_list_add(&list, item);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    let item = binding_list_item(&list, None);
    binding_list_add_at(&list, item, 0);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    let item = binding_list_item(&list, None);
    binding_list_add_at(&list, item, 10);
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.swap(0, 1))
        .expect("list");
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.remove_item_at(0))
        .expect("list");
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    list.with_downcast_mut::<ViewModelInstanceList, _>(|list| list.remove_item_at(10))
        .expect("list");
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("transition_self_comparator_test");
}

#[test]
fn wave_b_data_binding_test_015_direct_port_expected_red() {
    let fixture = binding_asset_silver("computed_root_transform.riv", Some("nested-artboard-main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let frames = (1.0_f32 / 0.016_f32) as i32;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("computed_root_transform-nested_artboard");
}

#[test]
fn wave_b_data_binding_test_016_direct_port_expected_red() {
    let fixture = binding_asset_silver("computed_root_transform.riv", Some("list-main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let frames = (1.0_f32 / 0.016_f32) as i32;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("computed_root_transform-list");
}

#[test]
fn wave_b_data_binding_test_017_direct_port_expected_red() {
    let fixture = binding_asset_silver("trigger_based_listeners.riv", Some("main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(25.0, 25.0), 0));
    fixture.advance(0.1);
    fixture.advance(1.0);
    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_up(Vec2D::new(25.0, 25.0), 0));
    fixture.advance(0.1);
    fixture.advance(1.0);
    fixture.silver.borrow_mut().add_frame();
    fixture.artboard.draw(&mut renderer);
    fixture.matches("trigger_based_listeners");
}

#[test]
fn wave_b_data_binding_test_018_direct_port_expected_red() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let file = binding_file("custom_property_trigger.riv", &mut silver);
    let artboard = file
        .with_file(|file| file.artboard_named("Main"))
        .expect("Main artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let instance = binding_default_instance(&file, &artboard);
    let machine = binding_default_machine(&artboard, &instance);
    let fixture = BindingSilver {
        machine,
        artboard,
        file,
        silver,
    };
    fixture.advance(0.0);
    let circle = binding_find::<Shape>(&fixture.artboard, "MainCircle");
    assert_eq!(
        circle
            .with_downcast::<Shape, _>(|shape| shape.base.scale_x())
            .unwrap(),
        1.0
    );
    assert_eq!(
        circle
            .with_downcast::<Shape, _>(|shape| shape.base.scale_y())
            .unwrap(),
        1.0
    );
    let _trigger = binding_find::<CustomPropertyTrigger>(&fixture.artboard, "Trig");
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let frames = (1.0_f32 / 0.16_f32) as i32;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.16);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("custom_property_trigger_bind");
}

#[test]
fn wave_b_data_binding_test_019_direct_port_expected_red() {
    let fixture = binding_asset_silver("data_bind_solo.riv", Some("values-to-solos"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let frames = (1.0_f32 / 0.016_f32) as i32;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("data_bind_solo-values-to-solos");
}

#[test]
fn wave_b_data_binding_test_020_direct_port_expected_red() {
    let fixture = binding_asset_silver("data_bind_solo.riv", Some("solos-to-values"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let frames = (1.0_f32 / 0.016_f32) as i32;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("data_bind_solo-solos-to-values");
}

#[test]
fn wave_b_data_binding_test_021_direct_port_expected_red() {
    let fixture = binding_asset_silver("state_transition_fire_trigger.riv", Some("main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.advance(0.1);
    fixture.advance(1.0);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.advance(1.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.advance(1.0);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("state_transition_fire_trigger");
}

#[test]
fn wave_b_data_binding_test_022_direct_port_expected_red() {
    let fixture = binding_asset_silver("custom_property_enum.riv", Some("main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let frames = (3.0_f32 / 0.048_f32) as i32;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.048);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("custom_property_enum");
}

#[test]
fn wave_b_data_binding_test_023_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("viewmodel_runtime_file.riv", &mut factory);
    let model = file
        .with_file(|file| file.view_model_by_name("vm"))
        .expect("runtime view model");
    let instance = model.create_default_instance();
    assert_eq!(instance.view_model_name(), "vm");
    let number = instance.property_number("num").expect("number");
    assert_eq!(number.data_type(), DataType::Number);
    let string = instance.property_string("str").expect("string");
    assert_eq!(string.data_type(), DataType::String);
    assert!(instance.property_number("str").is_none());
    let boolean = instance.property_boolean("boo").expect("boolean");
    assert_eq!(boolean.data_type(), DataType::Boolean);
    let color = instance.property_color("col").expect("color");
    assert_eq!(color.data_type(), DataType::Color);
    let trigger = instance.property_trigger("tri").expect("trigger");
    assert_eq!(trigger.data_type(), DataType::Trigger);
    let enumeration = instance.property_enum("enu").expect("enum");
    assert_eq!(enumeration.data_type(), DataType::Enum);
    let image = instance.property_image("ima").expect("image");
    assert_eq!(image.data_type(), DataType::AssetImage);
    let artboard = instance.property_artboard("art").expect("artboard");
    assert_eq!(artboard.data_type(), DataType::Artboard);
    let list = instance.property_list("lis").expect("list");
    assert_eq!(list.data_type(), DataType::List);
    let nested_number = instance
        .property_number("chi/chi-num")
        .expect("nested number");
    assert_eq!(nested_number.data_type(), DataType::Number);
    let properties = instance.properties();
    let enum_data = properties
        .iter()
        .find(|property| property.name == "enu")
        .expect("enum property data");
    assert_eq!(enum_data.data_type, DataType::Enum);
    assert_eq!(enum_data.enum_name, "Horizontal Align");
    let number_data = properties
        .iter()
        .find(|property| property.name == "num")
        .expect("number property data");
    assert_eq!(number_data.data_type, DataType::Number);
    assert!(number_data.enum_name.is_empty());
}

#[test]
fn wave_b_data_binding_test_024_direct_port_expected_red() {
    let fixture = binding_asset_silver("trigger_fires_single_change.riv", Some("main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for _ in 0..2 {
        fixture
            .machine
            .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(225.0, 275.0), 0));
        fixture.advance(0.1);
        fixture.advance(1.0);
        fixture
            .machine
            .with_instance_mut(|machine| machine.pointer_up(Vec2D::new(225.0, 275.0), 0));
        fixture.advance(0.1);
        fixture.advance(1.0);
        fixture.silver.borrow_mut().add_frame();
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("trigger_fires_single_change");
}

#[test]
fn wave_b_data_binding_test_025_direct_port_expected_red() {
    let fixture = binding_asset_silver("data_converter_to_number.riv", Some("main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let frames = (1.2_f32 / 0.016_f32) as i32;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("data_converter_to_number");
}

#[test]
fn wave_b_data_binding_test_026_direct_port_expected_red() {
    let fixture = binding_asset_silver("list_to_path.riv", Some("main"));
    let instance = fixture.instance();
    let list = binding_typed_property::<ViewModelInstanceList>(&instance, "lis");
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);

    fixture.silver.borrow_mut().add_frame();
    let vertex1 = binding_new_named_instance(&fixture.file, "vertex-x-y");
    let item1 = binding_list_item(&list, Some(vertex1.clone()));
    binding_list_add(&list, item1);
    let vertex2 = binding_new_named_instance(&fixture.file, "vertex-x-y");
    binding_set_number(&binding_property(&vertex2, "x"), 100.0);
    let item2 = binding_list_item(&list, Some(vertex2));
    binding_list_add(&list, item2);
    let vertex3 = binding_new_named_instance(&fixture.file, "vertex-x-y");
    binding_set_number(&binding_property(&vertex3, "x"), 100.0);
    binding_set_number(&binding_property(&vertex3, "y"), 100.0);
    let item3 = binding_list_item(&list, Some(vertex3));
    binding_list_add(&list, item3);
    let vertex4 = binding_new_named_instance(&fixture.file, "vertex-x-y");
    binding_set_number(&binding_property(&vertex4, "y"), 100.0);
    let item4 = binding_list_item(&list, Some(vertex4));
    binding_list_add(&list, item4);
    fixture.advance(0.0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);

    fixture.silver.borrow_mut().add_frame();
    let mirrored = binding_new_named_instance(&fixture.file, "vertex-rotation-distance");
    binding_set_number(&binding_property(&mirrored, "x"), 200.0);
    binding_set_number(&binding_property(&mirrored, "rotation"), 1.5);
    binding_set_number(&binding_property(&mirrored, "distance"), 20.0);
    let item = binding_list_item(&list, Some(mirrored.clone()));
    binding_list_add_at(&list, item, 2);
    fixture.advance(0.0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);

    fixture.silver.borrow_mut().add_frame();
    let detached = binding_new_named_instance(&fixture.file, "vertex-detached");
    binding_set_number(&binding_property(&detached, "x"), 200.0);
    binding_set_number(&binding_property(&detached, "y"), 100.0);
    binding_set_number(&binding_property(&detached, "inRotation"), 1.0);
    binding_set_number(&binding_property(&detached, "outRotation"), 2.0);
    binding_set_number(&binding_property(&detached, "inDistance"), 10.0);
    binding_set_number(&binding_property(&detached, "outDistance"), 30.0);
    let item = binding_list_item(&list, Some(detached.clone()));
    binding_list_add_at(&list, item, 3);
    fixture.advance(0.0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);

    fixture.silver.borrow_mut().add_frame();
    let in_out = binding_new_named_instance(&fixture.file, "vertex-in-out");
    binding_set_number(&binding_property(&in_out, "x"), 100.0);
    binding_set_number(&binding_property(&in_out, "y"), 200.0);
    binding_set_number(&binding_property(&in_out, "inX"), 40.0);
    binding_set_number(&binding_property(&in_out, "inY"), 20.0);
    binding_set_number(&binding_property(&in_out, "outX"), 10.0);
    binding_set_number(&binding_property(&in_out, "outY"), 30.0);
    let item = binding_list_item(&list, Some(in_out.clone()));
    binding_list_add_at(&list, item, 4);
    fixture.advance(0.0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);

    fixture.silver.borrow_mut().add_frame();
    let non_vertex = binding_new_named_instance(&fixture.file, "non-vertex");
    let item = binding_list_item(&list, Some(non_vertex));
    binding_list_add_at(&list, item, 5);
    fixture.advance(0.0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);

    fixture.silver.borrow_mut().add_frame();
    let incomplete = binding_new_named_instance(&fixture.file, "vertex-incomplete");
    binding_set_number(&binding_property(&incomplete, "x"), 100.0);
    binding_set_number(&binding_property(&incomplete, "y"), 300.0);
    binding_set_number(&binding_property(&incomplete, "inDistance"), 60.0);
    binding_set_number(&binding_property(&incomplete, "inRotation"), -1.0);
    binding_set_number(&binding_property(&incomplete, "outX"), 30.0);
    binding_set_number(&binding_property(&incomplete, "inX"), -30.0);
    let item = binding_list_item(&list, Some(incomplete.clone()));
    binding_list_add_at(&list, item, 4);
    fixture.advance(0.0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);

    fixture.silver.borrow_mut().add_frame();
    binding_set_number(&binding_property(&incomplete, "inX"), -30.0);
    binding_set_number(&binding_property(&vertex1, "x"), 50.0);
    binding_set_number(&binding_property(&mirrored, "rotation"), 1.0);
    binding_set_number(&binding_property(&detached, "inDistance"), 30.0);
    binding_set_number(&binding_property(&in_out, "outY"), 40.0);
    fixture.advance(0.0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    for i in 0..60 {
        fixture.silver.borrow_mut().add_frame();
        binding_set_number(&binding_property(&incomplete, "inRotation"), i as f32 * 6.0);
        binding_set_number(&binding_property(&mirrored, "rotation"), i as f32 * 6.0);
        fixture.advance(0.01);
        fixture.advance(0.0);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("list_to_path");
}

#[test]
fn wave_b_data_binding_test_027_direct_port_expected_red() {
    let fixture = binding_asset_silver("format_number_with_commas.riv", Some("main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.matches("format_number_with_commas");
}

#[test]
fn wave_b_data_binding_test_028_direct_port_expected_red() {
    let fixture = binding_asset_silver("time_based_interpolation.riv", Some("main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);

    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(25.0, 25.0), 0));
    fixture.advance(0.016);
    fixture.advance(0.016);
    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_up(Vec2D::new(25.0, 25.0), 0));
    fixture.advance(0.016);
    fixture.advance(0.016);
    fixture.silver.borrow_mut().add_frame();
    fixture.artboard.draw(&mut renderer);
    let frames = (1.0_f32 / 0.032_f32) as i32;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.032);
        fixture.artboard.draw(&mut renderer);
    }

    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(425.0, 25.0), 0));
    fixture.advance(0.016);
    fixture.advance(0.016);
    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_up(Vec2D::new(425.0, 25.0), 0));
    fixture.advance(0.016);
    fixture.advance(0.016);
    fixture.silver.borrow_mut().add_frame();
    fixture.artboard.draw(&mut renderer);
    for _ in 0..10 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.032);
        fixture.artboard.draw(&mut renderer);
    }

    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(25.0, 25.0), 0));
    fixture.advance(0.016);
    fixture.advance(0.016);
    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_up(Vec2D::new(25.0, 25.0), 0));
    fixture.advance(0.016);
    fixture.advance(0.016);
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.032);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("time_based_interpolation");
}

#[test]
fn wave_b_data_binding_test_029_direct_port_expected_red() {
    let fixture = binding_asset_silver("bidirectional_precedence.riv", Some("source_first"));
    let instance = fixture.instance();
    let x = binding_typed_property::<ViewModelInstanceNumber>(&instance, "x");
    let y = binding_typed_property::<ViewModelInstanceNumber>(&instance, "y");
    binding_set_number(&x, 100.0);
    binding_set_number(&y, 100.0);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.matches("bidirectional_precedence-source_first");
}

#[test]
fn wave_b_data_binding_test_030_direct_port_expected_red() {
    let fixture = binding_asset_silver("bidirectional_precedence.riv", Some("target_first"));
    let instance = fixture.instance();
    let x = binding_typed_property::<ViewModelInstanceNumber>(&instance, "x");
    let y = binding_typed_property::<ViewModelInstanceNumber>(&instance, "y");
    binding_set_number(&x, 100.0);
    binding_set_number(&y, 100.0);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.0);
    fixture.advance(0.016);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.matches("bidirectional_precedence-target_first");
}

use nuxie_runtime::source::{
    generated::node_base::NodeBase, lua::scripting_vm::RuntimeScriptingVmHandle, node::Node,
};
use nuxie_scripting::vm::{ScriptExecutionLimits, ScriptVm};

fn binding_authored_index_instance(
    file: &RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
) -> CoreHandle {
    let id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
    file.with_file(|file| file.create_view_model_instance_at(id as usize, 0))
        .expect("authored view model instance 0")
}

fn binding_default_named_instance(file: &RuntimeFileHandle, name: &str) -> CoreHandle {
    let model = file
        .with_file(|file| file.view_model_named(name))
        .expect("named view model");
    file.with_file_mut(|file| file.create_default_view_model_instance(model))
        .expect("default named view model instance")
}

fn binding_precedence_fixture(
    name: &str,
) -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
    CoreHandle,
) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("bidirectional_precedence.riv", &mut factory);
    let artboard = file
        .with_file(|file| file.artboard_named(name))
        .expect("precedence artboard");
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
    let instance = file
        .with_file_mut(|file| {
            if id == u32::MAX {
                file.create_view_model_instance_for_artboard(artboard.core_handle())
            } else {
                file.create_view_model_instance_at(id as usize, 0)
            }
        })
        .expect("view model instance");
    (file, artboard, machine, instance)
}

fn binding_first_node_target(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    let binds = artboard.with_artboard(|artboard| artboard.data_bind_handles());
    binds
        .into_iter()
        .find_map(|bind| {
            let target = bind
                .with(|bind| bind.as_data_bind().expect("DataBind or derived").target())
                .flatten()?;
            target.is_type_of(Node::TYPE_KEY).then_some(target)
        })
        .expect("a data bind targets a Node")
}

fn binding_number_value(property: &CoreHandle) -> f32 {
    property
        .with_downcast::<ViewModelInstanceNumber, _>(|property| property.base.property_value())
        .expect("number property")
}

fn binding_node_position(node: &CoreHandle) -> (f32, f32) {
    (
        CoreRegistry::get_double_handle(node, NodeBase::X_PROPERTY_KEY.into()).expect("Node x"),
        CoreRegistry::get_double_handle(node, NodeBase::Y_PROPERTY_KEY.into()).expect("Node y"),
    )
}

fn binding_scripted_silver(asset: &str, artboard_name: &str) -> BindingSilver {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let vm = RuntimeScriptingVmHandle::new(Box::new(
        ScriptVm::new_with_execution_limits(ScriptExecutionLimits::default())
            .expect("native script VM"),
    ));
    let bytes = std::fs::read(binding_path(&format!("assets/{asset}"))).expect("script fixture");
    // The native importer verifies the fixture's production signature; no
    // sample-key or unsigned-script admission is enabled here.
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut silver).expect("retained factory"),
        None,
        None,
        Some(vm),
    )
    .expect("native scripted File");
    let artboard = file
        .with_file(|file| file.artboard_named(artboard_name))
        .expect("scripted artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    BindingSilver {
        machine,
        artboard,
        file,
        silver,
    }
}

#[test]
fn wave_b_data_binding_test_031_direct_port_expected_red() {
    let (_file, artboard, machine, instance) = binding_precedence_fixture("target_first");
    let x = binding_typed_property::<ViewModelInstanceNumber>(&instance, "x");
    let y = binding_typed_property::<ViewModelInstanceNumber>(&instance, "y");
    binding_set_number(&x, 100.0);
    binding_set_number(&y, 100.0);
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    machine.advance_and_apply(0.0);
    for _ in 0..10 {
        machine.advance_and_apply(0.016);
    }
    let target = binding_first_node_target(&artboard);
    assert_eq!(binding_number_value(&x), binding_node_position(&target).0);
    assert_eq!(binding_number_value(&y), binding_node_position(&target).1);
    binding_set_number(&x, 500.0);
    binding_set_number(&y, 600.0);
    for _ in 0..20 {
        machine.advance_and_apply(0.016);
    }
    assert_eq!(binding_number_value(&x), 500.0);
    assert_eq!(binding_number_value(&y), 600.0);
    assert_eq!(binding_node_position(&target).0, 500.0);
    assert_eq!(binding_node_position(&target).1, 600.0);
}

#[test]
fn wave_b_data_binding_test_032_direct_port_expected_red() {
    let fixture = binding_asset_silver("databind_artboard.riv", None);
    let instance = binding_authored_index_instance(&fixture.file, &fixture.artboard);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_click(&fixture.machine, Vec2D::new(247.0, 332.0));
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("databind_artboard");
}

#[test]
fn wave_b_data_binding_test_033_direct_port_expected_red() {
    let fixture = binding_asset_silver("relative_data_binding.riv", None);
    let instance = binding_authored_index_instance(&fixture.file, &fixture.artboard);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.matches("relative_data_binding");
}

#[test]
fn wave_b_data_binding_test_034_direct_port_expected_red() {
    let fixture = binding_asset_silver("relative_data_bind_path.riv", None);
    let mut renderer = fixture.silver.borrow().make_renderer();
    {
        let instance = binding_authored_index_instance(&fixture.file, &fixture.artboard);
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
    }
    {
        let instance = binding_default_named_instance(&fixture.file, "ViewModel1");
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
    }
    {
        let instance = binding_default_named_instance(&fixture.file, "ViewModel2");
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("relative_data_bind_path");
}

#[test]
fn wave_b_data_binding_test_035_direct_port_expected_red() {
    let fixture = binding_asset_silver("relative_data_bind_path.riv", Some("listener"));
    let mut renderer = fixture.silver.borrow().make_renderer();
    {
        let instance = binding_authored_index_instance(&fixture.file, &fixture.artboard);
        let number = binding_typed_property::<ViewModelInstanceNumber>(&instance, "num");
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
        binding_set_number(&number, 100.0);
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
    }
    {
        let instance = binding_default_named_instance(&fixture.file, "SML_VM2");
        let number = binding_typed_property::<ViewModelInstanceNumber>(&instance, "num");
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
        binding_set_number(&number, 100.0);
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("relative_data_bind_path-listener");
}

#[test]
fn wave_b_data_binding_test_036_direct_port_expected_red() {
    let fixture = binding_asset_silver("relative_data_bind_path.riv", Some("fire-trigger"));
    let mut renderer = fixture.silver.borrow().make_renderer();
    {
        let instance = binding_authored_index_instance(&fixture.file, &fixture.artboard);
        let reset = binding_typed_property::<ViewModelInstanceTrigger>(&instance, "reset");
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
        binding_trigger(&reset);
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
    }
    {
        let instance = binding_default_named_instance(&fixture.file, "SMFT-VM2");
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("relative_data_bind_path-fire-trigger");
}

#[test]
fn wave_b_data_binding_test_037_direct_port_expected_red() {
    let fixture = binding_scripted_silver("relative_data_bind_path.riv", "scripted-input");
    let mut renderer = fixture.silver.borrow().make_renderer();
    {
        let instance = binding_authored_index_instance(&fixture.file, &fixture.artboard);
        let child = binding_typed_property::<ViewModelInstanceViewModel>(&instance, "child");
        let child_instance = binding_reference_instance(&child);
        let boolean = binding_typed_property::<ViewModelInstanceBoolean>(&child_instance, "boo");
        let paused = binding_typed_property::<ViewModelInstanceBoolean>(&child_instance, "paused");
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
        binding_set_boolean(&paused, false);
        fixture.advance(1.0);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
        binding_set_boolean(&paused, true);
        binding_set_boolean(&boolean, false);
        fixture.advance(1.0);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
    }
    {
        let instance = binding_default_named_instance(&fixture.file, "SI-VM2");
        let child = binding_typed_property::<ViewModelInstanceViewModel>(&instance, "child");
        let child_instance = binding_reference_instance(&child);
        let boolean = binding_typed_property::<ViewModelInstanceBoolean>(&child_instance, "boo");
        let paused = binding_typed_property::<ViewModelInstanceBoolean>(&child_instance, "paused");
        fixture
            .machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
        binding_set_boolean(&paused, false);
        fixture.advance(1.0);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
        binding_set_boolean(&paused, true);
        binding_set_boolean(&boolean, false);
        fixture.advance(1.0);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("relative_data_bind_path-scripted-input");
}

#[test]
fn wave_b_data_binding_test_038_direct_port_expected_red() {
    let fixture = binding_asset_silver("listener_view_model.riv", None);
    let instance = fixture.instance();
    let color = binding_typed_property::<ViewModelInstanceColor>(&instance, "col");
    let trigger = binding_typed_property::<ViewModelInstanceTrigger>(&instance, "tri");
    let number = binding_typed_property::<ViewModelInstanceNumber>(&instance, "num1");
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_color(&color, 0x64000a0f);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_trigger(&trigger);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_number(&number, 55.0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("listener_view_model");
}

#[test]
fn wave_b_data_binding_test_039_direct_port_expected_red() {
    let fixture = binding_asset_silver("artboard_width_test.riv", None);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("artboard_width_test");
}

#[test]
fn wave_b_data_binding_test_040_direct_port_expected_red() {
    let (_file, artboard, machine, instance) = binding_precedence_fixture("source_first");
    let x = binding_typed_property::<ViewModelInstanceNumber>(&instance, "x");
    let y = binding_typed_property::<ViewModelInstanceNumber>(&instance, "y");
    binding_set_number(&x, 100.0);
    binding_set_number(&y, 100.0);
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    machine.advance_and_apply(0.0);
    let target = binding_first_node_target(&artboard);
    CoreRegistry::set_double_handle(&target, NodeBase::X_PROPERTY_KEY.into(), 700.0);
    CoreRegistry::set_double_handle(&target, NodeBase::Y_PROPERTY_KEY.into(), 800.0);
    machine.advance_and_apply(0.016);
    assert_eq!(binding_number_value(&x), 700.0);
    assert_eq!(binding_number_value(&y), 800.0);
}

use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    animation::{
        easing::Easing, elastic_ease::ElasticEase, elastic_interpolator::ElasticInterpolator,
        linear_animation::LinearAnimation,
    },
    artboard::Scene as ArtboardScene,
    constraints::distance_constraint::DistanceConstraint,
};

fn binding_scripted_default_silver_configured(asset: &str, set_frame_size: bool) -> BindingSilver {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let vm = RuntimeScriptingVmHandle::new(Box::new(
        ScriptVm::new_with_execution_limits(ScriptExecutionLimits::default())
            .expect("native script VM"),
    ));
    let bytes = std::fs::read(binding_path(&format!("assets/{asset}"))).expect("script fixture");
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut silver).expect("retained factory"),
        None,
        None,
        Some(vm),
    )
    .expect("signed scripted File");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    if set_frame_size {
        let (width, height) =
            artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
        silver.borrow_mut().frame_size(width as u32, height as u32);
    }
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    BindingSilver {
        machine,
        artboard,
        file,
        silver,
    }
}

fn binding_scripted_default_silver(asset: &str) -> BindingSilver {
    binding_scripted_default_silver_configured(asset, true)
}

fn binding_source_find<T: CoreType>(root: &CoreHandle, name: &str) -> CoreHandle {
    root.with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<T>(name))
        .flatten()
        .expect("authored source object")
}

#[test]
fn wave_b_data_binding_viewmodels_test_001_direct_port_expected_red() {
    let fixture = binding_scripted_default_silver("databind_viewmodel.riv");
    let instance = binding_default_instance(&fixture.file, &fixture.artboard);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance.clone()));
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    let child = binding_new_named_instance(&fixture.file, "StatefulChild");
    let number = binding_typed_property::<ViewModelInstanceNumber>(&child, "num");
    binding_set_number(&number, 44.0);
    ViewModelInstance::replace_view_model_by_name(&instance, "statefulChild", child);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_number(&number, 44.0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_click(&fixture.machine, Vec2D::new(25.0, 25.0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("databind_viewmodel");
}

#[test]
fn wave_b_data_binding_viewmodels_test_002_direct_port_expected_red() {
    let fixture = binding_asset_silver("unbound_stateful_component.riv", None);
    let instance = binding_default_instance(&fixture.file, &fixture.artboard);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("unbound_stateful_component");
}

#[test]
fn wave_b_data_binding_viewmodels_test_003_direct_port_expected_red() {
    let fixture = binding_asset_silver("bidirectional_stateful_property.riv", None);
    let instance = binding_default_instance(&fixture.file, &fixture.artboard);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    for point in [
        Vec2D::new(175.0, 175.0),
        Vec2D::new(450.0, 450.0),
        Vec2D::new(175.0, 175.0),
        Vec2D::new(450.0, 450.0),
        Vec2D::new(450.0, 50.0),
    ] {
        fixture.silver.borrow_mut().add_frame();
        binding_click(&fixture.machine, point);
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
    }
    let frames = (1.0_f32 / 0.2_f32) as i32;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.2);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("bidirectional_stateful_property");
}

#[test]
fn wave_b_default_state_machine_test_001_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("entry.riv", &mut factory);
    let artboard = file
        .with_file(|file| file.artboard_at(0))
        .expect("artboard 0");
    let index = artboard.with_artboard(|artboard| artboard.default_state_machine_index());
    assert!(index >= 0);
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.state_machine_name_at(index as usize)),
        "State Machine 1"
    );
    let machine = artboard
        .default_state_machine_handle()
        .expect("default state machine");
    assert_eq!(
        machine.with_instance(|machine| machine.name()),
        "State Machine 1"
    );
    let scene = artboard.default_scene().expect("default scene");
    let scene_name = match scene {
        ArtboardScene::StateMachine(scene) => scene.with_instance(|scene| scene.name()),
        ArtboardScene::LinearAnimation(scene) => scene.name(),
    };
    assert_eq!(scene_name, machine.with_instance(|machine| machine.name()));
}

#[test]
fn wave_b_distance_constraint_test_001_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("distance_constraint.riv", &mut factory);
    let root = file.with_file(File::artboard).expect("authored artboard");
    let a = binding_source_find::<Shape>(&root, "A");
    let b = binding_source_find::<Shape>(&root, "B");
    let constraints = a
        .with_downcast::<Shape, _>(|shape| shape.base.constraints().to_vec())
        .expect("Shape");
    assert_eq!(constraints.len(), 1);
    assert!(constraints[0].is_type_of(DistanceConstraint::TYPE_KEY));
    assert_eq!(
        constraints[0]
            .with_downcast::<DistanceConstraint, _>(|constraint| constraint.base.mode_value())
            .unwrap(),
        1
    );
    CoreRegistry::set_double_handle(&b, NodeBase::X_PROPERTY_KEY.into(), 259.31);
    CoreRegistry::set_double_handle(&b, NodeBase::Y_PROPERTY_KEY.into(), 137.87);
    Artboard::advance_handle(
        &root,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
    let at = a
        .with_downcast::<Shape, _>(|shape| shape.base.world_translation())
        .unwrap();
    let expected = Vec2D::new(259.2808837890625_f32, 62.87000274658203_f32);
    assert!(Vec2D::distance(at, expected) < 0.001);
}

#[test]
fn wave_b_draw_order_test_001_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("draw_rule_cycle.riv", &mut factory);
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let node = binding_find::<Node>(&artboard, "Blue");
    assert!(node.is_type_of(Shape::TYPE_KEY));
    Artboard::update_components_handle(&artboard.core_handle());
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.animation_count()),
        1
    );
    let mut animation = artboard.animation_at(0).expect("animation 0");
    for _ in 0..10 {
        animation.advance_and_apply(1.0);
        let mut renderer = factory.borrow().make_renderer();
        artboard.draw(&mut renderer);
    }
}

#[test]
fn wave_b_elastic_easing_test_001_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("test_elastic.riv", &mut factory);
    let root = file.with_file(File::artboard).expect("authored artboard");
    let interpolators = root
        .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<ElasticInterpolator>())
        .unwrap();
    assert_eq!(interpolators.len(), 1);
    let interpolator = &interpolators[0];
    assert_eq!(
        interpolator
            .with_downcast::<ElasticInterpolator, _>(|interpolator| interpolator.easing())
            .unwrap(),
        Some(Easing::EaseOut)
    );
    assert_eq!(
        interpolator
            .with_downcast::<ElasticInterpolator, _>(|interpolator| interpolator.base.amplitude())
            .unwrap(),
        1.0
    );
    assert_eq!(
        interpolator
            .with_downcast::<ElasticInterpolator, _>(|interpolator| interpolator.base.period())
            .unwrap(),
        0.25
    );
    let shapes = root
        .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<Shape>())
        .unwrap();
    assert_eq!(shapes.len(), 1);
    let shape = &shapes[0];
    assert!(binding_asset_approx(
        binding_shape_position(shape).0,
        145.19
    ));
    let animation = root
        .with_downcast::<Artboard, _>(|artboard| artboard.animation_named("Timeline 1"))
        .flatten()
        .expect("Timeline 1");
    let fps = animation
        .with_downcast::<LinearAnimation, _>(|animation| animation.base.fps())
        .unwrap() as f32;
    animation
        .with_downcast_mut::<LinearAnimation, _>(|animation| {
            root.with_downcast_mut::<Artboard, _>(|artboard| {
                animation.apply(artboard, 7.0 / fps, 1.0, None)
            })
            .expect("Artboard");
        })
        .expect("LinearAnimation");
    assert!(binding_asset_approx(
        binding_shape_position(shape).0,
        423.98
    ));
    animation
        .with_downcast_mut::<LinearAnimation, _>(|animation| {
            root.with_downcast_mut::<Artboard, _>(|artboard| {
                animation.apply(artboard, 14.0 / fps, 1.0, None)
            })
            .expect("Artboard");
        })
        .expect("LinearAnimation");
    assert!(binding_asset_approx(
        binding_shape_position(shape).0,
        303.995
    ));
}

#[test]
fn wave_b_elastic_easing_test_002_direct_port_expected_red() {
    let easer = ElasticEase::new(0.5, 3.14);
    assert_eq!(easer.compute_actual_amplitude(0.0), 1.0);
    assert_eq!(easer.compute_actual_amplitude(1.57), 0.5);
    assert!(binding_asset_approx(easer.ease_out(0.22), 0.8307));
    assert!(binding_asset_approx(easer.ease_in(1.58), 14.01086));
    assert!(binding_asset_approx(easer.ease_in_out(1.58), 1.0));
}

use nuxie_runtime::source::{
    component::Component,
    generated::assets::image_asset_base::ImageAssetBase,
    shapes::{path::Path, points_path::PointsPath},
};

fn binding_file_component_name(object: &CoreHandle) -> String {
    object
        .with(|object| object.as_component().expect("Component").name().to_owned())
        .expect("live component")
}

fn binding_file_parent(object: &CoreHandle) -> CoreHandle {
    object
        .with(|object| object.component_parent_handle())
        .flatten()
        .expect("authored parent")
}

fn binding_file_graph_order(object: &CoreHandle) -> u32 {
    object
        .with(|object| object.as_component().expect("Component").graph_order())
        .expect("live component")
}

fn binding_file_bytes(asset: &str) -> Vec<u8> {
    use std::io::Read;
    let path = binding_path(&format!("assets/{asset}"));
    let mut input = std::fs::File::open(&path)
        .unwrap_or_else(|error| panic!("open {}: {error}", path.display()));
    let length = input.metadata().expect("fixture file length").len() as usize;
    let mut bytes = Vec::with_capacity(length);
    let read = input
        .read_to_end(&mut bytes)
        .expect("read complete fixture");
    assert_eq!(read, length);
    bytes
}

fn binding_import_file_bytes(
    bytes: &[u8],
    factory: &mut PersistentFactory<RecordingFactory>,
) -> RuntimeFileHandle {
    let mut result = ImportResult::Malformed;
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(factory).expect("retained factory"),
        Some(&mut result),
        None,
        None,
    );
    assert_eq!(result, ImportResult::Success);
    file.expect("imported File")
}

#[test]
fn wave_b_file_test_002_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("two_artboards.riv", &mut factory);
    let artboard = file
        .with_file(File::artboard)
        .expect("default authored artboard");
    assert_eq!(binding_file_component_name(&artboard), "Two");
    assert!(
        file.with_file(|file| file.artboard_named_source("One"))
            .is_some()
    );
}

#[test]
fn wave_b_file_test_003_direct_port_expected_red() {
    let bytes = binding_file_bytes("solar-system.riv");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let mut result = ImportResult::Success;
    let _file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        Some(&mut result),
        None,
        None,
    );
    assert_eq!(result, ImportResult::Malformed);
}

#[test]
fn wave_b_file_test_004_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("juice.riv", &mut factory);
    let artboard = file.with_file(File::artboard).expect("authored artboard");
    assert_eq!(binding_file_component_name(&artboard), "New Artboard");
    let shin = binding_source_find::<Component>(&artboard, "shin_right");
    assert!(shin.is_type_of(Node::TYPE_KEY));
    let leg = binding_file_parent(&shin);
    assert_eq!(binding_file_component_name(&leg), "leg_right");
    let root = binding_file_parent(&leg);
    assert_eq!(binding_file_component_name(&root), "root");
    assert_eq!(binding_file_parent(&root), artboard);
    let walk = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.animation_named("walk"))
        .flatten()
        .expect("walk animation");
    assert_eq!(
        walk.with_downcast::<LinearAnimation, _>(|animation| animation.keyed_objects().len())
            .unwrap(),
        22
    );
}

#[test]
fn wave_b_file_test_005_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("dependency_test.riv", &mut factory);
    assert_eq!(file.with_file(File::artboard_count), 1);
    assert!(file.with_file(|file| file.artboard_at_source(0)).is_some());
    assert!(
        file.with_file(|file| file.artboard_named_source("Blue"))
            .is_some()
    );
}

#[test]
fn wave_b_file_test_006_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("dependency_test.riv", &mut factory);
    let artboard = file.with_file(File::artboard).expect("authored artboard");
    assert_eq!(binding_file_component_name(&artboard), "Blue");
    let node_a = binding_source_find::<Node>(&artboard, "A");
    let node_b = binding_source_find::<Node>(&artboard, "B");
    let node_c = binding_source_find::<Node>(&artboard, "C");
    let shape = binding_source_find::<Shape>(&artboard, "Rectangle");
    let path = binding_source_find::<Path>(&artboard, "Rectangle Path");
    assert_eq!(binding_file_parent(&node_a), artboard);
    assert_eq!(binding_file_parent(&node_b), node_a);
    assert_eq!(binding_file_parent(&node_c), node_b);
    assert_eq!(binding_file_parent(&shape), node_b);
    assert_eq!(binding_file_parent(&path), shape);
    assert_eq!(
        node_b
            .with(|object| object.as_component().unwrap().dependents().len())
            .unwrap(),
        2
    );
    assert_eq!(binding_file_graph_order(&artboard), 0);
    assert!(binding_file_graph_order(&node_a) > binding_file_graph_order(&artboard));
    assert!(binding_file_graph_order(&node_b) > binding_file_graph_order(&node_a));
    assert!(binding_file_graph_order(&node_c) > binding_file_graph_order(&node_b));
    assert!(binding_file_graph_order(&shape) > binding_file_graph_order(&node_b));
    assert!(binding_file_graph_order(&path) > binding_file_graph_order(&shape));
    Artboard::advance_handle(
        &artboard,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
    let world = shape
        .with(|object| {
            *object
                .as_world_transform_component()
                .unwrap()
                .world_transform()
        })
        .expect("shape world transform");
    assert_eq!(world[4], 39.203125_f32);
    assert_eq!(world[5], 29.535156_f32);
}

#[test]
fn wave_b_file_test_007_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("long_name.riv", &mut factory);
    let artboard = file.with_file(File::artboard).expect("authored artboard");
    assert_eq!(
        artboard
            .with_downcast::<Artboard, _>(|artboard| artboard.objects().len())
            .unwrap(),
        7
    );
}

#[cfg(feature = "tools")]
#[test]
fn wave_b_file_test_008_direct_port_expected_red() {
    let bytes = binding_file_bytes("jellyfish_test.riv");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_import_file_bytes(&bytes, &mut factory);
    let artboard = file.with_file(File::artboard).expect("authored artboard");
    assert_eq!(binding_file_component_name(&artboard), "Jellyfish");
    {
        let mut result = ImportResult::Malformed;
        let stripped =
            File::strip_assets(&bytes, &std::collections::HashSet::new(), Some(&mut result));
        assert_eq!(result, ImportResult::Success);
        assert_eq!(bytes.len(), stripped.len());
        assert_eq!(bytes, stripped);
    }
    {
        let mut result = ImportResult::Malformed;
        let stripped = File::strip_assets(
            &bytes,
            &std::collections::HashSet::from([ImageAssetBase::TYPE_KEY]),
            Some(&mut result),
        );
        assert_eq!(result, ImportResult::Success);
        assert!(stripped.len() < bytes.len());
    }
}

#[test]
fn wave_b_file_test_009_direct_port_expected_red() {
    let bytes = binding_file_bytes("bad_skin.riv");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_import_file_bytes(&bytes, &mut factory);
    let authored = file.with_file(File::artboard).expect("authored artboard");
    assert_eq!(
        binding_file_component_name(&authored),
        "Illustration WOman.svg"
    );
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default instance");
    Artboard::update_components_handle(&artboard.core_handle());
    let paths = artboard.with_artboard(|artboard| artboard.find_all_handles::<PointsPath>());
    for path in paths {
        path.with_downcast_mut::<PointsPath, _>(|path| path.mark_path_dirty(true))
            .expect("PointsPath");
    }
    Artboard::update_components_handle(&artboard.core_handle());
}

#[test]
fn wave_b_file_test_010_direct_port_expected_red() {
    let bytes = binding_file_bytes("magic_alley_db_reduced_export.riv");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_import_file_bytes(&bytes, &mut factory);
    let authored = file.with_file(File::artboard).expect("authored artboard");
    assert_eq!(binding_file_component_name(&authored), "Artboard");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default instance");
    Artboard::update_components_handle(&artboard.core_handle());
}

#[test]
fn wave_b_file_test_012_direct_port_expected_red() {
    struct ResetDeterministicMode;
    impl Drop for ResetDeterministicMode {
        fn drop(&mut self) {
            File::set_deterministic_mode(false);
        }
    }
    File::set_deterministic_mode(true);
    let reset_mode = ResetDeterministicMode;
    let fixture = BindingSilver::new("deterministic_mode.riv");
    let instance = binding_default_instance(&fixture.file, &fixture.artboard);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    let width = || fixture.artboard.with_artboard(|artboard| artboard.width());
    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(width() / 2.0, 400.0), 0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    let frames = (0.25_f32 / 0.016_f32) as i32;
    let mut y_pos = 400.0;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.machine.with_instance_mut(|machine| {
            machine.pointer_move(Vec2D::new(width() / 2.0, y_pos), 0.016, 0)
        });
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        y_pos -= 40.0;
    }
    fixture.silver.borrow_mut().add_frame();
    fixture.machine.with_instance_mut(|machine| {
        machine.pointer_move(Vec2D::new(width() / 2.0, y_pos), 0.016, 0)
    });
    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_up(Vec2D::new(width() / 2.0, y_pos), 0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    let frames = (1.0_f32 / 0.016_f32) as i32;
    for _ in 0..frames {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("deterministic_mode");
    drop(reset_mode);
}

use nuxie_runtime::source::{
    animation::listener_invocation::ListenerInvocation,
    artboard_component_list::ArtboardComponentList,
    bindable_artboard::RuntimeBindableArtboardHandle,
    input::{
        focus_manager::{FocusManager, RuntimeFocusManagerHandle},
        focus_node::{EdgeBehavior, FocusNode, FocusNodeRef, FocusableRef},
        focusable::{Focusable, Key, KeyModifiers},
        gamepad_snapshot::GamepadSnapshot,
    },
};
use std::cell::RefCell;

// This is the pinned focus_test.cpp MockFocusable: only the external callback
// counts/arguments and return value are test state. Nodes and managers are native.
struct BindingFocusObserver {
    key_input_count: usize,
    text_input_count: usize,
    gamepad_dispatch_count: usize,
    focused_count: usize,
    blurred_count: usize,
    last_text: String,
    last_key: Key,
    return_value: bool,
    eligible: bool,
    accepts_keyboard: bool,
}
impl Default for BindingFocusObserver {
    fn default() -> Self {
        Self {
            key_input_count: 0,
            text_input_count: 0,
            gamepad_dispatch_count: 0,
            focused_count: 0,
            blurred_count: 0,
            last_text: String::new(),
            last_key: Key::A,
            return_value: false,
            eligible: true,
            accepts_keyboard: false,
        }
    }
}
impl Focusable for BindingFocusObserver {
    fn is_eligible_for_focus_traversal(&self) -> bool {
        self.eligible
    }
    fn accepts_keyboard_input(&self) -> bool {
        self.accepts_keyboard
    }
    fn key_input(&mut self, key: Key, _: KeyModifiers, _: bool, _: bool) -> bool {
        self.key_input_count += 1;
        self.last_key = key;
        self.return_value
    }
    fn text_input(&mut self, text: &str) -> bool {
        self.text_input_count += 1;
        self.last_text = text.to_owned();
        self.return_value
    }
    fn gamepad_dispatch(
        &mut self,
        _: &ListenerInvocation,
        _: Option<&mut Option<CoreHandle>>,
    ) -> bool {
        self.gamepad_dispatch_count += 1;
        self.return_value
    }
    fn focused(&mut self) {
        self.focused_count += 1;
    }
    fn blurred(&mut self) {
        self.blurred_count += 1;
    }
}
fn binding_focus_observer() -> Rc<RefCell<BindingFocusObserver>> {
    Rc::new(RefCell::new(BindingFocusObserver::default()))
}
fn binding_focus_is_primary(manager: &RuntimeFocusManagerHandle, expected: &FocusNodeRef) -> bool {
    manager.with_focus_manager(|manager| {
        manager
            .primary_focus()
            .is_some_and(|actual| Rc::ptr_eq(&actual, expected))
    })
}

#[test]
fn wave_b_focus_test_001_direct_port_expected_red() {
    let node = FocusNode::new(None);
    let node = node.borrow();
    assert!(node.can_focus());
    assert!(node.can_touch());
    assert!(node.can_traverse());
    assert_eq!(node.tab_index(), 0);
    assert_eq!(node.edge_behavior(), EdgeBehavior::ParentScope);
    assert!(node.focusable().is_none());
    assert!(node.parent().is_none());
    assert!(node.children().is_empty());
    assert!(!node.is_scope());
    assert!(!node.has_focus());
    assert!(node.manager().is_none());
}

#[test]
fn wave_b_focus_test_002_direct_port_expected_red() {
    let node = FocusNode::new(None);
    node.borrow_mut().set_can_focus(false);
    assert!(!node.borrow().can_focus());
    node.borrow_mut().set_can_touch(false);
    assert!(!node.borrow().can_touch());
    node.borrow_mut().set_can_traverse(false);
    assert!(!node.borrow().can_traverse());
    node.borrow_mut().set_tab_index(42);
    assert_eq!(node.borrow().tab_index(), 42);
    node.borrow_mut()
        .set_edge_behavior(EdgeBehavior::ClosedLoop);
    assert_eq!(node.borrow().edge_behavior(), EdgeBehavior::ClosedLoop);
    node.borrow_mut().set_edge_behavior(EdgeBehavior::Stop);
    assert_eq!(node.borrow().edge_behavior(), EdgeBehavior::Stop);
}

#[test]
fn wave_b_focus_test_003_direct_port_expected_red() {
    let focusable = binding_focus_observer();
    let backing: FocusableRef = focusable.clone();
    let node = FocusNode::new(Some(backing.clone()));
    assert!(Rc::ptr_eq(&node.borrow().focusable().unwrap(), &backing));
    node.borrow_mut()
        .key_input(Key::A, KeyModifiers::NONE, true, false);
    assert_eq!(focusable.borrow().key_input_count, 1);
    assert_eq!(focusable.borrow().last_key, Key::A);
    node.borrow_mut().text_input("hello");
    assert_eq!(focusable.borrow().text_input_count, 1);
    assert_eq!(focusable.borrow().last_text, "hello");
    node.borrow_mut().focused();
    assert_eq!(focusable.borrow().focused_count, 1);
    node.borrow_mut().blurred();
    assert_eq!(focusable.borrow().blurred_count, 1);
}

#[test]
fn wave_b_focus_test_004_direct_port_expected_red() {
    let node = FocusNode::new(None);
    assert!(
        !node
            .borrow_mut()
            .key_input(Key::A, KeyModifiers::NONE, true, false)
    );
    assert!(!node.borrow_mut().text_input("hello"));
    node.borrow_mut().focused();
    node.borrow_mut().blurred();
}

#[test]
fn wave_b_focus_test_005_direct_port_expected_red() {
    let focusable = binding_focus_observer();
    let backing: FocusableRef = focusable.clone();
    let node = FocusNode::new(None);
    assert!(node.borrow().focusable().is_none());
    node.borrow_mut().set_focusable(Some(backing.clone()));
    assert!(Rc::ptr_eq(&node.borrow().focusable().unwrap(), &backing));
    node.borrow_mut().clear_focusable();
    assert!(node.borrow().focusable().is_none());
}

#[test]
fn wave_b_focus_test_006_direct_port_expected_red() {
    let parent = FocusNode::new(None);
    let child1 = FocusNode::new(None);
    let child2 = FocusNode::new(None);
    FocusNode::add_child(&parent, child1.clone());
    FocusNode::add_child(&parent, child2.clone());
    assert!(Rc::ptr_eq(&child1.borrow().parent().unwrap(), &parent));
    assert!(Rc::ptr_eq(&child2.borrow().parent().unwrap(), &parent));
    assert_eq!(parent.borrow().children().len(), 2);
    assert!(parent.borrow().is_scope());
    FocusNode::remove_child(&parent, &child1);
    assert!(child1.borrow().parent().is_none());
    assert_eq!(parent.borrow().children().len(), 1);
}

#[test]
fn wave_b_focus_test_007_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let focusable = binding_focus_observer();
    let node = FocusNode::new(Some(focusable.clone()));
    assert!(manager.with_focus_manager(|manager| manager.primary_focus().is_none()));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, node.clone(), None);
        manager.set_focus(node.clone());
    });
    assert!(binding_focus_is_primary(&manager, &node));
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&node)));
    assert!(manager.with_focus_manager(|manager| manager.has_primary_focus(&node)));
    assert_eq!(focusable.borrow().focused_count, 1);
    manager.with_focus_manager_mut(FocusManager::clear_focus);
    assert!(manager.with_focus_manager(|manager| manager.primary_focus().is_none()));
    assert_eq!(focusable.borrow().blurred_count, 1);
}

#[test]
fn wave_b_focus_test_008_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let focusable1 = binding_focus_observer();
    let focusable2 = binding_focus_observer();
    let node1 = FocusNode::new(Some(focusable1.clone()));
    let node2 = FocusNode::new(Some(focusable2.clone()));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, node1.clone(), None);
        manager.add_child(None, node2.clone(), None);
        manager.set_focus(node1.clone());
    });
    assert_eq!(focusable1.borrow().focused_count, 1);
    assert_eq!(focusable1.borrow().blurred_count, 0);
    manager.with_focus_manager_mut(|manager| manager.set_focus(node2.clone()));
    assert_eq!(focusable1.borrow().blurred_count, 1);
    assert_eq!(focusable2.borrow().focused_count, 1);
}

#[test]
fn wave_b_focus_test_009_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = FocusNode::new(None);
    node.borrow_mut().set_can_focus(false);
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, node.clone(), None);
        manager.set_focus(node.clone());
    });
    assert!(manager.with_focus_manager(|manager| manager.primary_focus().is_none()));
}

#[test]
fn wave_b_focus_test_010_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let parent = FocusNode::new(None);
    let child1 = FocusNode::new(None);
    let child2 = FocusNode::new(None);
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, parent.clone(), None);
        manager.add_child(Some(parent.clone()), child1.clone(), None);
        manager.add_child(Some(parent.clone()), child2.clone(), None);
    });
    assert!(parent.borrow().parent().is_none());
    assert!(Rc::ptr_eq(&child1.borrow().parent().unwrap(), &parent));
    assert!(Rc::ptr_eq(&child2.borrow().parent().unwrap(), &parent));
    assert!(parent.borrow().is_scope());
    assert!(!child1.borrow().is_scope());
    assert_eq!(parent.borrow().children().len(), 2);
    assert!(parent.borrow().manager().unwrap().ptr_eq(&manager));
    assert!(child1.borrow().manager().unwrap().ptr_eq(&manager));
    assert!(child2.borrow().manager().unwrap().ptr_eq(&manager));
}

#[test]
fn wave_b_focus_test_011_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let parent = FocusNode::new(None);
    let child = FocusNode::new(None);
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, parent.clone(), None);
        manager.add_child(Some(parent.clone()), child.clone(), None);
        manager.set_focus(child.clone());
    });
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&parent)));
    assert!(!manager.with_focus_manager(|manager| manager.has_primary_focus(&parent)));
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&child)));
    assert!(manager.with_focus_manager(|manager| manager.has_primary_focus(&child)));
    assert!(parent.borrow().has_focus());
    assert!(child.borrow().has_focus());
}

#[test]
fn wave_b_focus_test_012_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let focusable = binding_focus_observer();
    let node = FocusNode::new(Some(focusable.clone()));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, node.clone(), None);
        manager.set_focus(node.clone());
    });
    assert!(binding_focus_is_primary(&manager, &node));
    manager.with_focus_manager_mut(|manager| manager.remove_child(&node));
    assert!(manager.with_focus_manager(|manager| manager.primary_focus().is_none()));
    assert_eq!(focusable.borrow().blurred_count, 1);
}

#[test]
fn wave_b_focus_test_013_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let f_leaf = binding_focus_observer();
    let scope = FocusNode::new(None);
    scope.borrow_mut().set_can_focus(true);
    scope.borrow_mut().set_can_traverse(true);
    let row = FocusNode::new(None);
    row.borrow_mut().set_can_focus(true);
    row.borrow_mut().set_can_traverse(true);
    let leaf = FocusNode::new(Some(f_leaf.clone()));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, scope.clone(), None);
        manager.add_child(Some(scope.clone()), row.clone(), None);
        manager.add_child(Some(row.clone()), leaf.clone(), None);
        manager.set_focus(leaf.clone());
    });
    assert!(binding_focus_is_primary(&manager, &leaf));
    FocusNode::remove_from_parent(&row);
    assert!(binding_focus_is_primary(&manager, &leaf));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), row.clone(), Some(0))
    });
    assert!(binding_focus_is_primary(&manager, &leaf));
    assert_eq!(f_leaf.borrow().blurred_count, 0);
}

#[test]
fn wave_b_focus_test_014_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::make_structural_scope();
    let child = FocusNode::make_structural_scope();
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, scope.clone(), None);
        manager.add_child(Some(scope.clone()), child.clone(), None);
    });
    assert!(!manager.with_focus_manager_mut(FocusManager::has_focusable_content));
    child.borrow_mut().set_can_focus(true);
    assert!(manager.with_focus_manager_mut(FocusManager::has_focusable_content));
    child.borrow_mut().set_can_focus(false);
    assert!(!manager.with_focus_manager_mut(FocusManager::has_focusable_content));
}

#[test]
fn wave_b_focus_test_015_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let focusable = binding_focus_observer();
    let scope = FocusNode::make_structural_scope();
    let child = FocusNode::make_structural_scope();
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, scope.clone(), None);
        manager.add_child(Some(scope.clone()), child.clone(), None);
    });
    assert!(!manager.with_focus_manager_mut(FocusManager::has_focusable_content));
    child.borrow_mut().set_focusable(Some(focusable.clone()));
    assert!(manager.with_focus_manager_mut(FocusManager::has_focusable_content));
    child.borrow_mut().clear_focusable();
    assert!(!manager.with_focus_manager_mut(FocusManager::has_focusable_content));
}

#[test]
fn wave_b_focus_test_016_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let focusable = binding_focus_observer();
    let scope = FocusNode::make_structural_scope();
    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    assert!(!manager.with_focus_manager_mut(FocusManager::has_focusable_content));
    let backed = FocusNode::new(Some(focusable.clone()));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), backed.clone(), None)
    });
    assert!(manager.with_focus_manager_mut(FocusManager::has_focusable_content));
    manager.with_focus_manager_mut(|manager| manager.remove_child(&backed));
    assert!(!manager.with_focus_manager_mut(FocusManager::has_focusable_content));
}

#[test]
fn wave_b_focus_test_017_direct_port_expected_red() {
    let first = RuntimeFocusManagerHandle::new(FocusManager::new());
    let second = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = FocusNode::new(None);
    node.borrow_mut().set_can_focus(true);
    first.with_focus_manager_mut(|manager| manager.add_child(None, node.clone(), None));
    assert!(first.with_focus_manager_mut(FocusManager::has_focusable_content));
    second.with_focus_manager_mut(|manager| manager.add_child(None, node.clone(), None));
    assert!(!first.with_focus_manager_mut(FocusManager::has_focusable_content));
    assert!(second.with_focus_manager_mut(FocusManager::has_focusable_content));
}

#[test]
fn wave_b_focus_test_018_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let focusable = binding_focus_observer();
    focusable.borrow_mut().return_value = true;
    let node = FocusNode::new(Some(focusable.clone()));
    manager.with_focus_manager_mut(|manager| manager.add_child(None, node.clone(), None));
    assert!(!manager.with_focus_manager_mut(|manager| manager.key_input(
        Key::A,
        KeyModifiers::NONE,
        true,
        false
    )));
    assert!(!manager.with_focus_manager_mut(|manager| manager.text_input("hello")));
    let snap = GamepadSnapshot {
        device_id: 1,
        button_mask: 1,
        ..Default::default()
    };
    assert!(!manager.with_focus_manager_mut(|manager| {
        manager.gamepad_dispatch(&ListenerInvocation::gamepad_connected(&snap), None)
    }));
    manager.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    assert!(manager.with_focus_manager_mut(|manager| manager.key_input(
        Key::B,
        KeyModifiers::NONE,
        true,
        false
    )));
    assert_eq!(focusable.borrow().key_input_count, 1);
    assert_eq!(focusable.borrow().last_key, Key::B);
    assert!(manager.with_focus_manager_mut(|manager| manager.text_input("world")));
    assert_eq!(focusable.borrow().text_input_count, 1);
    assert_eq!(focusable.borrow().last_text, "world");
    assert!(manager.with_focus_manager_mut(|manager| {
        manager.gamepad_dispatch(&ListenerInvocation::gamepad_connected(&snap), None)
    }));
    assert_eq!(focusable.borrow().gamepad_dispatch_count, 1);
}

#[test]
fn wave_b_focus_test_019_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let f1 = binding_focus_observer();
    let f2 = binding_focus_observer();
    let f3 = binding_focus_observer();
    let node1 = FocusNode::new(Some(f1));
    let node2 = FocusNode::new(Some(f2));
    let node3 = FocusNode::new(Some(f3));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, node1.clone(), None);
        manager.add_child(None, node2.clone(), None);
        manager.add_child(None, node3.clone(), None);
        manager.set_focus(node1.clone());
    });
    assert!(binding_focus_is_primary(&manager, &node1));
    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &node2));
    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &node3));
    manager.with_focus_manager_mut(FocusManager::focus_previous);
    assert!(binding_focus_is_primary(&manager, &node2));
}

#[test]
fn wave_b_focus_test_020_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node1 = FocusNode::new(None);
    let node2 = FocusNode::new(None);
    let node3 = FocusNode::new(None);
    node1.borrow_mut().set_tab_index(3);
    node2.borrow_mut().set_tab_index(1);
    node3.borrow_mut().set_tab_index(2);
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, node1.clone(), None);
        manager.add_child(None, node2.clone(), None);
        manager.add_child(None, node3.clone(), None);
    });
    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &node2));
    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &node3));
    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &node1));
}

use nuxie_runtime::source::{
    animation::{
        focus_action_clear::FocusActionClear, focus_action_traversal::FocusActionTraversal,
        nested_state_machine::NestedStateMachine,
        state_machine_instance::RuntimeStateMachineLayerInstanceWeakHandle,
        transition_focus_condition::TransitionFocusCondition,
    },
    core::CoreObject,
    focus_data::FocusData,
    generated::{
        animation::transition_focus_condition_base::TransitionFocusConditionBase,
        focus_data_base::FocusDataBase,
        viewmodel::viewmodel_instance_artboard_base::ViewModelInstanceArtboardBase,
    },
    viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard,
};

#[cfg(feature = "tools")]
fn binding_empty_focus_machine() -> (
    CoreArena,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
) {
    let arena = CoreArena::default();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let source = arena.insert(Artboard::with_factory(
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
    ));
    let instance = Artboard::instance_from_handle(&source).expect("empty artboard instance");
    let definition = arena.insert(StateMachine::default());
    let machine = StateMachineInstance::new(definition, instance.downgrade());
    (arena, instance, machine)
}

fn binding_focus_traversal_action(kind: u32) -> FocusActionTraversal {
    let mut action = FocusActionTraversal::default();
    let mut base = std::mem::take(&mut action.base);
    base.set_traversal_kind(kind, &mut action);
    action.base = base;
    action
}
#[test]
fn wave_b_focus_test_021_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node1 = FocusNode::new(None);
    let node2 = FocusNode::new(None);
    let node3 = FocusNode::new(None);

    node2.borrow_mut().set_can_traverse(false);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, node1.clone(), None));
    manager.with_focus_manager_mut(|manager| manager.add_child(None, node2.clone(), None));
    manager.with_focus_manager_mut(|manager| manager.add_child(None, node3.clone(), None));

    manager.with_focus_manager_mut(|manager| manager.set_focus(node1.clone()));
    manager.with_focus_manager_mut(FocusManager::focus_next);

    assert!(binding_focus_is_primary(&manager, &node3));
}

#[test]
fn wave_b_focus_test_022_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    let node1 = FocusNode::new(None);
    let node2 = FocusNode::new(None);

    scope
        .borrow_mut()
        .set_edge_behavior(EdgeBehavior::ClosedLoop);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), node1.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), node2.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(node2.clone()));
    manager.with_focus_manager_mut(FocusManager::focus_next);

    assert!(binding_focus_is_primary(&manager, &node1));
}

#[test]
fn wave_b_focus_test_023_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    let node1 = FocusNode::new(None);
    let node2 = FocusNode::new(None);

    scope.borrow_mut().set_edge_behavior(EdgeBehavior::Stop);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), node1.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), node2.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(node2.clone()));
    manager.with_focus_manager_mut(FocusManager::focus_next);

    assert!(binding_focus_is_primary(&manager, &node2));
}

#[test]
fn wave_b_focus_test_024_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let grandparentFocusable = binding_focus_observer();
    let parentFocusable = binding_focus_observer();
    let childFocusable = binding_focus_observer();
    let grandparent = FocusNode::new(Some(grandparentFocusable.clone()));
    let parent = FocusNode::new(Some(parentFocusable.clone()));
    let child = FocusNode::new(Some(childFocusable.clone()));

    manager.with_focus_manager_mut(|manager| manager.add_child(None, grandparent.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(grandparent.clone()), parent.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(parent.clone()), child.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(child.clone()));

    assert!(childFocusable.borrow().focused_count == 1);
    assert!(parentFocusable.borrow().focused_count == 1);
    assert!(grandparentFocusable.borrow().focused_count == 1);

    assert!(child.borrow().has_focus() == true);
    assert!(parent.borrow().has_focus() == true);
    assert!(grandparent.borrow().has_focus() == true);
}

#[test]
fn wave_b_focus_test_025_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let parentFocusable = binding_focus_observer();
    let child1Focusable = binding_focus_observer();
    let child2Focusable = binding_focus_observer();
    let parent = FocusNode::new(Some(parentFocusable.clone()));
    let child1 = FocusNode::new(Some(child1Focusable.clone()));
    let child2 = FocusNode::new(Some(child2Focusable.clone()));

    manager.with_focus_manager_mut(|manager| manager.add_child(None, parent.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(parent.clone()), child1.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(parent.clone()), child2.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(child1.clone()));
    assert!(parentFocusable.borrow().focused_count == 1);
    assert!(child1Focusable.borrow().focused_count == 1);

    manager.with_focus_manager_mut(|manager| manager.set_focus(child2.clone()));
    assert!(child1Focusable.borrow().blurred_count == 1);
    assert!(child2Focusable.borrow().focused_count == 1);

    assert!(parentFocusable.borrow().focused_count == 1);
    assert!(parentFocusable.borrow().blurred_count == 0);

    assert!(parent.borrow().has_focus() == true);
}

#[test]
fn wave_b_focus_test_026_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scopeFocusable = binding_focus_observer();
    let leaf1Focusable = binding_focus_observer();
    let leaf2Focusable = binding_focus_observer();
    let scope = FocusNode::new(Some(scopeFocusable.clone()));
    let leaf1 = FocusNode::new(Some(leaf1Focusable.clone()));
    let leaf2 = FocusNode::new(Some(leaf2Focusable.clone()));

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), leaf1.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), leaf2.clone(), None)
    });

    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &leaf1));
    assert!(manager.with_focus_manager(|manager| manager.has_primary_focus(&scope)) == false);
    assert!(scope.borrow().has_focus() == true);

    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &leaf2));
}

#[test]
fn wave_b_focus_test_027_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope1 = FocusNode::new(None);
    let scope2 = FocusNode::new(None);
    let leaf = FocusNode::new(None);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope1.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope1.clone()), scope2.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope2.clone()), leaf.clone(), None)
    });

    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &leaf));
    assert!(scope1.borrow().has_focus() == true);
    assert!(scope2.borrow().has_focus() == true);
}

#[test]
fn wave_b_focus_test_028_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let root = FocusNode::new(None);
    let scope = FocusNode::new(None);
    let inner1 = FocusNode::new(None);
    let inner2 = FocusNode::new(None);
    let outer = FocusNode::new(None);

    scope
        .borrow_mut()
        .set_edge_behavior(EdgeBehavior::ParentScope);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, root.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(root.clone()), scope.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), inner1.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), inner2.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(root.clone()), outer.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(inner2.clone()));
    assert!(binding_focus_is_primary(&manager, &inner2));

    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &outer));
}

#[test]
fn wave_b_focus_test_029_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let parentFocusable = binding_focus_observer();
    let childFocusable = binding_focus_observer();
    let parent = FocusNode::new(Some(parentFocusable.clone()));
    let child = FocusNode::new(Some(childFocusable.clone()));

    manager.with_focus_manager_mut(|manager| manager.add_child(None, parent.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(parent.clone()), child.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(child.clone()));
    assert!(parent.borrow().has_focus() == true);
    assert!(child.borrow().has_focus() == true);

    manager.with_focus_manager_mut(FocusManager::clear_focus);

    assert!(parent.borrow().has_focus() == false);
    assert!(child.borrow().has_focus() == false);

    assert!(parentFocusable.borrow().blurred_count == 1);
    assert!(childFocusable.borrow().blurred_count == 1);
}

#[test]
fn wave_b_focus_test_030_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = FocusNode::new(None);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, node.clone(), None));
    assert!(node.borrow().manager().unwrap().ptr_eq(&manager));

    manager.with_focus_manager_mut(|manager| manager.remove_child(&node));
    assert!(node.borrow().manager().is_none());
}

#[test]
fn wave_b_focus_test_031_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    {
        let row = FocusNode::new(None);
        manager.with_focus_manager_mut(|manager| manager.add_child(None, row.clone(), None));
        manager.with_focus_manager_mut(|manager| {
            manager.add_child(Some(row.clone()), scope.clone(), None)
        });
        assert!(Rc::ptr_eq(&scope.borrow().parent().unwrap(), &row));

        manager.with_focus_manager_mut(|manager| manager.remove_child(&row));
    }

    assert!(scope.borrow().parent().is_none());

    let newParent = FocusNode::new(None);
    manager.with_focus_manager_mut(|manager| manager.add_child(None, newParent.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(newParent.clone()), scope.clone(), None)
    });
    assert!(Rc::ptr_eq(&scope.borrow().parent().unwrap(), &newParent));
    assert!(newParent.borrow().children().len() == 1);
}

#[test]
fn wave_b_focus_test_032_direct_port_expected_red() {
    let internalManager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let parentManager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);

    internalManager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    assert!(scope.borrow().manager().unwrap().ptr_eq(&internalManager));
    assert!(internalManager.with_focus_manager(|manager| manager.root_nodes().len()) == 1);

    parentManager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    assert!(scope.borrow().manager().unwrap().ptr_eq(&parentManager));
    assert!(parentManager.with_focus_manager(|manager| manager.root_nodes().len()) == 1);

    assert!(internalManager.with_focus_manager(|manager| manager.root_nodes().is_empty()));
}

#[test]
fn wave_b_focus_test_033_direct_port_expected_red() {
    let parentManager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    {
        let internalManager = RuntimeFocusManagerHandle::new(FocusManager::new());
        internalManager
            .with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
        parentManager
            .with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
        assert!(scope.borrow().manager().unwrap().ptr_eq(&parentManager));
    }

    assert!(scope.borrow().manager().unwrap().ptr_eq(&parentManager));

    let old_manager = scope.borrow().manager();
    if let Some(manager) = old_manager {
        manager.with_focus_manager_mut(|manager| manager.remove_child(&scope));
    }
    assert!(parentManager.with_focus_manager(|manager| manager.root_nodes().is_empty()));
}

#[test]
fn wave_b_focus_test_034_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let root = FocusNode::new(None);
    let before = FocusNode::new(None);
    let scope = FocusNode::new(None);
    let inner = FocusNode::new(None);

    scope
        .borrow_mut()
        .set_edge_behavior(EdgeBehavior::ParentScope);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, root.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(root.clone()), before.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(root.clone()), scope.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), inner.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(inner.clone()));

    manager.with_focus_manager_mut(FocusManager::focus_previous);
    assert!(binding_focus_is_primary(&manager, &before));
}

#[test]
fn wave_b_focus_test_035_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    let node1 = FocusNode::new(None);
    let node2 = FocusNode::new(None);

    scope
        .borrow_mut()
        .set_edge_behavior(EdgeBehavior::ClosedLoop);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), node1.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), node2.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(node1.clone()));
    manager.with_focus_manager_mut(FocusManager::focus_previous);

    assert!(binding_focus_is_primary(&manager, &node2));
}

#[test]
fn wave_b_focus_test_036_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    let node1 = FocusNode::new(None);
    let node2 = FocusNode::new(None);

    scope.borrow_mut().set_edge_behavior(EdgeBehavior::Stop);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), node1.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), node2.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(node1.clone()));
    manager.with_focus_manager_mut(FocusManager::focus_previous);

    assert!(binding_focus_is_primary(&manager, &node1));
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_037_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let scope = FocusNode::new(None);
    scope.borrow_mut().set_can_focus(false);
    scope.borrow_mut().set_can_traverse(false);
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    assert!(smi.with_instance(StateMachineInstance::has_focus_nodes) == false);
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_038_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let scope = FocusNode::new(None);
    scope.borrow_mut().set_can_focus(false);
    scope.borrow_mut().set_can_traverse(false);
    scope.borrow_mut().set_can_touch(false);
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));

    assert!(smi.with_instance(StateMachineInstance::has_focus_nodes) == false);

    let leaf = FocusNode::new(None);
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| {
            manager.add_child(Some(scope.clone()), leaf.clone(), None)
        });
    assert!(smi.with_instance(StateMachineInstance::has_focus_nodes) == true);
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_039_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();
    let mut focus_data = FocusData::default();
    let mut base = std::mem::take(&mut focus_data.base);
    base.set_focus_flags(
        base.focus_flags()
            & !(FocusDataBase::CAN_FOCUS_BITMASK | FocusDataBase::CAN_TRAVERSE_BITMASK),
        &mut focus_data,
    );
    focus_data.base = base;
    let node = focus_data.focus_node();
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.add_child(None, node, None));
    assert!(smi.with_instance(StateMachineInstance::has_focus_nodes));
}

#[test]
fn wave_b_focus_test_040_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let leafA = FocusNode::new(None);
    let scope = FocusNode::new(None);
    let leafC = FocusNode::new(None);

    scope.borrow_mut().set_can_focus(false);
    scope.borrow_mut().set_can_traverse(false);
    scope.borrow_mut().set_can_touch(false);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, leafA.clone(), None));
    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| manager.add_child(None, leafC.clone(), None));

    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &leafA));
    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &leafC));

    manager.with_focus_manager_mut(FocusManager::clear_focus);
    let leafB = FocusNode::new(None);
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), leafB.clone(), None)
    });

    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &leafA));
    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &leafB));
    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &leafC));
}

#[test]
fn wave_b_focus_test_041_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());

    let scope = FocusNode::new(None);
    scope.borrow_mut().set_can_focus(false);
    scope.borrow_mut().set_can_traverse(false);
    scope.borrow_mut().set_can_touch(false);

    let leafFocusable = binding_focus_observer();
    let leaf = FocusNode::new(Some(leafFocusable.clone()));
    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), leaf.clone(), None)
    });

    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &leaf));

    leafFocusable.borrow_mut().eligible = false;
    manager.with_focus_manager_mut(FocusManager::drop_focus_if_focus_target_hidden);
    assert!(manager.with_focus_manager(|manager| manager.primary_focus().is_none()));
}

#[test]
fn wave_b_focus_test_042_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());

    let scopeA = FocusNode::new(None);
    scopeA.borrow_mut().set_can_focus(false);
    scopeA.borrow_mut().set_can_traverse(false);
    scopeA.borrow_mut().set_can_touch(false);
    let scopeB = FocusNode::new(None);
    scopeB.borrow_mut().set_can_focus(false);
    scopeB.borrow_mut().set_can_traverse(false);
    scopeB.borrow_mut().set_can_touch(false);

    let leafAFocusable = binding_focus_observer();
    let leafBFocusable = binding_focus_observer();
    let leafA = FocusNode::new(Some(leafAFocusable.clone()));
    let leafB = FocusNode::new(Some(leafBFocusable.clone()));
    manager.with_focus_manager_mut(|manager| manager.add_child(None, scopeA.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scopeA.clone()), leafA.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| manager.add_child(None, scopeB.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scopeB.clone()), leafB.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(leafA.clone()));
    assert!(binding_focus_is_primary(&manager, &leafA));

    manager.with_focus_manager_mut(|manager| manager.remove_child(&leafB));
    let leafB2Focusable = binding_focus_observer();
    let leafB2 = FocusNode::new(Some(leafB2Focusable.clone()));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scopeB.clone()), leafB2.clone(), None)
    });

    assert!(binding_focus_is_primary(&manager, &leafA));
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_043_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let fm = smi.with_instance(StateMachineInstance::focus_manager);
    let f1 = binding_focus_observer();
    let f2 = binding_focus_observer();
    let node1 = FocusNode::new(Some(f1.clone()));
    let node2 = FocusNode::new(Some(f2.clone()));
    fm.with_focus_manager_mut(|manager| manager.add_child(None, node1.clone(), None));
    fm.with_focus_manager_mut(|manager| manager.add_child(None, node2.clone(), None));
    fm.with_focus_manager_mut(|manager| manager.set_focus(node1.clone()));

    let action = binding_focus_traversal_action(0);
    smi.with_instance_mut(|smi| action.perform(Some(smi), &ListenerInvocation::none()));

    assert!(binding_focus_is_primary(&fm, &node2));
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_044_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let fm = smi.with_instance(StateMachineInstance::focus_manager);
    let f1 = binding_focus_observer();
    let f2 = binding_focus_observer();
    let node1 = FocusNode::new(Some(f1.clone()));
    let node2 = FocusNode::new(Some(f2.clone()));
    fm.with_focus_manager_mut(|manager| manager.add_child(None, node1.clone(), None));
    fm.with_focus_manager_mut(|manager| manager.add_child(None, node2.clone(), None));
    fm.with_focus_manager_mut(|manager| manager.set_focus(node2.clone()));

    let action = binding_focus_traversal_action(1);
    smi.with_instance_mut(|smi| action.perform(Some(smi), &ListenerInvocation::none()));

    assert!(binding_focus_is_primary(&fm, &node1));
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_045_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let fm = smi.with_instance(StateMachineInstance::focus_manager);
    let f1 = binding_focus_observer();
    let f2 = binding_focus_observer();
    let node1 = FocusNode::new(Some(f1.clone()));
    let node2 = FocusNode::new(Some(f2.clone()));
    fm.with_focus_manager_mut(|manager| manager.add_child(None, node1.clone(), None));
    fm.with_focus_manager_mut(|manager| manager.add_child(None, node2.clone(), None));
    fm.with_focus_manager_mut(|manager| manager.set_focus(node1.clone()));

    let action = binding_focus_traversal_action(999);
    smi.with_instance_mut(|smi| action.perform(Some(smi), &ListenerInvocation::none()));

    assert!(binding_focus_is_primary(&fm, &node2));
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_046_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let f1 = binding_focus_observer();
    let f2 = binding_focus_observer();
    let node1 = FocusNode::new(Some(f1.clone()));
    let node2 = FocusNode::new(Some(f2.clone()));

    assert!(smi.with_instance(StateMachineInstance::has_focus_nodes) == false);

    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.add_child(None, node1.clone(), None));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.add_child(None, node2.clone(), None));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.set_focus(node1.clone()));

    assert!(smi.with_instance(StateMachineInstance::has_focus_nodes) == true);
    assert!(smi.with_instance_mut(StateMachineInstance::focus_next) == true);
    assert!(smi.with_instance_mut(StateMachineInstance::focus_previous) == true);
}

#[test]
fn wave_b_focus_test_047_direct_port_expected_red() {
    let action = binding_focus_traversal_action(0);
    action.perform(None, &ListenerInvocation::none());
}

#[test]
fn wave_b_focus_test_048_direct_port_expected_red() {
    let f = binding_focus_observer();
    assert!(f.borrow().accepts_keyboard_input() == false);

    let kf = binding_focus_observer();
    kf.borrow_mut().accepts_keyboard = true;
    assert!(kf.borrow().accepts_keyboard_input() == true);
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_049_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let state = smi.with_instance(StateMachineInstance::focus_state);
    assert!(state.has_focus == false);
    assert!(state.expects_keyboard_input == false);
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_050_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let f = binding_focus_observer();
    let node = FocusNode::new(Some(f.clone()));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.add_child(None, node.clone(), None));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.set_focus(node.clone()));

    let state = smi.with_instance(StateMachineInstance::focus_state);
    assert!(state.has_focus == true);
    assert!(state.expects_keyboard_input == false);
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_051_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let kf = binding_focus_observer();
    kf.borrow_mut().accepts_keyboard = true;
    let node = FocusNode::new(Some(kf.clone()));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.add_child(None, node.clone(), None));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.set_focus(node.clone()));

    let state = smi.with_instance(StateMachineInstance::focus_state);
    assert!(state.has_focus == true);
    assert!(state.expects_keyboard_input == true);
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_052_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let kf = binding_focus_observer();
    kf.borrow_mut().accepts_keyboard = true;
    let node = FocusNode::new(Some(kf.clone()));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.add_child(None, node.clone(), None));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.set_focus(node.clone()));

    assert!(
        smi.with_instance(StateMachineInstance::focus_state)
            .has_focus
            == true
    );

    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(FocusManager::clear_focus);

    let state = smi.with_instance(StateMachineInstance::focus_state);
    assert!(state.has_focus == false);
    assert!(state.expects_keyboard_input == false);
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_053_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let plain = binding_focus_observer();
    let kf = binding_focus_observer();
    kf.borrow_mut().accepts_keyboard = true;
    let plainNode = FocusNode::new(Some(plain.clone()));
    let kfNode = FocusNode::new(Some(kf.clone()));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.add_child(None, plainNode.clone(), None));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.add_child(None, kfNode.clone(), None));

    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.set_focus(plainNode.clone()));
    {
        let state = smi.with_instance(StateMachineInstance::focus_state);
        assert!(state.has_focus == true);
        assert!(state.expects_keyboard_input == false);
    }

    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.set_focus(kfNode.clone()));
    {
        let state = smi.with_instance(StateMachineInstance::focus_state);
        assert!(state.has_focus == true);
        assert!(state.expects_keyboard_input == true);
    }

    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.set_focus(plainNode.clone()));
    {
        let state = smi.with_instance(StateMachineInstance::focus_state);
        assert!(state.has_focus == true);
        assert!(state.expects_keyboard_input == false);
    }
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_054_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let external = RuntimeFocusManagerHandle::new(FocusManager::new());
    let kf = binding_focus_observer();
    kf.borrow_mut().accepts_keyboard = true;
    let node = FocusNode::new(Some(kf.clone()));
    external.with_focus_manager_mut(|manager| manager.add_child(None, node.clone(), None));
    external.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));

    assert!(
        smi.with_instance(StateMachineInstance::focus_state)
            .has_focus
            == false
    );

    smi.with_instance_mut(|smi| smi.set_external_focus_manager(Some(external.clone())));

    let state = smi.with_instance(StateMachineInstance::focus_state);
    assert!(state.has_focus == true);
    assert!(state.expects_keyboard_input == true);
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_055_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let kf = binding_focus_observer();
    kf.borrow_mut().accepts_keyboard = true;
    let node = FocusNode::new(Some(kf.clone()));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.add_child(None, node.clone(), None));
    smi.with_instance(StateMachineInstance::focus_manager)
        .with_focus_manager_mut(|manager| manager.set_focus(node.clone()));

    assert!(
        smi.with_instance(StateMachineInstance::focus_state)
            .has_focus
            == true
    );

    smi.with_instance_mut(StateMachineInstance::clear_focus);

    let state = smi.with_instance(StateMachineInstance::focus_state);
    assert!(state.has_focus == false);
    assert!(state.expects_keyboard_input == false);
}

#[test]
fn wave_b_focus_test_056_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    let leaf1 = FocusNode::new(None);
    let leaf2 = FocusNode::new(None);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), leaf1.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), leaf2.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(scope.clone()));
    assert!(binding_focus_is_primary(&manager, &leaf1));
}

#[test]
fn wave_b_focus_test_057_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    let row = FocusNode::new(None);
    let leaf = FocusNode::new(None);
    let sibling = FocusNode::new(None);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), row.clone(), None)
    });
    manager
        .with_focus_manager_mut(|manager| manager.add_child(Some(row.clone()), leaf.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), sibling.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(scope.clone()));
    assert!(binding_focus_is_primary(&manager, &leaf));
}

#[test]
fn wave_b_focus_test_058_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    let child = FocusNode::new(None);

    child.borrow_mut().set_can_traverse(false);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), child.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(scope.clone()));
    assert!(binding_focus_is_primary(&manager, &scope));
}

#[test]
fn wave_b_focus_test_059_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    let leaf = FocusNode::new(None);

    scope.borrow_mut().set_can_focus(false);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), leaf.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(scope.clone()));
    assert!(manager.with_focus_manager(|manager| manager.primary_focus().is_none()));
}

#[test]
fn wave_b_focus_test_060_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    let leaf1 = FocusNode::new(None);
    let leaf2 = FocusNode::new(None);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), leaf1.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), leaf2.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(leaf2.clone()));
    assert!(binding_focus_is_primary(&manager, &leaf2));
}

#[test]
fn wave_b_focus_test_061_direct_port_expected_red() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    let leaf1 = FocusNode::new(None);
    let leaf2 = FocusNode::new(None);

    manager.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), leaf1.clone(), None)
    });
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), leaf2.clone(), None)
    });

    manager.with_focus_manager_mut(|manager| manager.set_focus(scope.clone()));
    assert!(binding_focus_is_primary(&manager, &leaf1));

    manager.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(&manager, &leaf2));
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_062_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    let fm = smi.with_instance(StateMachineInstance::focus_manager);
    let f1 = binding_focus_observer();
    let node1 = FocusNode::new(Some(f1.clone()));
    fm.with_focus_manager_mut(|manager| manager.add_child(None, node1.clone(), None));
    fm.with_focus_manager_mut(|manager| manager.set_focus(node1.clone()));
    assert!(binding_focus_is_primary(&fm, &node1));

    let action = FocusActionClear::default();
    smi.with_instance_mut(|smi| action.perform(Some(smi), &ListenerInvocation::none()));

    assert!(fm.with_focus_manager(|manager| manager.primary_focus().is_none()));
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_063_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();

    assert!(
        smi.with_instance(StateMachineInstance::focus_manager)
            .with_focus_manager(|manager| manager.primary_focus().is_none())
    );

    let action = FocusActionClear::default();
    smi.with_instance_mut(|smi| action.perform(Some(smi), &ListenerInvocation::none()));

    assert!(
        smi.with_instance(StateMachineInstance::focus_manager)
            .with_focus_manager(|manager| manager.primary_focus().is_none())
    );
}

#[test]
fn wave_b_focus_test_064_direct_port_expected_red() {
    let action = FocusActionClear::default();

    action.perform(None, &ListenerInvocation::none());
}

#[test]
fn wave_b_focus_test_065_direct_port_expected_red() {
    let type_key = TransitionFocusConditionBase::TYPE_KEY;
    assert_eq!(type_key, 1038);
    let condition = TransitionFocusCondition::default();
    assert_eq!(CoreObject::core_type(&condition), type_key);
    assert!(CoreObject::is_type_of(&condition, type_key));
}

#[test]
fn wave_b_focus_test_066_direct_port_expected_red() {
    let condition = TransitionFocusCondition::default();
    assert!(!condition.evaluate(None, &RuntimeStateMachineLayerInstanceWeakHandle::default()));
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_focus_test_067_direct_port_expected_red() {
    let (_arena, _instance, smi) = binding_empty_focus_machine();
    let condition = TransitionFocusCondition::default();
    assert!(!smi.with_instance(|smi| condition.evaluate(
        Some(smi),
        &RuntimeStateMachineLayerInstanceWeakHandle::default()
    )));
}

struct BindingFocusFixture {
    file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    view_model: CoreHandle,
}

impl BindingFocusFixture {
    fn before_frames(asset: &str, artboard_name: Option<&str>) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = binding_file(asset, &mut factory);
        let artboard = file
            .with_file(|file| match artboard_name {
                Some(name) => file.artboard_named(name),
                None => file.artboard_default(),
            })
            .expect("authored focus artboard");
        let machine = artboard.state_machine_at(0).expect("focus state machine 0");
        let view_model = binding_default_instance(&file, &artboard);
        Self {
            file,
            artboard,
            machine,
            view_model,
        }
    }

    fn load(asset: &str, artboard_name: Option<&str>, frames: usize) -> Self {
        let fixture = Self::before_frames(asset, artboard_name);
        fixture.machine.with_instance_mut(|machine| {
            machine.bind_view_model_instance(fixture.view_model.clone())
        });
        fixture.frames(frames, 0.016);
        fixture
    }

    fn frames(&self, count: usize, elapsed: f32) {
        for _ in 0..count {
            self.machine.advance_and_apply(elapsed);
        }
    }

    fn manager(&self) -> RuntimeFocusManagerHandle {
        self.machine
            .with_instance(|machine| machine.focus_manager())
    }

    fn bindable(&self, name: &str) -> RuntimeBindableArtboardHandle {
        self.file
            .with_file(|file| file.bindable_artboard_named(name))
            .unwrap_or_else(|| panic!("bindable artboard {name}"))
    }
}

fn binding_set_artboard(
    instance: &CoreHandle,
    name: &str,
    value: Option<RuntimeBindableArtboardHandle>,
) {
    binding_property(instance, name)
        .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| property.set_asset(value))
        .expect("artboard property");
}

fn binding_set_artboard_id(instance: &CoreHandle, name: &str, value: u32) {
    assert!(CoreRegistry::set_uint_handle(
        &binding_property(instance, name),
        i32::from(ViewModelInstanceArtboardBase::PROPERTY_VALUE_PROPERTY_KEY),
        value,
    ));
}

fn binding_primary(manager: &RuntimeFocusManagerHandle) -> Option<FocusNodeRef> {
    manager.with_focus_manager(FocusManager::primary_focus)
}

fn binding_focused_artboard(manager: &RuntimeFocusManagerHandle) -> Option<CoreHandle> {
    manager.with_focus_manager(FocusManager::primary_focus_immediate_artboard)
}

fn binding_focused_artboard_name(manager: &RuntimeFocusManagerHandle) -> String {
    binding_focused_artboard(manager)
        .and_then(|artboard| {
            artboard.with(|artboard| {
                artboard
                    .as_artboard()
                    .map(|artboard| artboard.base.name().to_owned())
            })
        })
        .flatten()
        .unwrap_or_else(|| "<none>".to_owned())
}

fn binding_focus_ptr_eq(left: &Option<FocusNodeRef>, right: &Option<FocusNodeRef>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Rc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn binding_focus_set_asset(fixture: &BindingFocusFixture, property: &str, source: &str) {
    let asset = fixture.bindable(source);
    binding_set_artboard(&fixture.view_model, property, Some(asset));
}

fn binding_nested_instance_named(
    artboard: &RuntimeArtboardInstanceHandle,
    name: &str,
) -> RuntimeArtboardInstanceHandle {
    artboard
        .with_artboard(|artboard| artboard.nested_artboards())
        .into_iter()
        .find_map(|host| {
            host.with_downcast::<NestedArtboard, _>(|host| host.artboard_instance_default())
                .flatten()
                .filter(|child| child.with_artboard(|child| child.base.name() == name))
        })
        .unwrap_or_else(|| panic!("mounted nested artboard {name}"))
}

#[test]
fn wave_b_focus_test_068_direct_port_expected_red() {
    let fixture = BindingFocusFixture::load("bindable_focus_tree_swap.riv", None, 1);
    let manager = fixture.manager();
    assert!(
        fixture
            .machine
            .with_instance(|machine| machine.has_focus_nodes())
    );
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.frames(1, 0.016);
    assert!(binding_primary(&manager).is_some());
    assert!(
        !fixture
            .machine
            .with_instance_mut(|machine| machine.focus_next())
    );
    fixture
        .machine
        .with_instance_mut(|machine| machine.focus_previous());

    binding_focus_set_asset(&fixture, "bindedArt", "Focusable");
    fixture.frames(1, 0.016);
    let focusable = binding_nested_instance_named(&fixture.artboard, "Focusable");
    assert!(
        fixture
            .machine
            .with_instance_mut(|machine| machine.focus_next())
    );
    assert!(binding_primary(&manager).is_some());
    assert_eq!(
        binding_focused_artboard(&manager),
        Some(focusable.core_handle())
    );
}

#[test]
fn wave_b_focus_test_069_direct_port_expected_red() {
    let fixture = BindingFocusFixture::load("bindable_focus_tree_swap.riv", None, 1);
    let manager = fixture.manager();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    let focused = binding_primary(&manager);
    assert!(focused.is_some());
    assert_eq!(
        binding_focused_artboard(&manager),
        Some(fixture.artboard.core_handle())
    );

    binding_focus_set_asset(&fixture, "bindedArt", "Focusable");
    fixture.frames(1, 0.016);
    assert!(binding_focus_ptr_eq(&binding_primary(&manager), &focused));
    assert_eq!(
        binding_focused_artboard(&manager),
        Some(fixture.artboard.core_handle())
    );
}

#[test]
fn wave_b_focus_test_070_direct_port_expected_red() {
    let fixture = BindingSilver::new("focus_collapsing.riv");
    let instance = binding_default_instance(&fixture.file, &fixture.artboard);
    let opacity = binding_property(&instance, "opacity");
    let visible = binding_property(&instance, "isMainLayout2Visible");
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    let manager = fixture
        .machine
        .with_instance(|machine| machine.focus_manager());
    let mut renderer = fixture.silver.borrow().make_renderer();

    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    assert!(binding_primary(&manager).is_some());
    assert!(binding_focused_artboard(&manager).is_some());
    assert_ne!(
        binding_focused_artboard(&manager),
        Some(fixture.artboard.core_handle())
    );
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    assert!(binding_primary(&manager).is_some());
    assert_eq!(
        binding_focused_artboard(&manager),
        Some(fixture.artboard.core_handle())
    );
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    binding_set_number(&opacity, 0.0);
    fixture.advance(0.016);
    fixture.advance(0.016);
    assert!(binding_primary(&manager).is_none());
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    binding_set_number(&opacity, 1.0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
        manager.focus_next();
    });
    fixture.advance(0.016);
    assert!(binding_primary(&manager).is_some());
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    binding_set_boolean(&visible, false);
    fixture.advance(0.016);
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
        manager.focus_next();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    binding_set_boolean(&visible, true);
    fixture.advance(0.016);
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.016);
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.advance(0.016);
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("focus_collapsing");
}

#[test]
fn wave_b_focus_test_071_direct_port_expected_red() {
    let fixture = BindingSilver::new("keyboard_listener.riv");
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    let manager = fixture
        .machine
        .with_instance(|machine| machine.focus_manager());
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    manager.with_focus_manager_mut(|manager| {
        manager.focus_previous();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.key_input(Key::SPACE, KeyModifiers::NONE, false, false);
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    manager.with_focus_manager_mut(|manager| {
        manager.focus_previous();
        manager.focus_previous();
        manager.focus_previous();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.key_input(Key::SPACE, KeyModifiers::NONE, false, false);
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    manager.with_focus_manager_mut(|manager| {
        manager.focus_previous();
        manager.focus_previous();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.key_input(Key::SPACE, KeyModifiers::NONE, false, false);
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_previous();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.key_input(Key::SPACE, KeyModifiers::NONE, false, false);
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("keyboard_listener");
}

#[test]
fn wave_b_focus_test_072_direct_port_expected_red() {
    let fixture = binding_asset_silver("keyboard_listener.riv", Some("KeyboardInput"));
    let instance = fixture.instance();
    let key_count = binding_property(&instance, "keyCount");
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    let manager = fixture
        .machine
        .with_instance(|machine| machine.focus_manager());
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    let send = |key, modifiers, pressed, repeat| {
        manager.with_focus_manager_mut(|manager| {
            manager.key_input(key, modifiers, pressed, repeat);
        });
    };
    send(Key::A, KeyModifiers::NONE, true, false);
    fixture.advance(0.016);
    assert_eq!(binding_number_value(&key_count), 1.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    send(Key::A, KeyModifiers::NONE, true, true);
    fixture.advance(0.016);
    assert_eq!(binding_number_value(&key_count), 1.0);
    send(Key::A, KeyModifiers::NONE, false, false);
    fixture.advance(0.016);
    assert_eq!(binding_number_value(&key_count), 2.0);
    send(Key::A, KeyModifiers::SHIFT, true, false);
    fixture.advance(0.016);
    assert_eq!(binding_number_value(&key_count), 2.0);
    send(Key::E, KeyModifiers::NONE, false, false);
    send(Key::E, KeyModifiers::NONE, true, true);
    send(Key::E, KeyModifiers::NONE, true, false);
    assert_eq!(binding_number_value(&key_count), 2.0);
    fixture.advance(0.016);
    send(Key::B, KeyModifiers::NONE, true, false);
    assert_eq!(binding_number_value(&key_count), 2.0);
    fixture.advance(0.016);
    send(Key::B, KeyModifiers::NONE, false, false);
    fixture.advance(0.016);
    assert_eq!(binding_number_value(&key_count), 3.0);
    send(Key::B, KeyModifiers::NONE, true, true);
    fixture.advance(0.016);
    assert_eq!(binding_number_value(&key_count), 4.0);
    send(Key::D, KeyModifiers::NONE, true, false);
    fixture.advance(0.016);
    assert_eq!(binding_number_value(&key_count), 4.0);
    send(
        Key::D,
        KeyModifiers::SHIFT | KeyModifiers::META,
        true,
        false,
    );
    fixture.advance(0.016);
    assert_eq!(binding_number_value(&key_count), 5.0);
    send(
        Key::C,
        KeyModifiers::SHIFT | KeyModifiers::META,
        true,
        false,
    );
    fixture.advance(0.016);
    assert_eq!(binding_number_value(&key_count), 5.0);
    send(Key::C, KeyModifiers::SHIFT, true, false);
    fixture.advance(0.016);
    assert_eq!(binding_number_value(&key_count), 6.0);
    send(Key::X, KeyModifiers::SHIFT, true, false);
    fixture.advance(0.016);
    assert_eq!(binding_number_value(&key_count), 6.0);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("keyboard_listener-KeyboardInput");
}

#[test]
fn wave_b_focus_test_073_direct_port_expected_red() {
    let mut fixture = BindingFocusFixture::before_frames("text_input_event.riv", None);
    fixture.view_model = fixture
        .file
        .with_file_mut(|file| {
            file.create_view_model_instance_for_artboard(fixture.artboard.core_handle())
        })
        .expect("artboard view-model instance");
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(fixture.view_model.clone()));
    fixture.frames(1, 0.016);
    let is_focused = binding_property(&fixture.view_model, "isFocused");
    let has_keyed = binding_property(&fixture.view_model, "hasKeyed");
    let has_texted = binding_property(&fixture.view_model, "hasTexted");
    let read = |property: &CoreHandle| {
        property
            .with_downcast::<ViewModelInstanceBoolean, _>(|property| property.value())
            .expect("boolean property")
    };
    let manager = fixture.manager();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.frames(1, 0.016);
    assert!(read(&is_focused));
    assert!(!read(&has_keyed));
    assert!(!read(&has_texted));
    manager.with_focus_manager_mut(|manager| {
        manager.key_input(Key::B, KeyModifiers::NONE, true, false);
    });
    fixture.frames(1, 0.016);
    assert!(read(&is_focused));
    assert!(!read(&has_keyed));
    assert!(!read(&has_texted));
    manager.with_focus_manager_mut(|manager| {
        manager.text_input("b");
    });
    fixture.frames(1, 0.016);
    assert!(read(&is_focused));
    assert!(!read(&has_keyed));
    assert!(read(&has_texted));
    manager.with_focus_manager_mut(|manager| {
        manager.key_input(Key::A, KeyModifiers::NONE, true, false);
    });
    fixture.frames(1, 0.016);
    assert!(read(&is_focused));
    assert!(read(&has_keyed));
    assert!(read(&has_texted));
}

#[test]
fn wave_b_focus_test_074_direct_port_expected_red() {
    let fixture = BindingSilver::new("focus_traversal.riv");
    let instance = binding_default_instance(&fixture.file, &fixture.artboard);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    let click = |point: Vec2D| {
        fixture
            .machine
            .with_instance_mut(|machine| machine.pointer_down(point, 0));
        fixture
            .machine
            .with_instance_mut(|machine| machine.pointer_up(point, 0));
        fixture.advance(0.016);
    };
    click(Vec2D::new(180.0, 450.0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    click(Vec2D::new(60.0, 450.0));
    click(Vec2D::new(60.0, 450.0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    click(Vec2D::new(60.0, 350.0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    click(Vec2D::new(420.0, 350.0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    click(Vec2D::new(300.0, 350.0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    click(Vec2D::new(180.0, 350.0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("focus_traversal");
}

#[test]
fn wave_b_focus_test_075_direct_port_expected_red() {
    let fixture = BindingSilver::new("focusable_element.riv");
    let instance = fixture
        .file
        .with_file_mut(|file| {
            file.create_view_model_instance_for_artboard(fixture.artboard.core_handle())
        })
        .expect("artboard view-model instance");
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    let manager = fixture
        .machine
        .with_instance(|machine| machine.focus_manager());
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    for _ in 0..7 {
        fixture.silver.borrow_mut().add_frame();
        manager.with_focus_manager_mut(|manager| {
            manager.focus_next();
        });
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("focusable_element");
}

#[test]
fn wave_b_focus_test_076_direct_port_expected_red() {
    let fixture = BindingFocusFixture::before_frames("component_list_1.riv", Some("Main"));
    fixture
        .artboard
        .bind_view_model_instance(Some(fixture.view_model.clone()));
    let _machine = fixture
        .artboard
        .state_machine_at(0)
        .expect("state machine 0");
    fixture.artboard.advance_default(0.0);
    let list = fixture
        .artboard
        .with_artboard(|artboard| artboard.find_handle::<ArtboardComponentList>("List"))
        .expect("List");
    let manager = fixture
        .artboard
        .with_artboard(|artboard| artboard.focus_manager_handle())
        .expect("shared focus manager");
    fixture
        .artboard
        .build_focus_tree(Some(manager.clone()), None);
    let scope = list
        .with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::list_scope_focus_node)
        .flatten()
        .expect("list scope");
    let scope = scope.borrow();
    assert!(scope.manager().expect("scope manager").ptr_eq(&manager));
    assert_eq!(scope.name, "ArtboardComponentListScope");
    assert!(!scope.can_focus());
    assert!(!scope.can_traverse());
    assert!(scope.focusable().is_none());
}

#[test]
fn wave_b_focus_test_077_direct_port_expected_red() {
    let fixture = BindingFocusFixture::before_frames("component_list_1.riv", Some("Main"));
    fixture
        .artboard
        .bind_view_model_instance(Some(fixture.view_model.clone()));
    let _machine = fixture
        .artboard
        .state_machine_at(0)
        .expect("state machine 0");
    fixture.artboard.advance_default(0.0);
    let list = fixture
        .artboard
        .with_artboard(|artboard| artboard.find_handle::<ArtboardComponentList>("List"))
        .expect("List");
    let parent = list
        .with(|list| list.component_parent_handle())
        .flatten()
        .expect("List parent");
    assert!(parent.is_type_of(NodeBase::TYPE_KEY));
    let children = parent
        .with(|parent| {
            parent
                .as_container_component()
                .expect("Node container")
                .children()
                .to_vec()
        })
        .expect("live Node");
    let direct = children
        .into_iter()
        .find(|child| child.is_type_of(FocusData::TYPE_KEY))
        .map(|child| {
            child
                .with_downcast_mut::<FocusData, _>(FocusData::focus_node)
                .expect("direct FocusData")
        });
    if let Some(direct) = direct {
        let closest = FocusData::find_closest_focus_node_handle(list).expect("closest focus node");
        assert!(Rc::ptr_eq(&closest, &direct));
    }
}

#[test]
fn wave_b_focus_test_078_direct_port_expected_red() {
    // Pinned File::registerScripts creates a VM automatically whenever the
    // imported file contains ScriptAssets. The Rust runtime receives its VM
    // from the host, so the direct port must supply the equivalent here.
    let fixture = binding_scripted_default_silver("list_focus_order.riv");
    let instance = binding_default_instance(&fixture.file, &fixture.artboard);
    let stage_processed = binding_property(&instance, "stageProcessed");
    let stage_count = binding_property(&instance, "stageCount");
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    let manager = fixture
        .machine
        .with_instance(|machine| machine.focus_manager());
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
        manager.focus_next();
        manager.focus_next();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    for stage in 1..=3 {
        binding_set_boolean(&stage_processed, false);
        binding_set_number(&stage_count, stage as f32);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        fixture.silver.borrow_mut().add_frame();
        if stage == 1 || stage == 2 {
            manager.with_focus_manager_mut(|manager| {
                manager.focus_next();
            });
            fixture.advance(0.016);
            fixture.artboard.draw(&mut renderer);
            fixture.silver.borrow_mut().add_frame();
        }
    }
    manager.with_focus_manager_mut(|manager| {
        manager.focus_next();
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("list_focus_order");
}

#[test]
fn wave_b_focus_test_079_direct_port_expected_red() {
    let fixture = BindingSilver::new("focus_test.riv");
    let instance = binding_default_instance(&fixture.file, &fixture.artboard);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(55.0, 65.0), 0));
    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_up(Vec2D::new(55.0, 65.0), 0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(442.0, 65.0), 0));
    fixture
        .machine
        .with_instance_mut(|machine| machine.pointer_up(Vec2D::new(442.0, 65.0), 0));
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("focus_test");
}

#[test]
fn wave_b_focus_test_080_direct_port_expected_red() {
    let fixture = BindingFocusFixture::load("list_focus_order.riv", None, 1);
    let manager = fixture.manager();
    let lists = fixture
        .artboard
        .with_artboard(|artboard| artboard.artboard_component_lists());
    assert_eq!(lists.len(), 1);
    let list = lists[0].clone();
    let count = list
        .with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::artboard_count)
        .expect("component list") as i32;
    assert!(count > 0);
    let row = |index: i32| -> Option<FocusNodeRef> {
        list.with_downcast::<ArtboardComponentList, _>(|list| {
            list.list_scope_focus_node()
                .and_then(|scope| scope.borrow().children().get(index as usize).cloned())
        })
        .flatten()
    };
    let machine_at = |index: i32| {
        list.with_downcast::<ArtboardComponentList, _>(|list| list.state_machine_instance(index))
            .flatten()
    };
    let target = (0..count)
        .find(|index| {
            row(*index).is_some_and(|row| !row.borrow().children().is_empty())
                && machine_at(*index).is_some()
        })
        .expect("list item with focus content and a state machine");
    let item_machine = machine_at(target).expect("target item state machine");
    item_machine.with_instance_mut(|machine| machine.set_external_focus_manager(None));
    assert!(
        !item_machine
            .with_instance(|machine| machine.focus_manager())
            .ptr_eq(&manager)
    );

    fixture.artboard.cleanup_focus_tree();
    fixture
        .artboard
        .build_focus_tree(Some(manager.clone()), None);
    let target_row = row(target).expect("rebuilt target row");
    assert!(
        target_row
            .borrow()
            .manager()
            .expect("row manager")
            .ptr_eq(&manager)
    );
    assert!(!target_row.borrow().children().is_empty());
    assert!(
        item_machine
            .with_instance(|machine| machine.focus_manager())
            .ptr_eq(&manager)
    );
}

fn binding_assert_focus_sequence(
    machine: &RuntimeStateMachineInstanceHandle,
    manager: &RuntimeFocusManagerHandle,
    expected: &[&str],
) {
    for name in expected {
        assert!(machine.with_instance_mut(|machine| machine.focus_next()));
        assert_eq!(binding_focused_artboard_name(manager), *name);
    }
    assert!(!machine.with_instance_mut(|machine| machine.focus_next()));
    assert!(binding_primary(manager).is_none());
}

#[test]
fn wave_b_focus_test_081_direct_port_expected_red() {
    let fixture = BindingFocusFixture::load("swappable_artboards_focus.riv", Some("Main"), 2);
    let manager = fixture.manager();
    let mut slot_found = false;
    for host in fixture
        .artboard
        .with_artboard(|artboard| artboard.nested_artboards())
    {
        let (source, bound) = host
            .with_downcast::<NestedArtboard, _>(|host| {
                (host.source_artboard(), host.is_artboard_data_bound())
            })
            .expect("nested host");
        let source = source.expect("nested source");
        let name = source
            .with(|source| {
                source
                    .as_artboard()
                    .map(|source| source.base.name().to_owned())
            })
            .flatten()
            .expect("source artboard");
        if name == "Swappable1" || name == "Swappable2" {
            assert!(bound);
            slot_found = true;
        } else {
            assert!(!bound);
        }
    }
    assert!(slot_found);
    assert!(
        fixture
            .machine
            .with_instance(|machine| machine.has_focus_nodes())
    );
    binding_assert_focus_sequence(
        &fixture.machine,
        &manager,
        &["Main", "Swappable1", "StaticNestWithFocusable"],
    );

    binding_focus_set_asset(&fixture, "artboardProp", "Swappable2");
    fixture.frames(1, 0.016);
    binding_assert_focus_sequence(
        &fixture.machine,
        &manager,
        &["Main", "StaticNestWithFocusable"],
    );

    assert!(
        fixture
            .machine
            .with_instance_mut(|machine| machine.focus_next())
    );
    assert_eq!(binding_focused_artboard_name(&manager), "Main");
    let held = binding_primary(&manager);
    binding_focus_set_asset(&fixture, "artboardProp", "Swappable1");
    fixture.frames(1, 0.016);
    assert!(binding_focus_ptr_eq(&binding_primary(&manager), &held));
    assert_eq!(binding_focused_artboard_name(&manager), "Main");
    assert!(
        fixture
            .machine
            .with_instance_mut(|machine| machine.focus_next())
    );
    assert_eq!(binding_focused_artboard_name(&manager), "Swappable1");
    assert!(
        fixture
            .machine
            .with_instance_mut(|machine| machine.focus_next())
    );
    assert_eq!(
        binding_focused_artboard_name(&manager),
        "StaticNestWithFocusable"
    );
    assert!(
        !fixture
            .machine
            .with_instance_mut(|machine| machine.focus_next())
    );
}

#[test]
fn wave_b_focus_test_082_direct_port_expected_red() {
    let fixture = BindingFocusFixture::load("swappable_artboards_focus.riv", Some("Main"), 2);
    let manager = fixture.manager();
    for _ in 0..3 {
        assert!(
            fixture
                .machine
                .with_instance_mut(|machine| machine.focus_next())
        );
    }
    assert_eq!(
        binding_focused_artboard_name(&manager),
        "StaticNestWithFocusable"
    );
    let held = binding_primary(&manager);
    assert!(held.is_some());
    fixture
        .artboard
        .build_focus_tree(Some(manager.clone()), None);
    assert!(binding_focus_ptr_eq(&binding_primary(&manager), &held));
    assert_eq!(
        binding_focused_artboard_name(&manager),
        "StaticNestWithFocusable"
    );
}

#[test]
fn wave_b_focus_test_083_direct_port_expected_red() {
    let fixture = BindingFocusFixture::load("swappable_artboards_focus.riv", Some("Main"), 1);
    let foreign = BindingFocusFixture::before_frames("swappable_artboards_focus.riv", Some("Main"));
    let manager = fixture.manager();
    binding_set_artboard(
        &fixture.view_model,
        "artboardProp",
        Some(foreign.bindable("Swappable1")),
    );
    fixture.frames(1, 0.016);
    binding_assert_focus_sequence(
        &fixture.machine,
        &manager,
        &["Main", "Swappable1", "StaticNestWithFocusable"],
    );

    let bound_machine = fixture
        .artboard
        .with_artboard(|artboard| artboard.nested_artboards())
        .into_iter()
        .find_map(|host| {
            host.with_downcast::<NestedArtboard, _>(|host| {
                host.is_artboard_data_bound().then(|| {
                    host.nested_animations().iter().find_map(|animation| {
                        animation
                            .with_downcast::<NestedStateMachine, _>(|nested| {
                                nested.state_machine_instance()
                            })
                            .flatten()
                    })
                })
            })
            .flatten()
            .flatten()
        })
        .expect("bound nested state machine");
    assert!(
        bound_machine
            .with_instance(|machine| machine.focus_manager())
            .ptr_eq(&manager)
    );
}

#[test]
fn wave_b_focus_test_084_direct_port_expected_red() {
    let fixture = BindingFocusFixture::load("swappable_artboards_focus.riv", Some("Main"), 2);
    let manager = fixture.manager();
    assert!(
        fixture
            .machine
            .with_instance_mut(|machine| machine.focus_next())
    );
    assert_eq!(binding_focused_artboard_name(&manager), "Main");
    let held = binding_primary(&manager);
    assert!(held.is_some());

    binding_set_artboard_id(&fixture.view_model, "artboardProp", 9999);
    let property = binding_property(&fixture.view_model, "artboardProp");
    let (asset, value) = property
        .with_downcast::<ViewModelInstanceArtboard, _>(|property| {
            (property.asset(), property.base.property_value())
        })
        .expect("artboard property");
    assert!(asset.is_none());
    assert_ne!(value, u32::MAX);
    fixture.frames(1, 0.016);
    assert!(binding_focus_ptr_eq(&binding_primary(&manager), &held));
    assert_eq!(binding_focused_artboard_name(&manager), "Main");
    assert!(
        fixture
            .machine
            .with_instance_mut(|machine| machine.focus_next())
    );
    assert_eq!(binding_focused_artboard_name(&manager), "Swappable1");
    assert!(
        fixture
            .machine
            .with_instance_mut(|machine| machine.focus_next())
    );
    assert_eq!(
        binding_focused_artboard_name(&manager),
        "StaticNestWithFocusable"
    );
    assert!(
        !fixture
            .machine
            .with_instance_mut(|machine| machine.focus_next())
    );
}

#[test]
fn wave_b_focus_test_085_direct_port_expected_red() {
    let fixture = BindingFocusFixture::before_frames("swappable_artboards_focus.riv", Some("Main"));
    binding_set_artboard(&fixture.view_model, "artboardProp", None);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(fixture.view_model.clone()));
    fixture.frames(2, 0.016);
    let manager = fixture.manager();
    binding_assert_focus_sequence(
        &fixture.machine,
        &manager,
        &["Main", "StaticNestWithFocusable"],
    );
    binding_focus_set_asset(&fixture, "artboardProp", "Swappable1");
    fixture.frames(1, 0.016);
    binding_assert_focus_sequence(
        &fixture.machine,
        &manager,
        &["Main", "Swappable1", "StaticNestWithFocusable"],
    );
}

use nuxie_runtime::source::transform_component::TransformComponent;

fn binding_follow_path_world_translation(asset: &str) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file(asset, &mut factory);
    let artboard = file.with_file(File::artboard).expect("authored artboard");
    let target = binding_source_find::<TransformComponent>(&artboard, "target");
    let rectangle = binding_source_find::<TransformComponent>(&artboard, "rect");
    Artboard::advance_handle(
        &artboard,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
    let components = |object: &CoreHandle| {
        object
            .with(|object| {
                object
                    .as_world_transform_component()
                    .expect("WorldTransformComponent")
                    .world_transform()
                    .decompose()
            })
            .expect("live transform")
    };
    let target_components = components(&target);
    let rect_components = components(&rectangle);
    assert_eq!(target_components.x(), rect_components.x());
    assert_eq!(target_components.y(), rect_components.y());
}

#[test]
fn wave_b_follow_path_constraint_test_001_direct_port_expected_red() {
    binding_follow_path_world_translation("follow_path.riv");
}

#[test]
fn wave_b_follow_path_constraint_test_002_direct_port_expected_red() {
    binding_follow_path_world_translation("follow_path_with_0_opacity.riv");
}

#[test]
fn wave_b_follow_path_constraint_test_003_direct_port_expected_red() {
    binding_follow_path_world_translation("follow_path_path_0_opacity.riv");
}

#[test]
fn wave_b_follow_path_constraint_test_004_direct_port_expected_red() {
    let fixture = BindingSilver::new("follow_path_shapes.riv");
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for _ in 0..60 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("follow_path_animate_shape");
}

#[test]
fn wave_b_follow_path_constraint_test_005_direct_port_expected_red() {
    let fixture = BindingSilver::new("follow_path_solos.riv");
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for _ in 0..240 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("follow_path_animate_solo");
}

#[test]
fn wave_b_follow_path_constraint_test_006_direct_port_expected_red() {
    let fixture = BindingSilver::new("follow_path_path.riv");
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for _ in 0..120 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("follow_path_animate_target");
}

#[test]
fn wave_b_follow_path_constraint_test_007_direct_port_expected_red() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let file = binding_file("text_follow_path_shape_length.riv", &mut silver);
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let instance = binding_default_instance(&file, &artboard);
    artboard.bind_view_model_instance(Some(instance));
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let fixture = BindingSilver {
        machine,
        artboard,
        file,
        silver,
    };
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for _ in 0..10 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("text_follow_path_shape_length");
}

#[test]
fn wave_b_follow_path_constraint_test_008_direct_port_expected_red() {
    let fixture = BindingSilver::new("follow_path_constraint.riv");
    let instance = fixture
        .file
        .with_file_mut(|file| {
            file.create_view_model_instance_for_artboard(fixture.artboard.core_handle())
        })
        .expect("fresh view model instance");
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    for _ in 0..(1.0_f32 / 0.16_f32) as i32 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.16);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("follow_path_constraint");
}

thread_local! {
    // The pinned fallback vector is test fixture state. Its fonts are the
    // actual production owners; thread scoping isolates concurrent Rust tests.
    static BINDING_FALLBACK_FONTS: RefCell<Vec<FontRef>> = const { RefCell::new(Vec::new()) };
}

fn binding_pick_fallback_font(missing: u32, fallback_index: u32, _: &dyn Font) -> Option<FontRef> {
    if fallback_index > 0 {
        return None;
    }
    BINDING_FALLBACK_FONTS.with(|fonts| {
        fonts
            .borrow()
            .iter()
            .skip(fallback_index as usize)
            .find(|font| font.has_glyph(missing))
            .cloned()
    })
}

fn binding_load_font(name: &str) -> FontRef {
    let path = binding_path(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned font {}: {error}", path.display()));
    HbFont::decode(&bytes).expect("pinned font decodes")
}

fn binding_font_tag_to_string(tag: u32) -> String {
    tag.to_be_bytes().into_iter().map(char::from).collect()
}

#[test]
fn wave_b_font_test_001_direct_port_expected_red() {
    let test_cases = [
        (
            "assets/fonts/AdventPro-VariableFont_wdth,wght.ttf",
            400,
            false,
        ),
        ("assets/fonts/Inter_18pt-Regular.ttf", 400, false),
        ("assets/fonts/Inter_28pt-Bold.ttf", 700, false),
        ("assets/fonts/OpenSans-Italic.ttf", 400, true),
        ("assets/fonts/OpenSans-ExtraBoldItalic.ttf", 800, true),
    ];
    for (font_path, expected_weight, expected_italic) in test_cases {
        let font = binding_load_font(font_path);
        assert_eq!(font.get_weight(), expected_weight, "{font_path}");
        assert_eq!(font.is_italic(), expected_italic, "{font_path}");
    }
}

#[test]
fn wave_b_font_test_002_direct_port_expected_red() {
    for path in [
        "assets/fonts/Inter_18pt-Regular.ttf",
        "assets/Montserrat.ttf",
    ] {
        let font = binding_load_font(path);
        let metrics = font.line_metrics();
        assert!(metrics.cap_height < 0.0, "{path}");
        assert!(metrics.cap_height >= metrics.ascent, "{path}");
        assert!(metrics.x_height > metrics.cap_height, "{path}");
        assert!(metrics.x_height < 0.0, "{path}");
        assert!((font.cap_height(20.0) - metrics.cap_height * 20.0).abs() <= f32::EPSILON);
        assert!((font.x_height(20.0) - metrics.x_height * 20.0).abs() <= f32::EPSILON);
    }
}

#[test]
fn wave_b_font_test_003_direct_port_expected_red() {
    BINDING_FALLBACK_FONTS.with(|fonts| assert!(fonts.borrow().is_empty()));
    let font = binding_load_font("assets/RobotoFlex.ttf");
    let fallback_font = binding_load_font("assets/IBMPlexSansArabic-Regular.ttf");
    BINDING_FALLBACK_FONTS.with(|fonts| fonts.borrow_mut().push(fallback_font));

    let unichars = "لمفاتيح ABC DEF".chars().map(u32::from).collect::<Vec<_>>();
    let runs = [TextRun {
        font: Some(font.clone()),
        size: 32.0,
        line_height: -1.0,
        letter_spacing: 0.0,
        unichar_count: unichars.len() as u32,
        script: 0,
        style_id: 0,
        level: 0,
    }];
    let mut paragraphs = with_host_fallback_proc(binding_pick_fallback_font, || {
        font.shape_text(&unichars, &runs, -1)
    });
    assert_eq!(paragraphs.len(), 1);
    paragraphs = Vec::new();
    assert!(paragraphs.is_empty());
    BINDING_FALLBACK_FONTS.with(|fonts| fonts.borrow_mut().clear());
}

#[test]
fn wave_b_font_test_004_direct_port_expected_red() {
    let fallback_fonts = Vec::<FontRef>::new();
    assert!(fallback_fonts.is_empty());
    let font = binding_load_font("assets/RobotoFlex.ttf");

    let mut has_weight = false;
    for index in 0..font.get_axis_count() {
        let axis = font.get_axis(index);
        if axis.tag == 2_003_265_652 {
            assert_eq!(axis.def, 400.0);
            has_weight = true;
            break;
        }
    }
    assert!(has_weight);
    assert_eq!(font.get_axis_value(2_003_265_652), 400.0);
    assert_eq!(font.get_axis_value(2_003_072_104), 100.0);

    let varied = font.make_at_coords(&[Coord {
        axis: 2_003_265_652,
        value: 800.0,
    }]);
    assert_eq!(varied.get_axis_value(2_003_265_652), 800.0);
    let varied_twice = varied.make_at_coords(&[Coord {
        axis: 2_003_072_104,
        value: 122.0,
    }]);
    assert_eq!(varied_twice.get_axis_value(2_003_072_104), 122.0);
    assert_eq!(varied_twice.get_axis_value(2_003_265_652), 800.0);
}

#[test]
fn wave_b_font_test_005_direct_port_expected_red() {
    let fallback_fonts = Vec::<FontRef>::new();
    assert!(fallback_fonts.is_empty());
    let font = binding_load_font("assets/RobotoFlex.ttf");
    let feature_strings = font
        .features()
        .iter()
        .copied()
        .map(binding_font_tag_to_string)
        .collect::<Vec<_>>();
    assert_eq!(font.features().len(), 7);
    assert!(feature_strings.iter().any(|tag| tag == "mkmk"));
    assert!(feature_strings.iter().any(|tag| tag == "kern"));
    assert!(feature_strings.iter().any(|tag| tag == "rvrn"));
    assert!(feature_strings.iter().any(|tag| tag == "mark"));
    assert!(feature_strings.iter().any(|tag| tag == "locl"));
    assert!(feature_strings.iter().any(|tag| tag == "pnum"));
    assert!(feature_strings.iter().any(|tag| tag == "liga"));
}

fn binding_gamepad_ready() -> BindingSilver {
    // Pinned `ReadRiveFile` registers a scripting VM whenever the file owns
    // ScriptAssets. The Rust host supplies that approved adapter explicitly.
    let fixture = binding_scripted_default_silver_configured("gamepad_test.riv", false);
    let view_model = binding_default_instance(&fixture.file, &fixture.artboard);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(view_model));
    fixture.machine.advance_and_apply(0.0);
    fixture
}

struct BindingGamepadWire {
    bytes: Vec<u8>,
}

impl BindingGamepadWire {
    fn new() -> Self {
        Self {
            bytes: nuxie_runtime::GAMEPAD_BATCH_WIRE_VERSION
                .to_le_bytes()
                .to_vec(),
        }
    }

    fn connected(&mut self, device_id: i32) {
        self.connected_with_shape(device_id, 17, 4, 0);
    }

    fn connected_with_shape(
        &mut self,
        device_id: i32,
        button_count: u8,
        axis_count: u8,
        mapping: u8,
    ) {
        self.bytes.push(0);
        self.bytes.extend_from_slice(&device_id.to_le_bytes());
        self.bytes
            .extend_from_slice(&[mapping, button_count, axis_count, 0]);
        self.bytes
            .resize(self.bytes.len() + usize::from(button_count) * 4, 0);
        self.bytes
            .resize(self.bytes.len() + usize::from(axis_count) * 4, 0);
    }

    fn update(&mut self, device_id: i32, kind: u8, index: u8, value: f32) {
        self.bytes.push(1);
        self.bytes.extend_from_slice(&device_id.to_le_bytes());
        self.bytes.push(1);
        self.bytes.extend_from_slice(&[kind, index]);
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn disconnected(&mut self, device_id: i32) {
        self.bytes.push(2);
        self.bytes.extend_from_slice(&device_id.to_le_bytes());
    }
}

#[test]
fn wave_b_gamepad_test_001_direct_port_expected_red() {
    let fixture = binding_gamepad_ready();
    let mut wire = BindingGamepadWire::new();
    wire.connected_with_shape(0, 17, 4, 0);
    assert_eq!(wire.bytes.len(), 4 + 1 + 4 + 4 + 17 * 4 + 4 * 4);
    assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
}

#[test]
fn wave_b_gamepad_test_007_direct_port_expected_red() {
    let fixture = binding_gamepad_ready();
    let (width, height) = fixture
        .artboard
        .with_artboard(|artboard| (artboard.width(), artboard.height()));
    fixture
        .silver
        .borrow_mut()
        .frame_size(width as u32, height as u32);

    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();

    let device_id_1 = 3;
    let device_id_2 = 5;
    let device_id_3 = 1;

    {
        let mut wire = BindingGamepadWire::new();
        wire.connected(device_id_1);
        assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    {
        fixture.silver.borrow_mut().add_frame();
        let mut wire = BindingGamepadWire::new();
        wire.update(device_id_1, 0, 0, 1.0);
        assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    {
        fixture.silver.borrow_mut().add_frame();
        let mut wire = BindingGamepadWire::new();
        wire.connected(device_id_2);
        wire.connected_with_shape(device_id_3, 17, 4, 1);
        assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    {
        fixture.silver.borrow_mut().add_frame();
        let mut wire = BindingGamepadWire::new();
        wire.update(device_id_2, 0, 2, 1.0);
        wire.update(device_id_3, 0, 2, 1.0);
        assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    for index in 1..10 {
        fixture.silver.borrow_mut().add_frame();
        let mut wire = BindingGamepadWire::new();
        wire.update(device_id_1, 1, 0, index as f32 * 0.1);
        wire.update(device_id_2, 1, 1, index as f32 * -0.1);
        assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    {
        fixture.silver.borrow_mut().add_frame();
        let mut wire = BindingGamepadWire::new();
        wire.disconnected(device_id_3);
        assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    {
        fixture
            .machine
            .with_instance(|machine| machine.focus_manager())
            .with_focus_manager_mut(FocusManager::focus_next);
        fixture.silver.borrow_mut().add_frame();
        let mut wire = BindingGamepadWire::new();
        wire.update(device_id_1, 0, 0, 0.0);
        assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    {
        fixture.silver.borrow_mut().add_frame();
        let mut wire = BindingGamepadWire::new();
        wire.update(device_id_1, 0, 0, 1.0);
        assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    {
        fixture.silver.borrow_mut().add_frame();
        let mut wire = BindingGamepadWire::new();
        wire.update(device_id_1, 0, 0, 0.0);
        assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    {
        fixture.silver.borrow_mut().add_frame();
        let mut wire = BindingGamepadWire::new();
        wire.update(device_id_1, 0, 1, 1.0);
        assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    {
        fixture.silver.borrow_mut().add_frame();
        let mut wire = BindingGamepadWire::new();
        wire.update(device_id_1, 1, 0, 0.5);
        wire.update(device_id_1, 1, 1, 0.5);
        assert!(fixture.machine.submit_gamepads_from_buffer(&wire.bytes));
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }

    fixture.matches("gamepad_test");
}

use nuxie_runtime::source::{
    data_bind::data_context::{DataContext, RuntimeDataContextHandle},
    view_model_type::ViewModelType,
    viewmodel::viewmodel::ViewModel,
};

fn binding_global_fixture() -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("global_variables_test.riv", &mut factory);
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    (file, artboard)
}

fn binding_global_model(instance: &CoreHandle) -> CoreHandle {
    instance
        .with_downcast::<ViewModelInstance, _>(ViewModelInstance::get_view_model)
        .flatten()
        .expect("instance's view model")
}

fn binding_global_model_name(model: &CoreHandle) -> String {
    model
        .with_downcast::<ViewModel, _>(|model| model.base.name().to_owned())
        .expect("ViewModel")
}

fn binding_global_model_type(model: &CoreHandle) -> u32 {
    model
        .with_downcast::<ViewModel, _>(|model| model.base.view_model_type())
        .expect("ViewModel")
}

// Exact observation helper from global_view_model_binding_test.cpp: null
// contexts yield no names; null instances or models contribute an empty name.
fn binding_global_bound_names(context: Option<&RuntimeDataContextHandle>) -> Vec<String> {
    let Some(context) = context else {
        return Vec::new();
    };
    context
        .with_context(|context| context.view_model_instances().to_vec())
        .into_iter()
        .map(|instance| {
            instance
                .and_then(|instance| {
                    instance
                        .with_downcast::<ViewModelInstance, _>(ViewModelInstance::get_view_model)
                        .flatten()
                })
                .map(|model| binding_global_model_name(&model))
                .unwrap_or_default()
        })
        .collect()
}

#[test]
fn wave_b_global_view_model_binding_test_001_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("global_variables_test.riv", &mut factory);
    let names = file.with_file(File::global_view_model_names);
    assert!(!names.is_empty());
    for name in names {
        let model = file
            .with_file(|file| file.view_model_named(&name))
            .expect("named global model");
        assert_eq!(
            binding_global_model_type(&model),
            ViewModelType::Global as u32
        );
    }
}

#[test]
fn wave_b_global_view_model_binding_test_002_direct_port_expected_red() {
    let (_file, artboard) = binding_global_fixture();
    assert!(artboard.data_context().is_none());
}

#[test]
fn wave_b_global_view_model_binding_test_003_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let names = file.with_file(File::global_view_model_names);
    assert!(!names.is_empty());
    let target = &names[0];
    assert!(
        artboard
            .with_artboard(|artboard| artboard.global_view_model_instance(target))
            .is_none()
    );
    let instance = binding_default_named_instance(&file, target);
    assert!(artboard.with_artboard_mut(|artboard| {
        artboard.set_global_view_model_instance(target, Some(instance.clone()))
    }));
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.global_view_model_instance(target)),
        Some(instance.clone())
    );
    assert!(!artboard.with_artboard_mut(|artboard| {
        artboard.set_global_view_model_instance("not-a-global", Some(instance))
    }));
    assert!(
        artboard
            .with_artboard(|artboard| artboard.global_view_model_instance("not-a-global"))
            .is_none()
    );
}

#[test]
fn wave_b_global_view_model_binding_test_004_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let names = file.with_file(File::global_view_model_names);
    assert!(!names.is_empty());
    let main = binding_default_instance(&file, &artboard);
    let main_model = binding_global_model(&main);
    assert_ne!(
        binding_global_model_type(&main_model),
        ViewModelType::Global as u32
    );
    Artboard::set_view_model_instance_handle(&artboard.core_handle(), main);
    for name in &names {
        let global = binding_default_named_instance(&file, name);
        assert!(artboard.with_artboard_mut(|artboard| {
            artboard.set_global_view_model_instance(name, Some(global))
        }));
    }
    let mut expected = vec![binding_global_model_name(&main_model)];
    expected.extend(names);
    assert_eq!(
        binding_global_bound_names(artboard.data_context().as_ref()),
        expected
    );
    artboard.bind();
    assert_eq!(
        binding_global_bound_names(artboard.data_context().as_ref()),
        expected
    );
}

#[test]
fn wave_b_global_view_model_binding_test_005_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let names = file.with_file(File::global_view_model_names);
    assert!(names.len() >= 2);
    for name in names.iter().rev() {
        let global = binding_default_named_instance(&file, name);
        assert!(artboard.with_artboard_mut(|artboard| {
            artboard.set_global_view_model_instance(name, Some(global))
        }));
    }
    assert_eq!(
        binding_global_bound_names(artboard.data_context().as_ref()),
        names
    );
    let main = file.with_file_mut(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    });
    if let Some(main) = main {
        let model = binding_global_model(&main);
        if binding_global_model_type(&model) != ViewModelType::Global as u32 {
            Artboard::set_view_model_instance_handle(&artboard.core_handle(), main);
            let mut expected = vec![binding_global_model_name(&model)];
            expected.extend(names);
            assert_eq!(
                binding_global_bound_names(artboard.data_context().as_ref()),
                expected
            );
        }
    }
}

#[test]
fn wave_b_global_view_model_binding_test_006_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let names = file.with_file(File::global_view_model_names);
    assert!(!names.is_empty());
    let main = binding_default_instance(&file, &artboard);
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(main));
    for name in &names {
        assert!(
            machine
                .with_instance(|machine| machine.global_view_model_instance(name))
                .is_some()
        );
    }
    let context = machine
        .with_instance(|machine| machine.data_context())
        .expect("bound data context");
    assert_eq!(
        binding_global_bound_names(Some(&context)).len(),
        names.len() + 1
    );
}

#[test]
fn wave_b_global_view_model_binding_test_007_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let names = file.with_file(File::global_view_model_names);
    assert!(names.len() >= 2);
    let slot_a = &names[0];
    let model_b = &names[1];
    let override_instance = binding_default_named_instance(&file, model_b);
    assert_eq!(
        binding_global_model_name(&binding_global_model(&override_instance)),
        *model_b
    );
    assert!(artboard.with_artboard_mut(|artboard| {
        artboard.set_global_view_model_instance(slot_a, Some(override_instance.clone()))
    }));
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.global_view_model_instance(slot_a)),
        Some(override_instance)
    );
    assert!(
        artboard
            .with_artboard(|artboard| artboard.global_view_model_instance(model_b))
            .is_none()
    );
}

#[test]
fn wave_b_global_view_model_binding_test_008_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let global_names = file.with_file(File::global_view_model_names);
    assert!(!global_names.is_empty());
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    assert!(
        machine
            .with_instance(|machine| machine.global_view_model_instance(&global_names[0]))
            .is_none()
    );
    let main = binding_default_instance(&file, &artboard);
    machine.with_instance_mut(|machine| machine.set_view_model_instance(main.clone()));
    for name in &global_names {
        let global = binding_default_named_instance(&file, name);
        assert!(
            machine
                .with_instance_mut(|machine| machine.set_global_view_model_instance(name, global))
        );
    }
    machine.with_instance_mut(|machine| machine.bind());
    let context = machine
        .with_instance(|machine| machine.data_context())
        .expect("bound data context");
    let names = binding_global_bound_names(Some(&context));
    assert_eq!(names.len(), global_names.len() + 1);
    assert_eq!(
        names[0],
        binding_global_model_name(&binding_global_model(&main))
    );
    let fetched = machine
        .with_instance(|machine| machine.global_view_model_instance(&global_names[0]))
        .expect("global instance");
    assert_eq!(
        binding_global_model_name(&binding_global_model(&fetched)),
        global_names[0]
    );
    let custom = binding_default_named_instance(&file, &global_names[0]);
    assert!(machine.with_instance_mut(|machine| {
        machine.set_global_view_model_instance(&global_names[0], custom.clone())
    }));
    assert_eq!(
        machine.with_instance(|machine| machine.global_view_model_instance(&global_names[0])),
        Some(custom)
    );
    assert_eq!(
        binding_global_bound_names(
            machine
                .with_instance(|machine| machine.data_context())
                .as_ref()
        ),
        names
    );
}

#[test]
fn wave_b_global_view_model_binding_test_009_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let mut non_global = String::new();
    for index in 0..file.with_file(File::view_model_count) {
        if let Some(model) = file.with_file(|file| file.view_model(index)) {
            if binding_global_model_type(&model) != ViewModelType::Global as u32 {
                non_global = binding_global_model_name(&model);
                break;
            }
        }
    }
    assert!(!non_global.is_empty());
    let instance = binding_default_named_instance(&file, &non_global);
    assert!(!artboard.with_artboard_mut(|artboard| {
        artboard.set_global_view_model_instance(&non_global, Some(instance.clone()))
    }));
    assert!(
        artboard
            .with_artboard(|artboard| artboard.global_view_model_instance(&non_global))
            .is_none()
    );
    assert!(!machine.with_instance_mut(|machine| {
        machine.set_global_view_model_instance(&non_global, instance)
    }));
    assert!(
        machine
            .with_instance(|machine| machine.global_view_model_instance(&non_global))
            .is_none()
    );
}

#[test]
fn wave_b_global_view_model_binding_test_010_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let names = file.with_file(File::global_view_model_names);
    assert!(!names.is_empty());
    assert!(
        machine
            .with_instance(|machine| machine.data_context())
            .is_none()
    );
    machine.with_instance_mut(|machine| machine.bind());
    let context = machine
        .with_instance(|machine| machine.data_context())
        .expect("completed data context");
    assert_eq!(
        binding_global_bound_names(Some(&context)).len(),
        names.len() + 1
    );
    for name in &names {
        assert!(
            machine
                .with_instance(|machine| machine.global_view_model_instance(name))
                .is_some()
        );
    }
}

#[test]
fn wave_b_global_view_model_binding_test_011_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let names = file.with_file(File::global_view_model_names);
    assert!(names.len() >= 2);
    for name in &names {
        let global = binding_default_named_instance(&file, name);
        assert!(
            machine
                .with_instance_mut(|machine| machine.set_global_view_model_instance(name, global))
        );
    }
    assert!(
        machine
            .with_instance(|machine| machine.global_view_model_instance(&names[0]))
            .is_some()
    );
    assert_eq!(
        binding_global_bound_names(
            machine
                .with_instance(|machine| machine.data_context())
                .as_ref()
        )
        .len(),
        names.len()
    );
    assert!(
        machine
            .with_instance_mut(|machine| machine.set_global_view_model_instance(&names[0], None))
    );
    assert!(
        machine
            .with_instance(|machine| machine.global_view_model_instance(&names[0]))
            .is_none()
    );
    for name in names.iter().skip(1) {
        assert!(
            machine
                .with_instance(|machine| machine.global_view_model_instance(name))
                .is_some()
        );
    }
    assert_eq!(
        binding_global_bound_names(
            machine
                .with_instance(|machine| machine.data_context())
                .as_ref()
        )
        .len(),
        names.len() - 1
    );
}

#[test]
fn wave_b_global_view_model_binding_test_012_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let global_names = file.with_file(File::global_view_model_names);
    assert!(!global_names.is_empty());
    let global = binding_default_named_instance(&file, &global_names[0]);
    assert!(machine.with_instance_mut(|machine| {
        machine.set_global_view_model_instance(&global_names[0], global)
    }));
    let context = machine
        .with_instance(|machine| machine.data_context())
        .expect("pre-bind data context");
    assert!(
        context
            .with_context(DataContext::main_view_model_instance)
            .is_none()
    );
    machine.with_instance_mut(|machine| machine.bind());
    // Keep the pre-bind context: bind must complete this same retained owner.
    let main = context
        .with_context(DataContext::main_view_model_instance)
        .expect("completed main");
    let names = binding_global_bound_names(Some(&context));
    assert_eq!(names.len(), global_names.len() + 1);
    assert_eq!(
        names[0],
        binding_global_model_name(&binding_global_model(&main))
    );
}

#[test]
fn wave_b_global_view_model_binding_test_013_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let names = file.with_file(File::global_view_model_names);
    assert!(!names.is_empty());
    assert!(
        machine
            .with_instance(|machine| machine.data_context())
            .is_none()
    );
    assert!(
        machine
            .with_instance_mut(|machine| machine.set_global_view_model_instance(&names[0], None))
    );
    assert!(
        machine
            .with_instance(|machine| machine.data_context())
            .is_none()
    );
    assert!(
        !machine.with_instance_mut(
            |machine| machine.set_global_view_model_instance("not-a-global", None)
        )
    );
}

#[test]
fn wave_b_global_view_model_binding_test_014_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let names = file.with_file(File::global_view_model_names);
    assert!(names.len() >= 2);
    for name in &names {
        let global = binding_default_named_instance(&file, name);
        assert!(artboard.with_artboard_mut(|artboard| {
            artboard.set_global_view_model_instance(name, Some(global))
        }));
    }
    assert!(
        artboard
            .with_artboard(|artboard| artboard.global_view_model_instance(&names[0]))
            .is_some()
    );
    assert_eq!(
        binding_global_bound_names(artboard.data_context().as_ref()).len(),
        names.len()
    );
    assert!(
        artboard
            .with_artboard_mut(|artboard| artboard.set_global_view_model_instance(&names[0], None))
    );
    assert!(
        artboard
            .with_artboard(|artboard| artboard.global_view_model_instance(&names[0]))
            .is_none()
    );
    for name in names.iter().skip(1) {
        assert!(
            artboard
                .with_artboard(|artboard| artboard.global_view_model_instance(name))
                .is_some()
        );
    }
    assert_eq!(
        binding_global_bound_names(artboard.data_context().as_ref()).len(),
        names.len() - 1
    );
}

#[test]
fn wave_b_global_view_model_binding_test_015_direct_port_expected_red() {
    let (file, artboard) = binding_global_fixture();
    let names = file.with_file(File::global_view_model_names);
    assert!(!names.is_empty());
    assert!(artboard.data_context().is_none());
    assert!(
        artboard
            .with_artboard_mut(|artboard| artboard.set_global_view_model_instance(&names[0], None))
    );
    assert!(artboard.data_context().is_none());
    assert!(!artboard.with_artboard_mut(|artboard| {
        artboard.set_global_view_model_instance("not-a-global", None)
    }));
}

#[test]
fn wave_b_global_viewmodels_test_001_direct_port_expected_red() {
    let fixture = BindingSilver::new("global_variables_test.riv");
    let main = binding_default_instance(&fixture.file, &fixture.artboard);
    fixture
        .machine
        .with_instance_mut(|machine| machine.set_view_model_instance(main));
    for name in fixture.file.with_file(File::global_view_model_names) {
        let global = binding_default_named_instance(&fixture.file, &name);
        assert!(
            fixture
                .machine
                .with_instance_mut(|machine| machine.set_global_view_model_instance(&name, global))
        );
    }
    fixture.machine.with_instance_mut(|machine| machine.bind());
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for _ in 0..(1.0_f32 / 0.016_f32) as i32 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("global_variables_test");
}

#[test]
fn wave_b_global_viewmodels_test_002_direct_port_expected_red() {
    let fixture = BindingSilver::new("global_viewmodels_test.riv");
    let main = binding_default_instance(&fixture.file, &fixture.artboard);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(main));
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("global_viewmodels_test-auto_instance");
}

#[test]
fn wave_b_global_viewmodels_test_003_direct_port_expected_red() {
    let fixture = BindingSilver::new("global_viewmodels_test.riv");
    let mut renderer = fixture.silver.borrow().make_renderer();
    {
        let main = binding_default_instance(&fixture.file, &fixture.artboard);
        let colors = binding_default_named_instance(&fixture.file, "GlobalColors");
        let c1 = binding_property(&colors, "c1");
        binding_set_color(&c1, (255 << 24) | (255 << 16) | (255 << 8));
        fixture
            .machine
            .with_instance_mut(|machine| machine.set_view_model_instance(main));
        fixture.machine.with_instance_mut(|machine| {
            machine.set_global_view_model_instance("GlobalColors", colors)
        });
        fixture.machine.with_instance_mut(|machine| machine.bind());
    }
    fixture.advance(0.0);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    {
        let main = binding_default_instance(&fixture.file, &fixture.artboard);
        binding_set_string(&binding_property(&main, "label"), "label updated");
        let colors = binding_default_named_instance(&fixture.file, "GlobalColors");
        let c1 = binding_property(&colors, "c1");
        binding_set_color(&c1, (255 << 24) | (255 << 8) | 255);
        fixture.machine.with_instance_mut(|machine| {
            machine.set_global_view_model_instance("GlobalColors", colors)
        });
        fixture
            .machine
            .with_instance_mut(|machine| machine.set_view_model_instance(main));
        fixture.machine.with_instance_mut(|machine| machine.bind());
    }
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("global_viewmodels_test-set_instance");
}

use nuxie_runtime::source::{
    animation::{
        animation_state::AnimationState, state_machine::StateMachine,
        state_machine_instance::StateMachineInstance,
    },
    math::{aabb::IAabb, hit_test::HitTester, path_types::FillRule},
};

fn binding_hit_machine(
    asset: &str,
    artboard_name: &str,
    machine_name: &str,
) -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    CoreHandle,
    RuntimeStateMachineInstanceHandle,
) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file(asset, &mut factory);
    let source = file
        .with_file(|file| file.artboard_named_source(artboard_name))
        .expect("named source artboard");
    let artboard = Artboard::instance_from_handle(&source).expect("artboard instance");
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.state_machine_count()),
        1
    );
    let definition = source
        .with_downcast::<Artboard, _>(|artboard| artboard.state_machine_named(machine_name))
        .flatten()
        .expect("named state machine");
    let machine = StateMachineInstance::new(definition.clone(), artboard.downgrade());
    (file, artboard, definition, machine)
}

fn binding_hit_initialize(
    artboard: &RuntimeArtboardInstanceHandle,
    machine: &RuntimeStateMachineInstanceHandle,
) {
    machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    artboard.advance_default(0.0);
    assert!(machine.with_instance(|machine| machine.needs_advance()));
    machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
}

fn binding_hit_bool(machine: &RuntimeStateMachineInstanceHandle, name: &str) -> bool {
    machine.with_instance(|machine| machine.get_bool(name).expect("boolean input").value())
}

fn binding_hit_move(machine: &RuntimeStateMachineInstanceHandle, x: f32, y: f32) {
    machine.with_instance_mut(|machine| machine.pointer_move(Vec2D::new(x, y), 0.0, 0));
}

fn binding_hit_down(machine: &RuntimeStateMachineInstanceHandle, x: f32, y: f32, id: i32) {
    machine.with_instance_mut(|machine| machine.pointer_down(Vec2D::new(x, y), id));
}

fn binding_hit_up(machine: &RuntimeStateMachineInstanceHandle, x: f32, y: f32, id: i32) {
    machine.with_instance_mut(|machine| machine.pointer_up(Vec2D::new(x, y), id));
}

fn binding_hit_exit(machine: &RuntimeStateMachineInstanceHandle, x: f32, y: f32, id: i32) {
    machine.with_instance_mut(|machine| machine.pointer_exit(Vec2D::new(x, y), id));
}

#[test]
fn wave_b_hittest_test_001_direct_port_expected_red() {
    let mut tester = HitTester::new();
    tester.reset_area(IAabb {
        left: 10,
        top: 10,
        right: 12,
        bottom: 12,
    });
    tester.move_to(Vec2D::new(0.0, 0.0));
    tester.line_to(Vec2D::new(20.0, 0.0));
    tester.line_to(Vec2D::new(20.0, 20.0));
    tester.line_to(Vec2D::new(0.0, 20.0));
    tester.close();
    assert!(tester.test(FillRule::NonZero));
    let points = [
        Vec2D::new(29.9785, 32.5261),
        Vec2D::new(231.102, 32.5261),
        Vec2D::new(231.102, 269.898),
        Vec2D::new(29.9785, 269.898),
    ];
    tester.reset_area(IAabb {
        left: 81,
        top: 156,
        right: 84,
        bottom: 159,
    });
    tester.move_to(points[0]);
    for point in points.iter().skip(1) {
        tester.line_to(*point);
    }
    tester.close();
    assert!(tester.test(FillRule::NonZero));
}

#[test]
fn wave_b_hittest_test_002_direct_port_expected_red() {
    let area = IAabb {
        left: 10,
        top: 10,
        right: 12,
        bottom: 12,
    };
    let vertices = [
        Vec2D::new(0.0, 0.0),
        Vec2D::new(20.0, 10.0),
        Vec2D::new(0.0, 20.0),
    ];
    assert!(HitTester::test_mesh_area(area, &vertices, &[0, 1, 2]));
}

#[test]
fn wave_b_hittest_test_003_direct_port_expected_red() {
    let (_file, artboard, _definition, machine) =
        binding_hit_machine("opaque_hit_test.riv", "main", "main-state-machine");
    binding_hit_initialize(&artboard, &machine);
    machine.with_instance(|machine| {
        assert!(machine.get_bool("toGreen").is_some());
        assert!(machine.get_bool("grayToggle").is_some());
    });
    binding_hit_down(&machine, 100.0, 50.0, 0);
    assert!(binding_hit_bool(&machine, "grayToggle"));
    assert!(!binding_hit_bool(&machine, "toGreen"));
    binding_hit_down(&machine, 100.0, 250.0, 0);
    assert!(!binding_hit_bool(&machine, "grayToggle"));
    assert!(binding_hit_bool(&machine, "toGreen"));
    binding_hit_down(&machine, 100.0, 110.0, 0);
    assert!(binding_hit_bool(&machine, "grayToggle"));
    assert!(!binding_hit_bool(&machine, "toGreen"));
}

#[test]
fn wave_b_hittest_test_004_direct_port_expected_red() {
    let (_file, artboard, _definition, machine) =
        binding_hit_machine("opaque_hit_test.riv", "second", "second-state-machine");
    let nested = binding_find::<NestedArtboard>(&artboard, "second-nested");
    let nested_animation = nested
        .with(|nested| nested.as_nested_artboard().unwrap().nested_animations()[0].clone())
        .unwrap();
    let nested_machine = nested_animation
        .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
        .flatten()
        .expect("nested state machine instance");
    assert!(nested_machine.with_instance(|machine| machine.get_bool("bool-target").is_some()));
    artboard.advance_default(0.0);
    machine.advance_and_apply(0.0);
    assert!(!binding_hit_bool(&nested_machine, "bool-target"));
    assert!(machine.with_instance(|machine| machine.get_bool("second-gray-toggle").is_some()));
    binding_hit_down(&machine, 100.0, 250.0, 0);
    assert!(binding_hit_bool(&machine, "second-gray-toggle"));
    binding_hit_down(&machine, 301.0, 50.0, 0);
    assert!(binding_hit_bool(&machine, "second-gray-toggle"));
    binding_hit_down(&machine, 100.0, 50.0, 0);
    assert!(binding_hit_bool(&machine, "second-gray-toggle"));
    assert!(binding_hit_bool(&nested_machine, "bool-target"));
    machine.advance_and_apply(1.0);
    machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
    binding_hit_down(&machine, 100.0, 50.0, 0);
    assert!(!binding_hit_bool(&machine, "second-gray-toggle"));
    assert!(binding_hit_bool(&nested_machine, "bool-target"));
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_hittest_test_005_direct_port_expected_red() {
    let (_file, artboard, _definition, machine) =
        binding_hit_machine("pointer_events.riv", "art-1", "sm-1");
    binding_hit_initialize(&artboard, &machine);
    assert_eq!(
        machine.with_instance(|machine| machine.hit_components_count()),
        4
    );
    let counts = || {
        machine.with_instance(|machine| {
            std::array::from_fn::<_, 4, _>(|i| {
                machine
                    .hit_component(i)
                    .expect("hit component")
                    .early_out_count()
            })
        })
    };
    assert_eq!(counts(), [0, 0, 0, 0]);
    binding_hit_move(&machine, 100.0, 250.0);
    assert_eq!(counts(), [1, 0, 0, 1]);
    binding_hit_exit(&machine, 100.0, 250.0, 0);
    assert_eq!(counts(), [2, 0, 0, 2]);
    binding_hit_down(&machine, 100.0, 250.0, 0);
    assert_eq!(counts(), [2, 0, 0, 2]);
    binding_hit_up(&machine, 100.0, 250.0, 0);
    assert_eq!(counts(), [2, 0, 0, 3]);
    binding_hit_move(&machine, 105.0, 205.0);
    assert_eq!(counts(), [3, 0, 0, 4]);
}

#[test]
fn wave_b_hittest_test_006_direct_port_expected_red() {
    let (_file, artboard, definition, machine) =
        binding_hit_machine("click_event.riv", "art-1", "sm-1");
    binding_hit_initialize(&artboard, &machine);
    assert_eq!(
        machine.with_instance(|machine| machine.hit_components_count()),
        2
    );
    assert_eq!(
        definition.with_downcast::<StateMachine, _>(StateMachine::layer_count),
        Some(1)
    );
    let events = || machine.with_instance(|machine| machine.reported_event_count());
    assert_eq!(events(), 0);
    binding_hit_down(&machine, 75.0, 75.0, 0);
    binding_hit_up(&machine, 75.0, 75.0, 0);
    assert_eq!(events(), 1);
    binding_hit_down(&machine, 75.0, 75.0, 0);
    binding_hit_up(&machine, 300.0, 75.0, 0);
    assert_eq!(events(), 1);
    binding_hit_down(&machine, 300.0, 75.0, 0);
    binding_hit_up(&machine, 75.0, 75.0, 0);
    assert_eq!(events(), 1);
    binding_hit_down(&machine, 75.0, 75.0, 0);
    binding_hit_up(&machine, 225.0, 225.0, 0);
    assert_eq!(events(), 2);
    binding_hit_down(&machine, 150.0, 150.0, 0);
    binding_hit_up(&machine, 150.0, 150.0, 0);
    assert_eq!(events(), 3);
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_hittest_test_007_direct_port_expected_red() {
    let (_file, artboard, definition, machine) =
        binding_hit_machine("click_event.riv", "art-2", "sm-1");
    binding_hit_initialize(&artboard, &machine);
    assert_eq!(
        machine.with_instance(|machine| machine.hit_components_count()),
        2
    );
    assert_eq!(
        definition.with_downcast::<StateMachine, _>(StateMachine::layer_count),
        Some(1)
    );
    for (x, expected) in [
        (75.0, "green"),
        (200.0, "green"),
        (400.0, "red"),
        (200.0, "green"),
    ] {
        binding_hit_move(&machine, x, 75.0);
        artboard.advance_default(0.0);
        machine.advance_and_apply(0.0);
        let state = machine
            .with_instance_mut(|machine| machine.layer_state(0))
            .expect("layer state");
        assert!(state.is_type_of(AnimationState::TYPE_KEY));
        let animation = state
            .with_downcast::<AnimationState, _>(AnimationState::animation)
            .flatten()
            .expect("state animation");
        assert_eq!(
            animation
                .with_downcast::<LinearAnimation, _>(|animation| animation.base.name().to_owned())
                .unwrap(),
            expected
        );
    }
}

fn binding_hit_hover_silver(artboard_name: &str, points: &[(f32, f32)], silver_name: &str) {
    let fixture = binding_asset_silver("hit_test_test.riv", Some(artboard_name));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for &(x, y) in points {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, x, y);
        fixture.advance(0.1);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches(silver_name);
}

#[test]
fn wave_b_hittest_test_008_direct_port_expected_red() {
    binding_hit_hover_silver("ab-1", &[(50.0, 150.0), (260.0, 150.0)], "hittest_ab1");
}

#[test]
fn wave_b_hittest_test_009_direct_port_expected_red() {
    binding_hit_hover_silver(
        "ab1-parent",
        &[(370.0, 110.0), (370.0, 180.0)],
        "hittest_ab1_parent",
    );
}

#[test]
fn wave_b_hittest_test_010_direct_port_expected_red() {
    binding_hit_hover_silver(
        "ab1-grand-parent",
        &[(370.0, 250.0), (370.0, 190.0), (510.0, 190.0)],
        "hittest_ab1_grand_parent",
    );
}

fn binding_hit_scroll_list(artboard_name: &str, silver_name: &str) {
    let fixture = binding_asset_silver("hit_test_test.riv", Some(artboard_name));
    let instance = fixture.instance();
    let scroll = binding_property(&instance, "scroll-offset");
    assert!(scroll.is_type_of(ViewModelInstanceNumber::TYPE_KEY));
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_set_number(&scroll, -100.0);
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    let mut coord = 200.0;
    while coord > 100.0 {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, 50.0, coord);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        coord -= 10.0;
    }
    coord = 75.0;
    fixture.silver.borrow_mut().add_frame();
    binding_hit_down(&fixture.machine, 50.0, coord, 0);
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    while coord > -500.0 {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, 50.0, coord);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        coord -= 20.0;
    }
    fixture.silver.borrow_mut().add_frame();
    binding_hit_up(&fixture.machine, 50.0, coord, 0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    coord = 110.0;
    while coord > -5.0 {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, 50.0, coord);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        coord -= 4.0;
    }
    fixture.matches(silver_name);
}

#[test]
fn wave_b_hittest_test_011_direct_port_expected_red() {
    binding_hit_scroll_list("ab-2-non-virtualized", "hittest_ab_2_non_virtualized");
}

#[test]
fn wave_b_hittest_test_012_direct_port_expected_red() {
    binding_hit_scroll_list("ab-2-virtualized", "hittest_ab_2_virtualized");
}

#[test]
fn wave_b_hittest_test_013_direct_port_expected_red() {
    let fixture = binding_asset_silver("hit_test_test.riv", Some("ab-text-parent"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let mut coord = 400.0;
    while coord < 550.0 {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, coord, 320.0);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        coord += 10.0;
    }
    coord = 200.0;
    while coord < 450.0 {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, 500.0, coord);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        coord += 10.0;
    }
    fixture.matches("hittest_ab_text_parent");
}

#[test]
fn wave_b_hittest_test_014_direct_port_expected_red() {
    let fixture = binding_asset_silver("hit_test_test.riv", Some("ab-shape-parent"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let mut coord = 0.0;
    while coord < 550.0 {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, 310.0, coord);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        coord += 20.0;
    }
    coord = 220.0;
    while coord < 530.0 {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, coord, 420.0);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        coord += 20.0;
    }
    fixture.matches("hittest_ab_shape_parent");
}

#[test]
fn wave_b_hittest_test_015_direct_port_expected_red() {
    let fixture = binding_asset_silver("hit_test_nested.riv", Some("Main"));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for (x, y) in [
        (150.0, 150.0),
        (300.0, 200.0),
        (100.0, 250.0),
        (400.0, 350.0),
    ] {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, x, y);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("hittest_nested");
}

#[test]
fn wave_b_hittest_test_016_direct_port_expected_red() {
    let fixture = binding_asset_silver("pointer_exit.riv", Some("main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let mut mouse_pos = 100.0;
    while mouse_pos <= 400.0 {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, mouse_pos, 250.0);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        mouse_pos += 30.0;
    }
    mouse_pos = 500.0;
    while mouse_pos > 100.0 {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, mouse_pos, 250.0);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        mouse_pos -= 30.0;
    }
    mouse_pos = 500.0;
    while mouse_pos > 100.0 {
        fixture.silver.borrow_mut().add_frame();
        binding_hit_move(&fixture.machine, 240.0, mouse_pos);
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
        mouse_pos -= 30.0;
    }
    fixture.matches("pointer_exit");
}

#[test]
fn wave_b_hittest_test_017_direct_port_expected_red() {
    let fixture = binding_asset_silver("multitouch.riv", Some("main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    for (down, id) in [
        (true, 1),
        (false, 1),
        (true, 1),
        (false, 0),
        (false, 1),
        (true, 1),
        (true, 0),
        (false, 0),
        (false, 1),
    ] {
        fixture.silver.borrow_mut().add_frame();
        if down {
            binding_hit_down(&fixture.machine, 200.0, 350.0, id);
        } else {
            binding_hit_up(&fixture.machine, 200.0, 350.0, id);
        }
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("multitouch");
}

fn binding_hit_multitouch_prefix(
    fixture: &BindingSilver,
    renderer: &mut dyn nuxie_render_api::Renderer,
) {
    fixture.silver.borrow_mut().add_frame();
    binding_hit_down(&fixture.machine, 122.5845, 443.8406, 9);
    fixture.advance(0.016);
    fixture.artboard.draw(renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_hit_down(&fixture.machine, 459.5410, 188.4058, 8);
    binding_hit_down(&fixture.machine, 333.3333, 248.1884, 7);
    fixture.advance(0.016);
    fixture.artboard.draw(renderer);
    fixture.silver.borrow_mut().add_frame();
    for (x, y, id) in [
        (459.5410, 188.4058, 8),
        (123.7923, 444.4445, 9),
        (333.3333, 248.1884, 7),
    ] {
        binding_hit_up(&fixture.machine, x, y, id);
        binding_hit_exit(&fixture.machine, x, y, id);
    }
    fixture.advance(0.016);
    fixture.artboard.draw(renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_hit_down(&fixture.machine, 118.9613, 439.6135, 7);
    binding_hit_down(&fixture.machine, 346.6183, 269.9276, 9);
    binding_hit_down(&fixture.machine, 459.5410, 194.4444, 8);
    fixture.advance(0.016);
    fixture.artboard.draw(renderer);
    fixture.silver.borrow_mut().add_frame();
    for (x, y, id) in [
        (346.6183, 269.9276, 9),
        (122.5845, 440.8212, 7),
        (459.5410, 194.4444, 8),
    ] {
        binding_hit_up(&fixture.machine, x, y, id);
        binding_hit_exit(&fixture.machine, x, y, id);
    }
    fixture.advance(0.016);
    fixture.artboard.draw(renderer);
}

#[test]
fn wave_b_hittest_test_018_direct_port_expected_red() {
    let fixture = binding_asset_silver("multitouch_enter.riv", Some("Main"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    binding_hit_multitouch_prefix(&fixture, &mut renderer);
    fixture.matches("multitouch_enter");
}

#[test]
fn wave_b_hittest_test_019_direct_port_expected_red() {
    let fixture = binding_asset_silver("multitouch_enter.riv", Some("MainList"));
    let instance = fixture.instance();
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    binding_hit_multitouch_prefix(&fixture, &mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.machine.with_instance_mut(|machine| {
        machine.pointer_move(Vec2D::new(50.0, 300.0), 0.0, 7);
        machine.pointer_move(Vec2D::new(250.0, 200.0), 0.0, 8);
    });
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    let mut x_offset = 0.0;
    while x_offset < 300.0 {
        x_offset += 20.0;
        fixture.silver.borrow_mut().add_frame();
        fixture.machine.with_instance_mut(|machine| {
            machine.pointer_move(Vec2D::new(50.0 + x_offset, 300.0), 0.0, 7);
            machine.pointer_move(Vec2D::new(250.0 + x_offset, 200.0), 0.0, 8);
        });
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("multitouch_enter-MainList");
}

#[test]
fn wave_b_hittest_test_020_direct_port_expected_red() {
    let fixture = binding_asset_silver("multitouch_enter.riv", Some("MultiScroll"));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    let mut y_offset = 400.0;
    binding_hit_down(&fixture.machine, 50.0, y_offset, 7);
    binding_hit_down(&fixture.machine, 350.0, y_offset, 8);
    while y_offset > 0.0 {
        y_offset -= 20.0;
        fixture.silver.borrow_mut().add_frame();
        fixture.machine.with_instance_mut(|machine| {
            machine.pointer_move(Vec2D::new(50.0, y_offset), 0.0, 7);
            machine.pointer_move(Vec2D::new(350.0, y_offset), 0.0, 8);
        });
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    binding_hit_up(&fixture.machine, 50.0, y_offset, 7);
    binding_hit_up(&fixture.machine, 350.0, y_offset, 8);
    fixture.matches("multitouch_enter-MultiScroll");
}

#[test]
fn wave_b_hittest_test_021_direct_port_expected_red() {
    let fixture = BindingSilver::new("hittest_collapsed_layouts.riv");
    let instance = binding_default_instance(&fixture.file, &fixture.artboard);
    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_hit_down(&fixture.machine, 250.0, 50.0, 0);
    binding_hit_up(&fixture.machine, 250.0, 50.0, 0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    binding_hit_down(&fixture.machine, 250.0, 50.0, 0);
    binding_hit_up(&fixture.machine, 250.0, 50.0, 0);
    fixture.advance(0.016);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("hittest_collapsed_layouts");
}

#[cfg(feature = "tools")]
use nuxie_runtime::source::{
    assets::image_asset::ImageAsset, file_asset_loader::FileAssetLoaderRef,
    relative_local_asset_loader::RelativeLocalAssetLoader,
};
use nuxie_runtime::source::{
    bones::{bone::Bone, skin::Skin},
    constraints::ik_constraint::IKConstraint,
};

#[test]
fn wave_b_ik_constraint_test_001_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("complex_ik_dependency.riv", &mut factory);
    let artboard = file.with_file(File::artboard).expect("authored artboard");
    let one = binding_source_find::<Bone>(&artboard, "One");
    let two = binding_source_find::<Bone>(&artboard, "Two");
    let skin = artboard
        .with_downcast::<Artboard, _>(|artboard| {
            artboard
                .objects()
                .iter()
                .flatten()
                .find(|object| object.is_type_of(Skin::TYPE_KEY))
                .cloned()
        })
        .flatten()
        .expect("Skin");
    let first_constraint = two
        .with(|object| {
            object
                .as_transform_component()
                .expect("Bone transform")
                .constraints()[0]
                .clone()
        })
        .expect("live bone");
    assert!(first_constraint.is_type_of(IKConstraint::TYPE_KEY));
    assert!(binding_file_graph_order(&skin) > binding_file_graph_order(&one));
    assert!(binding_file_graph_order(&skin) > binding_file_graph_order(&two));
}

fn binding_ik_world_about_equal(bone: &CoreHandle, expected: [f32; 6]) {
    let actual = bone
        .with(|object| {
            *object
                .as_world_transform_component()
                .expect("bone world transform")
                .world_transform()
        })
        .expect("live bone");
    // The pinned rive_testing.cpp aboutEqual compares each matrix entry to 1e-4.
    for index in 0..6 {
        assert!(
            !((actual[index] - expected[index]).abs() > 0.0001),
            "matrix entry {index}: {} != {}",
            actual[index],
            expected[index],
        );
    }
}

fn binding_two_bone_ik(iterations: usize) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("two_bone_ik.riv", &mut factory);
    let artboard = file.with_file(File::artboard).expect("authored artboard");
    let circle_a = binding_source_find::<Shape>(&artboard, "circle a");
    let circle_b = binding_source_find::<Shape>(&artboard, "circle b");
    let bone_a = binding_source_find::<Bone>(&artboard, "a");
    let bone_b = binding_source_find::<Bone>(&artboard, "b");
    let target = binding_source_find::<Node>(&artboard, "target");
    let animation = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.animation_named("Animation 1"))
        .flatten()
        .expect("Animation 1");
    let dependents = bone_b
        .with(|object| {
            object
                .as_component()
                .expect("Bone component")
                .dependents()
                .to_vec()
        })
        .expect("live bone");
    assert!(dependents.contains(&circle_a.into()));
    assert!(dependents.contains(&circle_b.into()));

    for _ in 0..iterations {
        animation
            .with_downcast_mut::<LinearAnimation, _>(|animation| {
                artboard
                    .with_downcast_mut::<Artboard, _>(|artboard| {
                        animation.apply(artboard, 0.0, 1.0, None)
                    })
                    .expect("Artboard");
            })
            .expect("LinearAnimation");
        Artboard::advance_handle(
            &artboard,
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
        let position = target
            .with_downcast::<Node, _>(|node| (node.base.x(), node.base.y()))
            .expect("target Node");
        assert_eq!(position.0, 296.0);
        assert_eq!(position.1, 202.0);
        binding_ik_world_about_equal(
            &bone_a,
            [
                0.11632211506366729736328125,
                -0.993211567401885986328125,
                0.993211567401885986328125,
                0.11632211506366729736328125,
                26.015254974365234375,
                475.2149658203125,
            ],
        );
        binding_ik_world_about_equal(
            &bone_b,
            [
                0.974071562290191650390625,
                0.2262403070926666259765625,
                -0.2262403070926666259765625,
                0.974071562290191650390625,
                64.31568145751953125,
                148.1883544921875,
            ],
        );

        animation
            .with_downcast_mut::<LinearAnimation, _>(|animation| {
                artboard
                    .with_downcast_mut::<Artboard, _>(|artboard| {
                        animation.apply(artboard, 1.0, 1.0, None)
                    })
                    .expect("Artboard");
            })
            .expect("LinearAnimation");
        Artboard::advance_handle(
            &artboard,
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
        let position = target
            .with_downcast::<Node, _>(|node| (node.base.x(), node.base.y()))
            .expect("target Node");
        assert_eq!(position.0, 450.0);
        assert_eq!(position.1, 337.0);
        binding_ik_world_about_equal(
            &bone_a,
            [
                0.650279819965362548828125,
                -0.7596948146820068359375,
                0.7596948146820068359375,
                0.650279819965362548828125,
                26.015254974365234375,
                475.2149658203125,
            ],
        );
        binding_ik_world_about_equal(
            &bone_b,
            [
                0.8823678493499755859375,
                0.470560371875762939453125,
                -0.47056043148040771484375,
                0.882367908954620361328125,
                240.1275634765625,
                225.07647705078125,
            ],
        );
    }
}

#[test]
fn wave_b_ik_test_001_direct_port_expected_red() {
    binding_two_bone_ik(1);
}

#[test]
fn wave_b_ik_test_002_direct_port_expected_red() {
    binding_two_bone_ik(1000);
}

#[cfg(feature = "tools")]
fn binding_image_asset_checks(
    file: &RuntimeFileHandle,
    factory: &PersistentFactory<RecordingFactory>,
) {
    let artboard = file.with_file(File::artboard).expect("authored artboard");
    let walle = binding_source_find::<Component>(&artboard, "walle");
    assert!(walle.is_type_of(ImageBase::TYPE_KEY));
    let walle_asset = walle
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .expect("walle image asset");
    assert_eq!(
        walle_asset
            .with_downcast::<ImageAsset, _>(|asset| asset.decoded_byte_size)
            .expect("ImageAsset"),
        218873,
    );

    let eve_left = binding_source_find::<Component>(&artboard, "eve_left");
    assert!(eve_left.is_type_of(ImageBase::TYPE_KEY));
    let eve_left_asset = eve_left
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .expect("eve_left image asset");
    assert_eq!(
        eve_left_asset
            .with_downcast::<ImageAsset, _>(|asset| asset.decoded_byte_size)
            .expect("ImageAsset"),
        246825,
    );

    let eve_right = binding_source_find::<Component>(&artboard, "eve_right");
    assert!(eve_right.is_type_of(ImageBase::TYPE_KEY));
    let eve_right_asset = eve_right
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .expect("eve_right image asset");
    assert_ne!(eve_right_asset, walle_asset);
    assert_eq!(eve_right_asset, eve_left_asset);
    Artboard::update_components_handle(&artboard);
    let mut renderer = factory.borrow().make_renderer();
    Artboard::draw_handle(&artboard, &mut renderer);
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_image_asset_test_001_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = binding_file("walle.riv", &mut factory);
    binding_image_asset_checks(&file, &factory);
}

#[test]
#[cfg(feature = "tools")]
fn wave_b_image_asset_test_002_direct_port_expected_red() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let path = binding_path("assets/out_of_band/walle.riv");
    let bytes = std::fs::read(&path).expect("out-of-band walle file");
    let loader = FileAssetLoaderRef::new(Box::new(RelativeLocalAssetLoader::new(
        path.to_str().expect("fixture path").to_owned(),
    )));
    let mut result = ImportResult::Malformed;
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        Some(&mut result),
        Some(loader),
        None,
    )
    .expect("out-of-band file imports");
    assert_eq!(result, ImportResult::Success);
    binding_image_asset_checks(&file, &factory);
}
