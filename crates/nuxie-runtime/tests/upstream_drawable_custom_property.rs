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

#[cfg(any(feature = "testing", feature = "tools"))]
struct ArtboardBuilder {
    arena: CoreArena,
    artboard: CoreHandle,
    next_id: u32,
}
#[cfg(any(feature = "testing", feature = "tools"))]
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
#[cfg(any(feature = "testing", feature = "tools"))]
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
#[cfg(any(feature = "testing", feature = "tools"))]
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

#[test]
#[cfg(any(feature = "testing", feature = "tools"))]
fn draw_visitor_draw_order_update_changes_the_next_drawable() {
    use nuxie_runtime::source::{
        draw_rules::DrawRules,
        draw_target::DrawTarget,
        generated::{draw_rules_base::DrawRulesBase, draw_target_base::DrawTargetBase},
    };
    let mut builder = ArtboardBuilder::new();
    let mut shapes = Vec::new();
    for _ in 0..3 {
        let (id, shape) = builder.add_shape();
        let property = builder.property(3, 0.5);
        builder.add(&property, id);
        shapes.push((id, shape));
    }
    let rules = builder.arena.insert(DrawRules::default());
    let rules_id = builder.add(&rules, shapes[1].0);
    let target = builder.arena.insert(DrawTarget::default());
    uint(
        &target,
        DrawTargetBase::DRAWABLE_ID_PROPERTY_KEY,
        shapes[0].0,
    );
    uint(&target, DrawTargetBase::PLACEMENT_VALUE_PROPERTY_KEY, 0);
    let target_id = builder.add(&target, rules_id);
    builder.initialize();

    let visits = Rc::new(RefCell::new(Vec::new()));
    let recorded = visits.clone();
    let root = builder.artboard.clone();
    let first = shapes[2].1.clone();
    let changed = Cell::new(false);
    let visitor: RuntimeDrawVisitor = Rc::new(move |drawable, _| {
        recorded.borrow_mut().push(drawable.clone());
        if drawable == &first && !changed.replace(true) {
            // Move the middle shape before the bottom shape while the
            // top shape's visitor is running. C++'s for-loop increment
            // follows the freshly relinked prev pointer after this callback.
            uint(
                &rules,
                DrawRulesBase::DRAW_TARGET_ID_PROPERTY_KEY,
                target_id,
            );
            Artboard::update_components_handle(&root);
        }
    });
    let mut renderer = ModulationRecorder::default();
    Artboard::draw_internal_with_visitor_handle(
        &builder.artboard,
        &mut renderer,
        Some(visitor),
        None,
    );
    assert_eq!(
        *visits.borrow(),
        vec![
            shapes[2].1.clone(),
            shapes[0].1.clone(),
            shapes[1].1.clone(),
        ]
    );
    assert_eq!(renderer.depth, 0);
}

#[test]
#[cfg(any(feature = "testing", feature = "tools"))]
fn draw_visitor_can_release_the_last_external_runtime_root() {
    use nuxie_runtime::source::{
        focus_data::FocusData,
        input::{
            focus_manager::FocusManager,
            focusable::{Focusable, Key, KeyModifiers},
        },
    };

    struct ObserveBlur {
        root: CoreHandle,
        events: Rc<RefCell<Vec<&'static str>>>,
    }
    impl Focusable for ObserveBlur {
        fn key_input(&mut self, _: Key, _: KeyModifiers, _: bool, _: bool) -> bool {
            false
        }
        fn text_input(&mut self, _: &str) -> bool {
            false
        }
        fn focused(&mut self) {}
        fn blurred(&mut self) {
            assert_eq!(self.root.with(|_| true), Some(true));
            assert_eq!(self.root.with_mut(|_| true), Some(true));
            self.events.borrow_mut().push("blur");
        }
    }

    let mut builder = ArtboardBuilder::new();
    for label in ["second", "first"] {
        let (id, shape) = builder.add_shape();
        name(&shape, label);
        let property = builder.property(3, 0.5);
        builder.add(&property, id);
        if label == "first" {
            let focus_data = builder.arena.insert(FocusData::default());
            builder.add(&focus_data, id);
        }
    }
    builder.initialize();
    let instance = Artboard::instance_from_handle(&builder.artboard).unwrap();
    let root = instance.core_handle();
    advance(&root);
    let (first, second, focus_data) = instance.with_artboard(|artboard| {
        (
            artboard.find_handle::<Shape>("first").unwrap(),
            artboard.find_handle::<Shape>("second").unwrap(),
            artboard
                .objects()
                .iter()
                .flatten()
                .find(|object| object.is_type_of(FocusData::TYPE_KEY))
                .cloned()
                .unwrap(),
        )
    });
    let manager = instance.ensure_focus_manager();
    let node = focus_data
        .with_downcast_mut::<FocusData, _>(FocusData::focus_node)
        .unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    node.borrow_mut()
        .set_focusable(Some(Rc::new(RefCell::new(ObserveBlur {
            root: root.clone(),
            events: events.clone(),
        }))));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, node.clone(), None);
        manager.set_focus(node.clone());
    });
    assert!(
        manager
            .with_focus_manager(FocusManager::primary_focus)
            .is_some()
    );

    let external = Rc::new(RefCell::new(Some(instance)));
    let retained = external.clone();
    let recorded = events.clone();
    let live_root = root.clone();
    let visitor: RuntimeDrawVisitor = Rc::new(move |drawable, _| {
        if drawable == &first {
            recorded.borrow_mut().push("first");
            // Only the active draw range now retains the runtime instance.
            drop(retained.borrow_mut().take().expect("last external root"));
            assert!(live_root.is_alive());
        } else {
            assert_eq!(drawable, &second);
            assert!(retained.borrow().is_none());
            recorded.borrow_mut().push("second");
        }
    });
    let mut renderer = ModulationRecorder::default();
    Artboard::draw_internal_with_visitor_handle(&root, &mut renderer, Some(visitor), None);

    assert_eq!(&*events.borrow(), &["first", "second", "blur"]);
    assert!(external.borrow().is_none());
    assert!(!root.is_alive());
    assert!(root.with(|_| ()).is_none());
    assert!(
        manager
            .with_focus_manager(FocusManager::primary_focus)
            .is_none()
    );
    assert!(node.borrow().manager().is_none());
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
#[cfg(any(feature = "testing", feature = "tools"))]
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
