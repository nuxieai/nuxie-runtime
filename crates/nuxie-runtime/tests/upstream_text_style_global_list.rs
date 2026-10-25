//! Literal public text_style_global_list_test.cpp, upstream 0238f299.
#[path = "support/text_style_global.rs"]
mod support;
use support::*;
struct ListScene {
    file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    _values: CoreHandle,
    items: CoreHandle,
    a: CoreHandle,
    b: CoreHandle,
}
fn text_with_child<T: CoreType>(a: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    a.with_artboard(|a| {
        a.objects()
            .iter()
            .flatten()
            .find(|o| {
                o.with(|o| {
                    o.as_text().is_some()
                        && o.as_node()
                            .unwrap()
                            .children()
                            .iter()
                            .any(|c| c.is_type_of(T::TYPE_KEY))
                })
                .unwrap()
            })
            .cloned()
    })
    .unwrap()
}
impl ListScene {
    fn new(f: &RuntimeFileHandle) -> Self {
        let artboard = instance(f, "Main");
        let values = vm(f, &artboard);
        artboard.bind_view_model_instance(Some(values.clone()));
        let items = property(&values, "items");
        let a = text_with_child::<TextStylePaint>(&artboard);
        let b = text_with_child::<TextStyleGlobal>(&artboard);
        frame(&artboard);
        Self {
            file: f.clone(),
            artboard,
            _values: values,
            items,
            a,
            b,
        }
    }
    fn add(&self, name: &str) -> CoreHandle {
        let v = self
            .file
            .with_file(|f| f.create_view_model_instance_for_name("ItemVM"))
            .unwrap();
        string(&v, "textContent", "item ");
        string(&v, "textStyle", name);
        let item = self
            .items
            .insert_sibling(ViewModelInstanceListItem::default())
            .unwrap();
        item.with_downcast_mut::<ViewModelInstanceListItem, _>(|i| {
            i.set_view_model_instance(Some(v.clone()))
        })
        .unwrap();
        self.items
            .with_downcast_mut::<ViewModelInstanceList, _>(|i| i.add_item(item))
            .unwrap();
        v
    }
    fn font(&self, text: &CoreHandle, i: usize) -> f32 {
        frame(&self.artboard);
        font(&run(text, i))
    }
}
fn global(v: &CoreHandle) -> CoreHandle {
    property(v, "globalTextStyle")
}
fn count(text: &CoreHandle) -> usize {
    text.with_downcast::<Text, _>(|t| t.on_demand_style_count())
        .unwrap()
}
#[test]
fn list_runs_resolve_global_styles_on_demand() {
    let f = load("text_style_global_list.riv");
    let s = ListScene::new(&f);
    assert_eq!(s.font(&s.a, 0), 30.0);
    assert_eq!(s.font(&s.b, 0), 30.0);
    assert_eq!(count(&s.a), 1);
    assert_eq!(count(&s.b), 1);
    let heading = s.add("Local");
    style_index(&global(&heading), index(&f, "Heading"));
    assert_eq!(s.font(&s.a, 1), 20.0);
    assert_eq!(s.font(&s.b, 1), 48.0);
    assert_eq!(count(&s.a), 2);
    assert_eq!(count(&s.b), 1);
    let body = s.add("Local");
    asset(&f, &global(&body), Some("Body"));
    assert_eq!(s.font(&s.a, 2), 30.0);
    assert_eq!(count(&s.a), 2);
    assert!(reference(&run(&s.a, 2)) == reference(&run(&s.a, 0)));
    style_index(&global(&heading), -1);
    assert_eq!(s.font(&s.a, 1), 12.0);
    assert_eq!(s.font(&s.b, 1), 48.0);
    style_index(&global(&heading), index(&f, "Plain"));
    assert_eq!(s.font(&s.a, 1), 12.0);
    style_index(&global(&heading), 99999);
    assert_eq!(s.font(&s.a, 1), 12.0);
    while s
        .items
        .with_downcast::<ViewModelInstanceList, _>(|i| !i.list_items().is_empty())
        .unwrap()
    {
        s.items
            .with_downcast_mut::<ViewModelInstanceList, _>(|i| i.remove_item_at(0))
            .unwrap();
    }
    frame(&s.artboard);
    assert!(
        s.a.with_downcast::<Text, _>(|t| t.runs().is_empty())
            .unwrap()
    );
    assert_eq!(count(&s.a), 0);
    assert_eq!(count(&s.b), 0);
    assert_eq!(
        s.a.with(|t| t.advancing_component_quiet_state()).unwrap(),
        QuietState::Quiet
    );
}
#[test]
fn list_runs_reresolve_after_handle_swaps() {
    let f = load("text_style_global_list.riv");
    let s = ListScene::new(&f);
    let heading = s.add("");
    style_index(&global(&heading), index(&f, "Heading"));
    assert_eq!(s.font(&s.b, 1), 48.0);
    let h = first::<TextStyleGlobal>(&s.artboard);
    let swap = h
        .insert_sibling(ViewModelInstanceArtboard::default())
        .unwrap();
    asset(&f, &swap, Some("Body"));
    update_global(&h, Some(swap));
    assert_eq!(s.font(&s.b, 1), 20.0);
    assert!(!reference_is(&run(&s.b, 1), &h));
}
#[test]
fn item_view_models_outlive_text() {
    let f = load("text_style_global_list.riv");
    let item;
    {
        let s = ListScene::new(&f);
        item = s.add("Local");
        style_index(&global(&item), index(&f, "Heading"));
        assert_eq!(s.font(&s.a, 1), 20.0);
    }
    style_index(&global(&item), index(&f, "Body"));
    string(&item, "textStyle", "Other");
}
