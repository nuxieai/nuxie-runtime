//! Literal text_style_global_bound_run_test.cpp at upstream 111bde6f.
#[path = "support/text_style_global.rs"]
mod support;
use nuxie_runtime::source::text::text_style_on_demand::TextStyleOnDemand;
use std::{cell::RefCell, rc::Rc};
use support::*;

const LOCAL: u32 = 0xFFFFFFFF;
const RED: u32 = 0xFFB40000;
const BLUE: u32 = 0xFF0000FF;
const GREEN: u32 = 0xFF00B400;
const YELLOW: u32 = 0xFFFFC800;

struct BoundScene {
    machine: RuntimeStateMachineInstanceHandle,
    text: CoreHandle,
    view_model: CoreHandle,
    artboard: RuntimeArtboardInstanceHandle,
    file: RuntimeFileHandle,
}
impl BoundScene {
    fn new(file: &RuntimeFileHandle) -> Self {
        // This upstream fixture explicitly uses file->artboard("Main")->instance(),
        // rather than File::artboardNamed (which additionally builds a focus tree).
        let artboard = instance(file, "Main");
        let machine = artboard
            .default_state_machine()
            .expect("default state machine");
        let view_model = vm(file, &artboard);
        machine.with_instance_mut(|m| m.bind_view_model_instance(view_model.clone()));
        let text = named::<Text>(&artboard, "Runs");
        let scene = Self {
            machine,
            text,
            view_model,
            artboard,
            file: file.clone(),
        };
        scene.frames();
        scene
    }
    fn frames(&self) {
        frames(&self.artboard, &self.machine, 4);
    }
    fn run(&self, index: usize) -> TextValueRunHandle {
        run(&self.text, index)
    }
    fn handle(&self) -> CoreHandle {
        handle(&self.text)
    }
    fn style_index(&self, name: &str, index: i32) {
        style_index(&property(&self.view_model, name), index);
        self.frames();
    }
    fn style(&self, name: &str, artboard_name: &str) {
        let index = index(&self.file, artboard_name);
        assert_ne!(index, -1);
        self.style_index(name, index);
    }
    fn count(&self) -> usize {
        self.text
            .with_downcast::<Text, _>(Text::on_demand_style_count)
            .unwrap()
    }
}
fn on_demand(run: &TextValueRunHandle) -> Rc<RefCell<TextStyleOnDemand>> {
    match reference(run) {
        TextStyleReferenceHandle::OnDemand(style) => style,
        TextStyleReferenceHandle::Authored(_) => {
            panic!("expected on-demand style, not global/paint")
        }
    }
}
fn authored_identity(run: &TextValueRunHandle) -> (usize, usize, u64) {
    match run {
        TextValueRunHandle::Core(run) => run.identity_key(),
        TextValueRunHandle::Runtime(_) => panic!("fixture run must be authored"),
    }
}
fn authored_paint(run: &TextValueRunHandle) -> bool {
    reference(run)
        .authored()
        .is_some_and(|h| h.is_type_of(TextStylePaint::TYPE_KEY))
}
fn pressed_in(run: &TextValueRunHandle) -> bool {
    let style = on_demand(run);
    let values = style
        .borrow()
        .instance()
        .expect("style instance")
        .view_model_instance()
        .expect("style view model");
    property(&values, "pressed")
        .with_downcast::<ViewModelInstanceBoolean, _>(|v| v.base.property_value())
        .expect("pressed boolean")
}

#[test]
fn runs_bound_to_stateful_style_get_an_instance_each() {
    let file = load("text_style_global_bound_run.riv");
    let scene = BoundScene::new(&file);
    let a = scene.run(0);
    let b = scene.run(1);
    let c = scene.run(2);
    let d = scene.run(3);
    let a_style = on_demand(&a);
    let b_style = on_demand(&b);
    assert!(!Rc::ptr_eq(&a_style, &b_style));
    assert_eq!(a_style.borrow().owner(), Some(authored_identity(&a)));
    assert_eq!(b_style.borrow().owner(), Some(authored_identity(&b)));
    assert!(!std::ptr::eq(
        a_style.borrow().instance().unwrap(),
        b_style.borrow().instance().unwrap()
    ));
    assert_eq!(color(&a), RED);
    assert_eq!(color(&b), RED);
    assert!(reference_is(&c, &scene.handle()));
    assert_eq!(color(&c), GREEN);
    assert!(authored_paint(&d));
    assert_eq!(color(&d), LOCAL);
    assert_eq!(scene.count(), 2);
}

#[test]
fn bound_runs_instance_keeps_what_its_listeners_write() {
    let file = load("text_style_global_bound_run.riv");
    let scene = BoundScene::new(&file);
    let a = scene.run(0);
    let b = scene.run(1);
    // "aaa " occupies offsets 0 through 3.
    let point = glyph(&scene.text, 1);
    click(&scene.machine, point);
    scene.frames();
    assert!(pressed_in(&a));
    assert!(!pressed_in(&b));
    assert_eq!(color(&a), BLUE);
    assert_eq!(color(&b), RED);
}

#[test]
fn runs_bound_to_plain_style_share_it() {
    let file = load("text_style_global_bound_run.riv");
    let scene = BoundScene::new(&file);
    let a = scene.run(0);
    let b = scene.run(1);
    scene.style("styleA", "Plain2");
    assert!(Rc::ptr_eq(&on_demand(&a), &on_demand(&b)));
    assert_eq!(on_demand(&a).borrow().owner(), None);
    assert_eq!(color(&a), YELLOW);
    assert_eq!(color(&b), YELLOW);
    assert_eq!(scene.count(), 1);
    scene.style("styleA", "Plain");
    assert!(reference_is(&a, &scene.handle()));
    assert!(reference_is(&b, &scene.handle()));
    assert_eq!(color(&a), GREEN);
    assert_eq!(scene.count(), 0);
    scene.style("styleA", "Stateful");
    assert!(!Rc::ptr_eq(&on_demand(&a), &on_demand(&b)));
    assert_eq!(color(&a), RED);
    assert_eq!(scene.count(), 2);
}

#[test]
fn bound_run_naming_no_text_style_draws_authored_style() {
    let file = load("text_style_global_bound_run.riv");
    let scene = BoundScene::new(&file);
    let a = scene.run(0);
    let c = scene.run(2);
    scene.style_index("styleA", -1);
    assert!(authored_paint(&a));
    assert_eq!(color(&a), LOCAL);
    scene.style("styleB", "Empty");
    assert!(authored_paint(&c));
    assert_eq!(color(&c), LOCAL);
    assert_eq!(scene.count(), 0);
}

// Rust integration regression: borrowed registry entry points must complete the
// same live style update without re-borrowing the run that the caller holds.
#[test]
fn borrowed_uint_style_setters_preserve_binding_and_new_authored_fallback() {
    use nuxie_runtime::source::generated::{
        core_registry::{CoreField, CoreRegistryObject},
        text::text_value_run_base::TextValueRunBase,
    };
    for direct_trait in [false, true] {
        let file = load("text_style_global_bound_run.riv");
        let scene = BoundScene::new(&file);
        let a = scene.run(0);
        let TextValueRunHandle::Core(owner) = &a else {
            panic!("fixture run must be authored");
        };
        let plain = scene.handle();
        let plain_id = scene.artboard.with_artboard(|a| a.id_of(&plain));
        assert_eq!(
            scene.artboard.with_artboard(|a| a.resolve_handle(plain_id)),
            Some(plain.clone())
        );
        let original = reference(&a);
        assert_eq!(color(&a), RED);
        owner
            .with_downcast_mut::<TextValueRun, _>(|run| {
                assert_ne!(run.base.style_id(), plain_id);
                if direct_trait {
                    CoreRegistryObject::set_uint(run, CoreField::TextValueRunStyleId, plain_id);
                } else {
                    CoreRegistry::set_uint(
                        run,
                        i32::from(TextValueRunBase::STYLE_ID_PROPERTY_KEY),
                        plain_id,
                    );
                }
                assert_eq!(run.base.style_id(), plain_id);
            })
            .unwrap();
        // The active binding continues to resolve to the per-run Stateful style.
        assert_eq!(reference(&a), original);
        assert_eq!(color(&a), RED);
        scene.frames();
        assert_eq!(reference(&a), original);
        assert_eq!(color(&a), RED);
        drop(original);
        // An invalid bound style now reveals the newly authored Plain handle.
        scene.style_index("styleA", -1);
        assert!(reference_is(&a, &plain));
        assert_eq!(color(&a), GREEN);
        assert_eq!(
            owner.with_downcast::<TextValueRun, _>(|r| r.base.style_id()),
            Some(plain_id)
        );
    }
}
