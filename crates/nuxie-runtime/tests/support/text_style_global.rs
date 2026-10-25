#![allow(dead_code, unused_imports)]
pub use nuxie_render_api::{NullRenderer, PersistentFactory, RecordingFactory};
pub use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    advancing_component::{AdvancingComponent, AdvancingComponentHandle, QuietState},
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    core::{CoreArena, CoreHandle, CoreType},
    file::RuntimeFileHandle,
    generated::core_registry::CoreRegistry,
    math::vec2d::Vec2D,
    pointer_button::PointerButton,
    text::{
        cursor::CursorPosition,
        glyph_lookup::GlyphLookup,
        text::{Text, TextValueRunHandle},
        text_layout_view::TextLayoutView,
        text_style_global::TextStyleGlobal,
        text_style_paint::TextStylePaint,
        text_style_share_cache::TextStyleShareCache,
        text_value_run::TextValueRun,
    },
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_artboard::ViewModelInstanceArtboard,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_enum::ViewModelInstanceEnum,
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_string::ViewModelInstanceString,
    },
};
pub use nuxie_runtime::{File, RuntimeFactoryHandle};
pub fn load(name: &str) -> RuntimeFileHandle {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR"),
    );
    let bytes = std::fs::read(root.join("tests/unit_tests/assets").join(name)).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap()
}
pub fn source(f: &RuntimeFileHandle, name: &str) -> CoreHandle {
    f.with_file(|f| f.artboard_named_source(name)).unwrap()
}
pub fn instance(f: &RuntimeFileHandle, name: &str) -> RuntimeArtboardInstanceHandle {
    Artboard::instance_from_handle(&source(f, name)).unwrap()
}
pub fn vm(f: &RuntimeFileHandle, a: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    f.with_file(|f| f.create_default_view_model_instance_for_artboard(a.core_handle()))
        .unwrap()
}
pub fn property(v: &CoreHandle, name: &str) -> CoreHandle {
    v.with_downcast::<ViewModelInstance, _>(|v| v.property_value_named(name))
        .flatten()
        .unwrap()
}
pub fn number(v: &CoreHandle, name: &str, value: f32) {
    property(v, name)
        .with_downcast_mut::<ViewModelInstanceNumber, _>(|v| v.set_value(value))
        .unwrap();
}
pub fn string(v: &CoreHandle, name: &str, value: &str) {
    property(v, name)
        .with_downcast_mut::<ViewModelInstanceString, _>(|v| v.set_value(value))
        .unwrap();
}
pub fn style_index(v: &CoreHandle, index: i32) {
    v.with_downcast_mut::<ViewModelInstanceArtboard, _>(|v| v.set_property_value(index as u32))
        .unwrap();
}
pub fn asset(f: &RuntimeFileHandle, v: &CoreHandle, name: Option<&str>) {
    let asset = name.map(|n| f.with_file(|f| f.bindable_artboard_named(n)).unwrap());
    v.with_downcast_mut::<ViewModelInstanceArtboard, _>(|v| v.set_asset(asset))
        .unwrap();
}
pub fn index(f: &RuntimeFileHandle, name: &str) -> i32 {
    f.with_file(|f| {
        (0..f.artboard_count())
            .find(|i| {
                f.artboard_at_source(*i)
                    .unwrap()
                    .with_downcast::<Artboard, _>(|a| a.name() == name)
                    .unwrap()
            })
            .map_or(-1, |i| i as i32)
    })
}
pub fn first<T: CoreType>(a: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    first_source::<T>(&a.core_handle())
}
pub fn first_source<T: CoreType>(a: &CoreHandle) -> CoreHandle {
    a.with_downcast::<Artboard, _>(|a| {
        a.objects()
            .iter()
            .flatten()
            .find(|o| o.is_type_of(T::TYPE_KEY))
            .cloned()
    })
    .flatten()
    .unwrap()
}
pub fn named<T: CoreType>(a: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    a.with_artboard(|a| a.base.find_handle::<T>(name)).unwrap()
}
pub fn frame(a: &RuntimeArtboardInstanceHandle) {
    a.advance_default(0.0);
    a.draw(&mut NullRenderer::new());
}
pub fn frames(
    a: &RuntimeArtboardInstanceHandle,
    m: &RuntimeStateMachineInstanceHandle,
    count: usize,
) {
    for _ in 0..count {
        m.advance_and_apply(0.016);
        a.draw(&mut NullRenderer::new());
    }
}
pub fn run(text: &CoreHandle, index: usize) -> TextValueRunHandle {
    text.with_downcast::<Text, _>(|t| {
        assert!(t.runs().len() > index);
        t.runs()[index].clone()
    })
    .unwrap()
}
pub fn paint(run: &TextValueRunHandle) -> CoreHandle {
    with_run(run, |r| r.style()).unwrap()
}
pub fn font(run: &TextValueRunHandle) -> f32 {
    paint(run)
        .with_downcast::<TextStylePaint, _>(|p| p.base.font_size())
        .unwrap()
}
pub fn handle(text: &CoreHandle) -> CoreHandle {
    text.with(|t| {
        t.as_node()
            .unwrap()
            .children()
            .iter()
            .find(|c| c.is_type_of(TextStyleGlobal::TYPE_KEY))
            .cloned()
    })
    .flatten()
    .unwrap()
}
pub fn glyph(text: &CoreHandle, offset: u32) -> Vec2D {
    let (point, board) = text
        .with_downcast::<Text, _>(|t| {
            let mut lookup = GlyphLookup::default();
            lookup.compute(t.unichars(), t.shape());
            let view = TextLayoutView::new(
                t.shape(),
                &[],
                t.ordered_lines(),
                &lookup,
                t.unichars().len() as u32,
            );
            let x = |at| {
                let mut p = CursorPosition::unresolved(at);
                p.resolve_line(&view);
                let v = p.clamped(&view).visual_position(&view);
                assert!(v.found());
                Vec2D::new(v.x(), (v.top() + v.bottom()) / 2.0)
            };
            (
                t.shape_world_transform() * ((x(offset) + x(offset + 1)) * 0.5),
                t.artboard_handle().unwrap(),
            )
        })
        .unwrap();
    board
        .with_downcast_mut::<Artboard, _>(|a| a.root_transform(point))
        .unwrap()
}
pub fn color(run: &TextValueRunHandle) -> u32 {
    let p = paint(run);
    let shape = p
        .with_downcast::<TextStylePaint, _>(|p| {
            assert!(!p.paints.shape_paints().is_empty());
            p.paints.shape_paints()[0].clone()
        })
        .unwrap();
    let children = shape
        .with(|s| s.as_container_component().unwrap().children().to_vec())
        .unwrap();
    for c in children {
        if let Some(color) = c.with_downcast::<SolidColor, _>(|c| c.base.color_value()) {
            return color as u32;
        }
    }
    panic!("no solid color")
}
pub fn click(m: &RuntimeStateMachineInstanceHandle, p: Vec2D) {
    m.with_instance_mut(|m| {
        m.pointer_down(p, 0, PointerButton::Primary);
        m.pointer_up(p, 0, PointerButton::Primary);
    });
}
pub fn card(a: &RuntimeArtboardInstanceHandle) -> RuntimeArtboardInstanceHandle {
    let h = a
        .with_artboard(|a| a.base.nested_artboard_handle("Card"))
        .unwrap();
    h.with(|h| h.as_nested_artboard().unwrap().artboard_instance_default())
        .flatten()
        .unwrap()
}
pub use nuxie_runtime::source::text::text_style_reference::TextStyleReferenceHandle;
pub fn reference(r: &TextValueRunHandle) -> TextStyleReferenceHandle {
    with_run(r, |r| r.style_reference()).unwrap()
}
pub fn reference_is(r: &TextValueRunHandle, h: &CoreHandle) -> bool {
    reference(r).authored().as_ref() == Some(h)
}
pub fn machine(r: &TextValueRunHandle) -> RuntimeStateMachineInstanceHandle {
    reference(r).run_state_machine().unwrap()
}
pub fn machine_eq(a: &TextValueRunHandle, b: &TextValueRunHandle) -> bool {
    machine(a).downgrade().ptr_eq(&machine(b).downgrade())
}
pub fn input(r: &TextValueRunHandle, name: &str) -> bool {
    machine(r).with_instance(|m| m.get_bool(name).unwrap().value())
}
pub fn resolved(h: &CoreHandle) -> CoreHandle {
    h.with_downcast::<TextStyleGlobal, _>(|g| g.resolved_paint())
        .flatten()
        .unwrap()
}
pub fn shared(h: &CoreHandle) -> bool {
    h.with_downcast::<TextStyleGlobal, _>(|g| g.is_shared())
        .unwrap()
}
pub fn live(f: &RuntimeFileHandle) -> usize {
    f.with_file(|f| f.text_style_shares()).borrow().live_count()
}
pub fn update_global(h: &CoreHandle, p: Option<CoreHandle>) {
    TextStyleGlobal::update_artboard_occurrence(h, p);
}
pub fn run_machines(t: &CoreHandle) -> Vec<RuntimeStateMachineInstanceHandle> {
    t.with_downcast::<Text, _>(|t| {
        let mut result = vec![];
        t.run_state_machines(&mut result);
        result
    })
    .unwrap()
}
pub fn bound_scene(f: &RuntimeFileHandle) -> (RuntimeArtboardInstanceHandle, CoreHandle) {
    let a = instance(f, "Main");
    let v = vm(f, &a);
    a.bind_view_model_instance(Some(v.clone()));
    frame(&a);
    (a, v)
}
pub fn named_font(a: &RuntimeArtboardInstanceHandle, name: &str) -> f32 {
    font(&run(&named::<Text>(a, name), 0))
}
pub fn named_handle(a: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    handle(&named::<Text>(a, name))
}
pub fn global_instance(h: &CoreHandle) -> RuntimeArtboardInstanceHandle {
    h.with_downcast::<TextStyleGlobal, _>(|g| g.artboard_instance())
        .flatten()
        .unwrap()
}

pub use nuxie_runtime::source::{
    animation::state_machine::StateMachine, shapes::paint::solid_color::SolidColor,
};

fn with_run<R>(run: &TextValueRunHandle, f: impl FnOnce(&TextValueRun) -> R) -> R {
    match run {
        TextValueRunHandle::Core(handle) => handle
            .with(|owner| f(owner.as_text_value_run().unwrap()))
            .unwrap(),
        TextValueRunHandle::Runtime(run) => f(&run.borrow()),
    }
}
