//! Direct native-owner ports of the pinned image data-binding cases.

use std::{path::PathBuf, rc::Rc};

use nuxie::runtime::{
    core::CoreType,
    generated::{
        core_registry::CoreRegistry, shapes::image_base::ImageBase,
        viewmodel::viewmodel_instance_asset_base::ViewModelInstanceAssetBase,
    },
    layout::Fit,
    layout_component::LayoutComponent,
    math::vec2d::Vec2D,
    nested_artboard::NestedArtboard,
    shapes::image::Image,
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_asset_image::ViewModelInstanceAssetImage,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};
use nuxie::{
    CoreHandle, File, ImportResult, PersistentFactory, RuntimeArtboardInstanceHandle,
    RuntimeFactoryHandle, RuntimeFileHandle, RuntimeStateMachineInstanceHandle,
};
use nuxie_render_api::{Factory, RecordingFactory, SerializingFactory};
use silver_corpus::{compare_sriv, parse_sriv};

fn pinned_path(relative: &str) -> PathBuf {
    PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests")
    .join(relative)
}

fn pinned(relative: &str) -> Vec<u8> {
    let path = pinned_path(relative);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn import<F: Factory + 'static>(
    asset: &str,
    factory: &mut PersistentFactory<F>,
) -> RuntimeFileHandle {
    let retained = RuntimeFactoryHandle::from_factory(factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(
        &pinned(&format!("assets/{asset}")),
        retained,
        Some(&mut result),
        None,
        None,
    )
    .unwrap_or_else(|| panic!("{asset} imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    file
}

fn property(instance: &CoreHandle, name: &str) -> CoreHandle {
    instance
        .with_downcast::<ViewModelInstance, _>(|instance| instance.property_value_named(name))
        .flatten()
        .unwrap_or_else(|| panic!("view model property {name}"))
}

fn default_instance(
    file: &RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
) -> CoreHandle {
    file.with_file_mut(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    })
    .expect("default view model instance")
}

fn find<T: CoreType>(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    artboard
        .with_artboard(|artboard| artboard.find_handle::<T>(name))
        .unwrap_or_else(|| panic!("authored object {name}"))
}

fn child(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> RuntimeArtboardInstanceHandle {
    find::<NestedArtboard>(artboard, name)
        .with(|nested| nested.nested_artboard_instance_handle())
        .flatten()
        .expect("mounted nested artboard")
}

fn set_asset_index(property: &CoreHandle, index: u32) {
    assert!(CoreRegistry::set_uint_handle(
        property,
        ViewModelInstanceAssetBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
        index,
    ));
}

fn asset_index(property: &CoreHandle) -> usize {
    property
        .with_downcast::<ViewModelInstanceAssetImage, _>(|property| {
            property.base.property_value() as usize
        })
        .expect("image property")
}

fn set_image(property: &CoreHandle, image: Option<Rc<dyn nuxie_render_api::RenderImage>>) {
    property
        .with_downcast_mut::<ViewModelInstanceAssetImage, _>(|property| property.set_value(image))
        .expect("image property");
}

fn click(machine: &RuntimeStateMachineInstanceHandle, point: Vec2D) {
    machine.with_instance_mut(|machine| machine.pointer_down(point, 0));
    machine.with_instance_mut(|machine| machine.pointer_up(point, 0));
}

struct SilverFixture {
    machine: RuntimeStateMachineInstanceHandle,
    artboard: RuntimeArtboardInstanceHandle,
    file: RuntimeFileHandle,
    silver: PersistentFactory<SerializingFactory>,
}

impl SilverFixture {
    fn new(asset: &str, artboard_name: Option<&str>) -> Self {
        let mut silver = PersistentFactory::new(SerializingFactory::new());
        let file = import(asset, &mut silver);
        let artboard = file
            .with_file(|file| match artboard_name {
                Some(name) => file.artboard_named(name),
                None => file.artboard_default(),
            })
            .expect("authored artboard");
        let (width, height) = artboard.with_artboard(|a| (a.width(), a.height()));
        silver.borrow_mut().frame_size(width as u32, height as u32);
        let machine = artboard.state_machine_at(0).expect("state machine 0");
        Self {
            machine,
            artboard,
            file,
            silver,
        }
    }

    fn authored_instance(&self) -> CoreHandle {
        let id = self.artboard.with_artboard(|a| a.base.view_model_id());
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
        let expected = parse_sriv(&pinned(&format!("silvers/{name}.sriv"))).expect("pinned SRIV");
        let actual = parse_sriv(&self.silver.borrow().bytes()).expect("native SRIV");
        compare_sriv(&expected, &actual)
            .unwrap_or_else(|difference| panic!("{name}: {difference}"));
    }
}

#[test]
fn data_binding_images_from_file_assets() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = import("data_binding_images_test.riv", &mut factory);
    let artboard = file
        .with_file(|file| file.artboard_named("main"))
        .expect("main");
    let vmi = default_instance(&file, &artboard);
    artboard.bind_view_model_instance(Some(vmi.clone()));
    artboard.advance_default(0.0);
    let main = property(&vmi, "main_im");
    assert!(main.is_type_of(ViewModelInstanceAssetImage::TYPE_KEY));
    let sub = property(&vmi, "sub_1");
    assert!(sub.is_type_of(ViewModelInstanceViewModel::TYPE_KEY));
    let sub_vmi = sub
        .with_downcast::<ViewModelInstanceViewModel, _>(|p| p.reference_view_model_instance())
        .flatten()
        .expect("sub view model");
    let sub_image_property = property(&sub_vmi, "sub_1_im");
    let assets = file.with_file(|file| file.assets().to_vec());
    let root_image = find::<Image>(&artboard, "root_img");
    let nested_image = find::<Image>(&child(&artboard, "sub_1"), "sub_1_img");
    let original_root = root_image
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .expect("root asset");
    let original_sub = nested_image
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .expect("sub asset");
    assert_eq!(original_root, assets[asset_index(&main)]);
    assert_eq!(original_sub, assets[asset_index(&sub_image_property)]);
    set_asset_index(&main, 2);
    set_asset_index(&sub_image_property, 6);
    artboard.advance_default(0.0);
    assert_ne!(original_root, assets[asset_index(&main)]);
    assert_ne!(original_sub, assets[asset_index(&sub_image_property)]);
    assert_eq!(
        root_image
            .with_downcast::<Image, _>(Image::image_asset)
            .flatten(),
        Some(assets[2].clone())
    );
    assert_eq!(
        nested_image
            .with_downcast::<Image, _>(Image::image_asset)
            .flatten(),
        Some(assets[6].clone())
    );
}

#[test]
fn embedded_images_can_be_reset_by_passing_live_image_null() {
    let fixture = SilverFixture::new("viewmodel_image_reset.riv", None);
    let vmi = fixture.authored_instance();
    fixture
        .machine
        .with_instance_mut(|m| m.bind_view_model_instance(vmi.clone()));
    fixture.advance(0.1);
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let image_property = property(&vmi, "img");
    set_image(&image_property, None);
    image_property.with_downcast::<ViewModelInstanceAssetImage, _>(|property| {
        assert!(property.asset().render_image().is_none());
        assert_eq!(property.base.property_value(), u32::MAX);
    });
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("viewmodel_image_reset");
}

fn images(artboard: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
    artboard.with_artboard(|artboard| artboard.objects_typed::<Image>().iter().collect())
}

fn image_fit(image: &CoreHandle) -> u32 {
    image
        .with_downcast::<Image, _>(|i| i.base.fit())
        .expect("Image")
}
fn image_alignment(image: &CoreHandle) -> (f32, f32) {
    image
        .with_downcast::<Image, _>(|i| (i.base.alignment_x(), i.base.alignment_y()))
        .expect("Image")
}
fn set_fit(image: &CoreHandle, fit: u32, x: f32, y: f32) {
    CoreRegistry::set_uint_handle(image, ImageBase::FIT_PROPERTY_KEY.into(), fit);
    CoreRegistry::set_double_handle(image, ImageBase::ALIGNMENT_X_PROPERTY_KEY.into(), x);
    CoreRegistry::set_double_handle(image, ImageBase::ALIGNMENT_Y_PROPERTY_KEY.into(), y);
}

#[test]
fn image_fit_alignment_preserves_generated_owners_and_all_three_twenty_frame_phases() {
    let fixture = SilverFixture::new("image_fit_alignment.riv", Some("Main"));
    let vmi = fixture.authored_instance();
    fixture
        .machine
        .with_instance_mut(|m| m.bind_view_model_instance(vmi.clone()));
    fixture.advance(0.1);
    let image = images(&fixture.artboard).into_iter().next().expect("Image");
    let fit = image_fit(&image);
    let (x, y) = image_alignment(&image);
    let test_fit = if fit == Fit::Contain as u32 {
        Fit::Cover as u32
    } else {
        Fit::Contain as u32
    };
    let test_x = if x == -1.0 { 1.0 } else { -1.0 };
    let test_y = if y == -1.0 { 1.0 } else { -1.0 };
    set_fit(&image, test_fit, test_x, test_y);
    assert_eq!(
        (image_fit(&image), image_alignment(&image)),
        (test_fit, (test_x, test_y))
    );
    set_fit(&image, fit, x, y);
    assert_eq!((image_fit(&image), image_alignment(&image)), (fit, (x, y)));
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    let assets = fixture.file.with_file(|file| file.assets().to_vec());
    let asset_index = |name: &str| {
        assets
            .iter()
            .position(|a| {
                a.with(|a| {
                    a.as_file_asset()
                        .is_some_and(|a| a.file_asset_base().name() == name)
                })
                .unwrap_or(false)
            })
            .expect("asset") as u32
    };
    let property = property(&vmi, "imageProperty");
    for next in [asset_index("image2"), asset_index("image3")] {
        for _ in 0..20 {
            fixture.silver.borrow_mut().add_frame();
            fixture.advance(0.016);
            fixture.artboard.draw(&mut renderer);
        }
        set_asset_index(&property, next);
        fixture.advance(0.0);
        let no_scale = images(&fixture.artboard)
            .into_iter()
            .find(|i| image_fit(i) == Fit::None as u32)
            .expect("Fit::none Image");
        let transform = no_scale
            .with_downcast::<Image, _>(|i| *i.base.transform())
            .expect("transform");
        assert!(transform[4] < 0.0);
        assert!(transform[5] < 0.0);
    }
    for _ in 0..20 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.artboard.draw(&mut renderer);
    }
    fixture.matches("image_fit_alignment");
}

#[test]
fn dynamic_image_binding_with_listener_action() {
    let fixture = SilverFixture::new("image_binding_with_listener.riv", Some("main"));
    let vmi = fixture.authored_instance();
    fixture
        .machine
        .with_instance_mut(|m| m.bind_view_model_instance(vmi.clone()));
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    click(&fixture.machine, Vec2D::new(650.0, 650.0));
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    let bytes = pinned("assets/open_source.jpg");
    assert_eq!(bytes.len(), 8880);
    let image = Rc::from(
        fixture
            .silver
            .borrow_mut()
            .decode_image(&bytes)
            .expect("decode"),
    );
    let property = property(&vmi, "image1");
    set_image(&property, Some(image));
    assert!(
        property
            .with_downcast::<ViewModelInstanceAssetImage, _>(|property| {
                property.asset().render_image().is_some()
            })
            .unwrap_or(false)
    );
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    click(&fixture.machine, Vec2D::new(650.0, 650.0));
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    set_image(&property, None);
    assert!(
        property
            .with_downcast::<ViewModelInstanceAssetImage, _>(|property| {
                property.asset().render_image().is_none()
            })
            .unwrap_or(false)
    );
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    click(&fixture.machine, Vec2D::new(650.0, 650.0));
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("image_binding_with_listener");
}

fn approx(actual: f32, expected: f32) -> bool {
    let actual = f64::from(actual);
    let expected = f64::from(expected);
    (actual - expected).abs() <= f64::from(f32::EPSILON) * 100.0 * expected.abs()
}

#[test]
fn catch_approx_widens_float_operands_before_comparing() {
    let expected = f32::from_bits(0x0072_abfc);
    assert!(!approx(f32::from_bits(expected.to_bits() + 90), expected));
}

#[test]
fn layout_image_composes_user_scale_on_top_of_fit_for_7_2_files() {
    let legacy = pinned("assets/image_fit_alignment.riv");
    assert_eq!(&legacy[..5], b"RIVE\x07");
    assert!(legacy[5] < 2);
    let mut modern = legacy.clone();
    modern[5] = 2;
    struct Loaded {
        _file: RuntimeFileHandle,
        _artboard: RuntimeArtboardInstanceHandle,
        machine: RuntimeStateMachineInstanceHandle,
        images: Vec<CoreHandle>,
    }
    let load = |bytes: &[u8], factory: &mut PersistentFactory<SerializingFactory>| {
        let retained = RuntimeFactoryHandle::from_factory(factory).expect("factory");
        let file = File::import(bytes, retained, None, None, None).expect("File");
        let artboard = file
            .with_file(|file| file.artboard_named("Main"))
            .expect("Main");
        let machine = artboard.state_machine_at(0).expect("machine");
        let id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
        let vmi = file
            .with_file_mut(|file| {
                if id == u32::MAX {
                    file.create_view_model_instance_for_artboard(artboard.core_handle())
                } else {
                    file.create_view_model_instance_at(id as usize, 0)
                }
            })
            .expect("vmi");
        machine.with_instance_mut(|m| m.bind_view_model_instance(vmi));
        machine.advance_and_apply(0.1);
        let images = images(&artboard)
            .into_iter()
            .filter(|image| {
                image
                    .with_downcast::<Image, _>(|i| i.base.parent_handle())
                    .flatten()
                    .is_some_and(|p| p.is_type_of(LayoutComponent::TYPE_KEY))
            })
            .collect();
        Loaded {
            _file: file,
            _artboard: artboard,
            machine,
            images,
        }
    };
    let mut lf = PersistentFactory::new(SerializingFactory::new());
    let mut mf = PersistentFactory::new(SerializingFactory::new());
    let legacy = load(&legacy, &mut lf);
    let modern = load(&modern, &mut mf);
    assert!(!legacy.images.is_empty());
    assert_eq!(legacy.images.len(), modern.images.len());
    let scale = |image: &CoreHandle| {
        image
            .with_downcast::<Image, _>(|i| {
                Vec2D::new(i.base.world_transform()[0], i.base.world_transform()[1]).length()
            })
            .expect("Image")
    };
    let mut pick = None;
    for _ in 0..120 {
        pick = legacy.images.iter().position(|i| scale(i) > 1.0);
        if pick.is_some() {
            break;
        }
        legacy.machine.advance_and_apply(0.016);
        modern.machine.advance_and_apply(0.016);
    }
    let pick = pick.expect("open image");
    let li = &legacy.images[pick];
    let mi = &modern.images[pick];
    let user = mi.with_downcast::<Image, _>(|i| i.base.scale_x()).unwrap();
    let old_user = li.with_downcast::<Image, _>(|i| i.base.scale_x()).unwrap();
    assert!(!approx(user, 1.0));
    assert!(!approx(old_user, user));
    assert!(scale(li) > 0.0);
    assert!(approx(scale(mi), scale(li) * user));
}

#[test]
fn stateful_component_image_bind() {
    let fixture = SilverFixture::new("stateful_component_image_test.riv", None);
    let vmi = fixture.authored_instance();
    fixture
        .machine
        .with_instance_mut(|m| m.bind_view_model_instance(vmi.clone()));
    let mut renderer = fixture.silver.borrow().make_renderer();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    let bytes = pinned("assets/open_source.jpg");
    assert_eq!(bytes.len(), 8880);
    let image = Rc::from(
        fixture
            .silver
            .borrow_mut()
            .decode_image(&bytes)
            .expect("decode"),
    );
    let image_property = property(&vmi, "img");
    set_image(&image_property, Some(image));
    assert!(
        image_property
            .with_downcast::<ViewModelInstanceAssetImage, _>(|property| {
                property.asset().render_image().is_some()
            })
            .unwrap_or(false)
    );
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    fixture.artboard.draw(&mut renderer);
    fixture.matches("stateful_component_image_test");
}
