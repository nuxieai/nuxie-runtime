//! Literal public text_style_global_list_item_test.cpp, upstream 0238f299.
#[path = "support/text_style_global.rs"]
mod support;
use support::*;
const VISITED: u32 = 0xffb40000;
const NON_VISITED: u32 = 0xff009eb9;
struct ListScene {
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    values: CoreHandle,
    list: CoreHandle,
}
impl ListScene {
    fn new(f: &RuntimeFileHandle) -> Self {
        let artboard = instance(f, "Main");
        let machine = artboard.default_state_machine_handle().unwrap();
        let values = vm(f, &artboard);
        machine.with_instance_mut(|m| m.bind_view_model_instance(values.clone()));
        let list = named::<Text>(&artboard, "List");
        frames(&artboard, &machine, 4);
        Self {
            artboard,
            machine,
            values,
            list,
        }
    }
    fn frames(&self) {
        frames(&self.artboard, &self.machine, 4)
    }
    fn run(&self, i: usize) -> TextValueRunHandle {
        run(&self.list, i)
    }
    fn item(&self, i: usize) -> CoreHandle {
        let list = property(&self.values, "listProperty1");
        let item = list
            .with_downcast::<ViewModelInstanceList, _>(|l| {
                assert!(l.list_items().len() > i);
                l.list_items()[i].clone()
            })
            .unwrap();
        item.with_downcast::<ViewModelInstanceListItem, _>(|i| i.view_model_instance())
            .flatten()
            .unwrap()
    }
    fn url_state(&self, i: usize, value: u32) {
        property(&self.item(i), "urlState")
            .with_downcast_mut::<ViewModelInstanceEnum, _>(|v| v.set_value_at(value))
            .unwrap();
        self.frames();
    }
    fn same_style(&self, f: &RuntimeFileHandle) {
        style_index(
            &property(&self.item(1), "globalTextStyle"),
            index(f, "Text Style1"),
        );
        self.frames();
    }
}
fn not_global(r: &TextValueRunHandle) -> bool {
    reference(r)
        .authored()
        .is_none_or(|h| !h.is_type_of(TextStyleGlobal::TYPE_KEY))
}
fn values_of(r: &TextValueRunHandle) -> CoreHandle {
    match reference(r) {
        TextStyleReferenceHandle::OnDemand(style) => style
            .borrow()
            .instance()
            .unwrap()
            .view_model_instance()
            .unwrap(),
        _ => panic!("expected on-demand style"),
    }
}
fn pressed(v: &CoreHandle) -> bool {
    property(v, "pressed")
        .with_downcast::<ViewModelInstanceBoolean, _>(|v| v.base.property_value())
        .unwrap()
}
#[test]
fn one_list_run_style_reads_its_item() {
    let f = load("text_style_global_list_item.riv");
    let s = ListScene::new(&f);
    let a = s.run(0);
    let b = s.run(1);
    assert_eq!(color(&a), VISITED);
    assert_eq!(color(&b), NON_VISITED);
    s.url_state(0, 1);
    assert_eq!(color(&a), NON_VISITED);
    s.url_state(1, 0);
    assert_eq!(color(&b), VISITED);
}
#[test]
fn list_runs_naming_stateful_style_get_separate_instances() {
    let f = load("text_style_global_list_item.riv");
    let s = ListScene::new(&f);
    s.same_style(&f);
    let a = s.run(0);
    let b = s.run(1);
    assert!(reference(&a) != reference(&b));
    assert!(not_global(&a));
    assert!(not_global(&b));
    assert_eq!(color(&a), VISITED);
    assert_eq!(color(&b), NON_VISITED);
    s.url_state(0, 1);
    assert_eq!(color(&a), NON_VISITED);
    assert_eq!(color(&b), NON_VISITED);
    s.url_state(1, 0);
    assert_eq!(color(&a), NON_VISITED);
    assert_eq!(color(&b), VISITED);
}
#[test]
fn list_item_instance_keeps_listener_writes() {
    let f = load("text_style_global_list_item.riv");
    let s = ListScene::new(&f);
    s.same_style(&f);
    let a = s.run(0);
    let b = s.run(1);
    assert!(not_global(&a));
    assert!(not_global(&b));
    let h = handle(&s.list);
    let hv = h
        .with_downcast::<TextStyleGlobal, _>(|g| g.style_view_model_instance())
        .flatten()
        .unwrap();
    assert_ne!(values_of(&a), hv);
    assert_ne!(values_of(&a), values_of(&b));
    click(&s.machine, glyph(&s.list, 1));
    s.frames();
    assert!(pressed(&values_of(&a)));
    assert!(!pressed(&values_of(&b)));
    assert!(!pressed(&hv));
}
