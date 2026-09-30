//! Complete drawable_custom_property_test.cpp at upstream df0cc777.
use nuxie_render_api::*;
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    artboard::{Artboard, RuntimeDrawVisitor},
    assets::manifest_asset::ManifestAsset,
    core::{CoreArena, CoreHandle},
    custom_property::{CustomProperty, CustomPropertyKind},
    custom_property_number::CustomPropertyNumber,
    drawable::Drawable,
    file::RuntimeFileHandle,
    generated::{
        component_base::ComponentBase, core_registry::CoreRegistry,
        custom_property_base::CustomPropertyBase,
        custom_property_number_base::CustomPropertyNumberBase,
        shapes::parametric_path_base::ParametricPathBase,
    },
    shapes::{rectangle::Rectangle, shape::Shape},
    status_code::StatusCode,
};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

fn uint(owner: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(owner, key.into(), value));
}
fn number(owner: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(owner, key.into(), value));
}
fn name(owner: &CoreHandle, value: &str) {
    assert!(CoreRegistry::set_string_handle(
        owner,
        ComponentBase::NAME_PROPERTY_KEY.into(),
        value.to_owned()
    ));
}
fn advance(root: &CoreHandle) {
    Artboard::advance_handle(
        root,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}

struct ArtboardBuilder {
    arena: CoreArena,
    artboard: CoreHandle,
    next_id: u32,
}
impl ArtboardBuilder {
    fn new() -> Self {
        let arena = CoreArena::default();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        let artboard = arena.insert(Artboard::with_factory(factory));
        artboard
            .with_downcast_mut::<Artboard, _>(|a| {
                a.set_core_arena(arena.clone());
                a.add_object(Some(artboard.clone()));
            })
            .unwrap();
        Self {
            arena,
            artboard,
            next_id: 1,
        }
    }
    fn add(&mut self, component: &CoreHandle, parent: u32) -> u32 {
        uint(component, ComponentBase::PARENT_ID_PROPERTY_KEY, parent);
        self.artboard
            .with_downcast_mut::<Artboard, _>(|a| a.add_object(Some(component.clone())))
            .unwrap();
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    fn add_shape(&mut self) -> (u32, CoreHandle) {
        let shape = self.arena.insert(Shape::default());
        let id = self.add(&shape, 0);
        let rectangle = self.arena.insert(Rectangle::default());
        number(&rectangle, ParametricPathBase::WIDTH_PROPERTY_KEY, 10.0);
        number(&rectangle, ParametricPathBase::HEIGHT_PROPERTY_KEY, 10.0);
        self.add(&rectangle, id);
        (id, shape)
    }
    fn property(&self, key: u32, value: f32) -> CoreHandle {
        let property = self.arena.insert(CustomPropertyNumber::default());
        uint(&property, CustomPropertyBase::NAME_ID_PROPERTY_KEY, key);
        number(
            &property,
            CustomPropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY,
            value,
        );
        property
    }
    fn initialize(&self) {
        assert_eq!(Artboard::initialize_handle(&self.artboard), StatusCode::Ok);
        advance(&self.artboard);
    }
}

#[derive(Default)]
struct ModulationRecorder {
    colors: Vec<u32>,
    depth: i32,
}
impl Renderer for ModulationRecorder {
    fn save(&mut self) {
        self.depth += 1;
    }
    fn restore(&mut self) {
        self.depth -= 1;
    }
    fn transform(&mut self, _: Mat2D) {}
    fn draw_path(&mut self, _: &dyn RenderPath, _: &dyn RenderPaint) {}
    fn clip_path(&mut self, _: &dyn RenderPath) {}
    fn draw_image(&mut self, _: Option<&dyn RenderImage>, _: ImageSampler, _: BlendMode, _: f32) {}
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        _: BlendMode,
        _: f32,
    ) {
    }
    fn modulate_opacity(&mut self, _: f32) {}
    fn modulate_color(&mut self, color: u32, replace: bool) {
        if replace {
            self.colors.push(color);
        }
    }
}

#[test]
fn draw_visitor_sees_only_drawables_tagged_with_custom_properties() {
    let mut builder = ArtboardBuilder::new();
    builder.add_shape();
    let (tagged_id, tagged) = builder.add_shape();
    let emissive = builder.property(3, 0.5);
    builder.add(&emissive, tagged_id);
    let input = builder.arena.insert(CustomPropertyNumber::default());
    name(&input, "input");
    builder.add(&input, tagged_id);
    let (input_holder_id, input_holder) = builder.add_shape();
    let named = builder.arena.insert(CustomPropertyNumber::default());
    name(&named, "speed");
    builder.add(&named, input_holder_id);
    builder.initialize();
    assert!(
        tagged
            .with(|o| o.as_drawable().unwrap().has_custom_properties())
            .unwrap()
    );
    assert_eq!(
        Drawable::custom_property_handle(&tagged, 3),
        Some(emissive.clone())
    );
    assert!(Drawable::custom_property_handle(&tagged, 4).is_none());
    assert_eq!(
        input
            .with(|o| o.as_component().unwrap().parent_handle())
            .flatten(),
        Some(tagged.clone())
    );
    assert_eq!(
        CoreRegistry::get_uint_handle(&input, CustomPropertyBase::NAME_ID_PROPERTY_KEY.into()),
        Some(CustomProperty::NO_NAME_ID)
    );
    assert!(Drawable::custom_property_handle(&tagged, CustomProperty::NO_NAME_ID).is_none());
    assert_eq!(
        CustomProperty::kind_handle(&emissive),
        CustomPropertyKind::Number
    );
    assert!(
        !input_holder
            .with(|o| o.as_drawable().unwrap().has_custom_properties())
            .unwrap()
    );
    let mut renderer = ModulationRecorder::default();
    let visits = Rc::new(RefCell::new(Vec::new()));
    let recorded = visits.clone();
    let visitor: RuntimeDrawVisitor = Rc::new(move |drawable, renderer| {
        recorded.borrow_mut().push(drawable.clone());
        Drawable::draw_handle(drawable, renderer);
    });
    Artboard::draw_internal_with_visitor_handle(
        &builder.artboard,
        &mut renderer,
        Some(visitor),
        None,
    );
    assert_eq!(visits.borrow().len(), 1);
    assert_eq!(visits.borrow()[0], tagged);
    visits.borrow_mut().clear();
    Artboard::draw_internal_handle(&builder.artboard, &mut renderer);
    assert!(visits.borrow().is_empty());
}

#[test]
fn draw_modulated_sets_the_color_from_each_tagged_drawable() {
    let mut builder = ArtboardBuilder::new();
    builder.add_shape();
    for (key, value) in [(3, 0.4), (3, 3.0), (9, 0.0)] {
        let property = builder.property(key, value);
        let (id, _) = builder.add_shape();
        builder.add(&property, id);
    }
    builder.initialize();
    let mut renderer = ModulationRecorder::default();
    Artboard::draw_modulated_handle(&builder.artboard, &mut renderer, 3, None);
    renderer.colors.sort_unstable();
    assert_eq!(renderer.colors, vec![0xff666666, 0xffffffff]);
    assert_eq!(renderer.depth, 0);
}

fn read_file(name: &str) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(|root| std::path::PathBuf::from(root).join("tests/unit_tests/assets"))
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sync")
        });
    let bytes = std::fs::read(root.join(name)).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    File::import(&bytes, factory, None, None, None).unwrap()
}
fn hosting_visitor(
    host: CoreHandle,
    hosted: CoreHandle,
) -> (
    RuntimeDrawVisitor,
    Rc<Cell<bool>>,
    Rc<RefCell<Vec<CoreHandle>>>,
) {
    let did_host = Rc::new(Cell::new(false));
    let visited = Rc::new(RefCell::new(Vec::new()));
    let did = did_host.clone();
    let visits = visited.clone();
    let visitor: RuntimeDrawVisitor = Rc::new(move |drawable, renderer| {
        visits.borrow_mut().push(drawable.clone());
        if !did.replace(true) {
            Artboard::draw_hosted_handle(&host, &hosted, renderer);
        }
        Drawable::draw_handle(drawable, renderer);
    });
    (visitor, did_host, visited)
}

#[test]
fn property_key_does_not_reach_into_artboard_from_another_file() {
    let file = read_file("drawable_custom_properties.riv");
    let other_file = read_file("drawable_custom_properties_other.riv");
    let manifest = file.with_file(File::manifest).unwrap();
    let other_manifest = other_file.with_file(File::manifest).unwrap();
    let emissive = manifest
        .with_downcast::<ManifestAsset, _>(|m| m.name_id(b"emissive"))
        .unwrap();
    assert!(emissive >= 0);
    assert_eq!(
        other_manifest
            .with_downcast::<ManifestAsset, _>(|m| m.name_id(b"emissive"))
            .unwrap(),
        -1
    );
    assert_eq!(
        other_manifest
            .with_downcast::<ManifestAsset, _>(|m| m.resolve_name(emissive).to_owned())
            .unwrap(),
        "unrelated"
    );
    let host = file.with_file(File::artboard_default).unwrap();
    let hosted = other_file
        .with_file(File::bindable_artboard_default)
        .unwrap();
    let hosted_artboard = hosted.artboard_handle();
    advance(&host.core_handle());
    advance(&hosted_artboard.core_handle());
    let unrelated = hosted_artboard
        .with_artboard(|a| a.find_handle::<Shape>("unrelated"))
        .unwrap();
    assert!(Drawable::custom_property_handle(&unrelated, emissive as u32).is_some());
    let mut renderer = ModulationRecorder::default();
    let (visitor, did_host, visited) =
        hosting_visitor(host.core_handle(), hosted_artboard.core_handle());
    Artboard::draw_internal_with_visitor_handle(
        &host.core_handle(),
        &mut renderer,
        Some(visitor),
        None,
    );
    assert!(did_host.get());
    assert!(!visited.borrow().contains(&unrelated));
    let source = file.with_file(File::artboard).unwrap();
    let scripted = Artboard::instance_from_handle(&source).unwrap();
    advance(&scripted.core_handle());
    let same_file = file.with_file(File::bindable_artboard_default).unwrap();
    let same_artboard = same_file.artboard_handle();
    advance(&same_artboard.core_handle());
    let dim = same_artboard
        .with_artboard(|a| a.find_handle::<Shape>("dim"))
        .unwrap();
    for bound in [same_artboard.core_handle(), hosted_artboard.core_handle()] {
        let (visitor, did_host, visited) = hosting_visitor(scripted.core_handle(), bound.clone());
        Artboard::draw_internal_with_visitor_handle(
            &scripted.core_handle(),
            &mut renderer,
            Some(visitor),
            Some(file.clone()),
        );
        assert!(did_host.get());
        assert_eq!(
            visited.borrow().contains(&dim),
            bound == same_artboard.core_handle()
        );
        assert!(!visited.borrow().contains(&unrelated));
    }
}

/// Supplemental Rust ownership regression: the hosted visitor can access the
/// node whose draw is still on the stack, just as the C++ visitor can.
#[test]
fn hosted_visitor_can_reborrow_its_nested_host() {
    use nuxie_runtime::source::{generated::node_base::NodeBase, nested_artboard::NestedArtboard};
    let file = read_file("drawable_custom_properties.riv");
    let source = file.with_file(File::artboard).unwrap();
    let instance = Artboard::instance_from_handle(&source).unwrap();
    advance(&instance.core_handle());
    let mut builder = ArtboardBuilder::new();
    let nested = builder.arena.insert(NestedArtboard::new());
    builder.add(&nested, 0);
    builder.initialize();
    nested
        .with_downcast_mut::<NestedArtboard, _>(|nested| {
            nested.referenced_artboard_instance(instance);
        })
        .unwrap();
    advance(&builder.artboard);
    let visits = Rc::new(Cell::new(0));
    let count = visits.clone();
    let host = nested.clone();
    let visitor: RuntimeDrawVisitor = Rc::new(move |drawable, renderer| {
        // Both shared and mutable access must work while NestedArtboard's
        // hosted draw is active; no callback is run under its RefMut.
        assert!(
            host.with(|object| object.as_nested_artboard().is_some())
                .unwrap()
        );
        let x = CoreRegistry::get_double_handle(&host, NodeBase::X_PROPERTY_KEY.into()).unwrap();
        assert!(CoreRegistry::set_double_handle(
            &host,
            NodeBase::X_PROPERTY_KEY.into(),
            x
        ));
        count.set(count.get() + 1);
        Drawable::draw_handle(drawable, renderer);
    });
    let mut renderer = ModulationRecorder::default();
    Artboard::draw_internal_with_visitor_handle(
        &builder.artboard,
        &mut renderer,
        Some(visitor),
        Some(file),
    );
    assert!(
        visits.get() > 0,
        "nested owner must dispatch a hosted visitor"
    );
    assert_eq!(renderer.depth, 0);
}
