//! Literal public text_style_global_test.cpp, upstream 0238f299.
#[path = "support/text_style_global.rs"]
mod support;
use support::*;
#[test]
fn global_is_advancing_and_artboard_referencer() {
    let arena = CoreArena::default();
    let global = arena.insert(TextStyleGlobal::default());
    assert!(global.with(|g| g.is_advancing_component()).unwrap());
    assert!(
        global
            .with(|g| g.artboard_referencer_referenced_artboard_id().is_some())
            .unwrap()
    );
    global
        .with_downcast_mut::<TextStyleGlobal, _>(|g| g.set_artboard_id(42))
        .unwrap();
    assert_eq!(
        global
            .with(|g| g.artboard_referencer_referenced_artboard_id().unwrap())
            .unwrap(),
        42
    );
}
#[test]
fn view_model_without_model_clones() {
    let arena = CoreArena::default();
    let v = arena.insert(ViewModelInstance::default());
    assert!(
        v.with_downcast::<ViewModelInstance, _>(|v| v.get_view_model())
            .flatten()
            .is_none()
    );
    let cloned = ViewModelInstance::clone_instance(&v).unwrap();
    assert!(
        cloned
            .with_downcast::<ViewModelInstance, _>(|v| v.get_view_model())
            .flatten()
            .is_none()
    );
}
#[test]
fn resolved_paint_reads_referenced_artboard() {
    let arena = CoreArena::default();
    let board = arena.insert(Artboard::default());
    let paint = arena.insert(TextStylePaint::default());
    board
        .with_downcast_mut::<Artboard, _>(|a| a.add_object(Some(paint.clone())))
        .unwrap();
    let mut global = TextStyleGlobal::default();
    assert!(global.resolved_paint().is_none());
    global.set_referenced_artboard(Some(board));
    assert_eq!(global.resolved_paint(), Some(paint));
}
struct BindingScene {
    artboard: RuntimeArtboardInstanceHandle,
    values: CoreHandle,
    style: CoreHandle,
    run: TextValueRunHandle,
    handle: CoreHandle,
}
impl BindingScene {
    fn new(f: &RuntimeFileHandle) -> Self {
        let artboard = instance(f, "Main");
        let values = vm(f, &artboard);
        artboard.bind_view_model_instance(Some(values.clone()));
        let style = property(&values, "style");
        let run = TextValueRunHandle::Core(first::<TextValueRun>(&artboard));
        let handle = first::<TextStyleGlobal>(&artboard);
        Self {
            artboard,
            values,
            style,
            run,
            handle,
        }
    }
    fn font(&self) -> f32 {
        frame(&self.artboard);
        font(&self.run)
    }
}
#[test]
fn style_property_swaps_global_handle() {
    let f = load("text_style_global_binding.riv");
    let s = BindingScene::new(&f);
    assert!(s.handle.is_alive());
    assert_eq!(
        s.style.core_type(),
        Some(ViewModelInstanceArtboard::TYPE_KEY)
    );
    assert_eq!(
        property(&s.values, "n")
            .with_downcast::<ViewModelInstanceNumber, _>(|v| v.base.property_value())
            .unwrap(),
        7.0
    );
    assert_eq!(s.font(), 48.0);
    asset(&f, &s.style, Some("Body"));
    assert_eq!(s.font(), 30.0);
    asset(&f, &s.style, None);
    assert_eq!(
        s.style
            .with_downcast::<ViewModelInstanceArtboard, _>(|v| v.base.property_value())
            .unwrap(),
        u32::MAX
    );
    assert_eq!(s.font(), 48.0);
    style_index(&s.style, index(&f, "Body"));
    assert_eq!(s.font(), 30.0);
    style_index(&s.style, index(&f, "Heading"));
    assert_eq!(s.font(), 48.0);
    style_index(&s.style, index(&f, "Plain"));
    assert_eq!(s.font(), 48.0);
    asset(&f, &s.style, Some("Body"));
    assert_eq!(s.font(), 30.0);
    asset(&f, &s.style, Some("Plain"));
    assert_eq!(s.font(), 30.0);
}
#[test]
fn style_bindings_per_host() {
    let f = load("text_style_global_binding.riv");
    let first = BindingScene::new(&f);
    let second = BindingScene::new(&f);
    asset(&f, &first.style, Some("Body"));
    assert_eq!(first.font(), 30.0);
    assert_eq!(second.font(), 48.0);
    let h = first_source::<TextStyleGlobal>(&source(&f, "Main"));
    assert!(
        h.with_downcast::<TextStyleGlobal, _>(|g| g.artboard_instance())
            .flatten()
            .is_none()
    );
}
