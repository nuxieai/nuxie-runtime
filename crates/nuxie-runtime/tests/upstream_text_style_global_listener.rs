//! Literal public text_style_global_listener_test.cpp, upstream 0238f299.
#[path = "support/text_style_global.rs"]
mod support;
use support::*;
const IDLE: u32 = 0xff000000;
const HOVER: u32 = 0xff0000ff;
struct LinkScene {
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    values: CoreHandle,
    links: CoreHandle,
}
impl LinkScene {
    fn new(f: &RuntimeFileHandle) -> Self {
        let artboard = instance(f, "Main");
        let machine = artboard.default_state_machine_handle().unwrap();
        let values = vm(f, &artboard);
        machine.with_instance_mut(|m| m.bind_view_model_instance(values.clone()));
        let links = named::<Text>(&artboard, "Links");
        frames(&artboard, &machine, 2);
        Self {
            artboard,
            machine,
            values,
            links,
        }
    }
    fn frame(&self) {
        frames(&self.artboard, &self.machine, 1)
    }
    fn run(&self, i: usize) -> TextValueRunHandle {
        run(&self.links, i)
    }
    fn move_to(&self, p: Vec2D) {
        self.machine
            .with_instance_mut(|m| m.pointer_move(p, 0.0, 0));
        self.frame();
    }
    fn move_at(&self, i: u32) {
        self.move_to(glyph(&self.links, i));
    }
}
#[test]
fn targetless_listeners_listen_to_runs() {
    let f = load("text_style_global_listeners.riv");
    let link = source(&f, "Link");
    let plain = source(&f, "Plain");
    assert!(TextStyleShareCache::analyze(&link, Some(&f)).listens_to_runs);
    assert!(!TextStyleShareCache::analyze(&link, Some(&f)).shareable);
    assert!(!TextStyleShareCache::analyze(&plain, Some(&f)).listens_to_runs);
    assert!(
        f.with_file(|f| f.text_style_shares())
            .borrow_mut()
            .any_listens_to_runs()
    );
    let machine = link
        .with_downcast::<Artboard, _>(|a| a.state_machine_handle_at(0))
        .flatten()
        .unwrap();
    assert_eq!(
        machine
            .with_downcast::<StateMachine, _>(|m| m.listener_count())
            .unwrap(),
        3
    );
    let sharing = load("text_style_global_sharing.riv");
    assert!(
        !sharing
            .with_file(|f| f.text_style_shares())
            .borrow_mut()
            .any_listens_to_runs()
    );
    let a = instance(&sharing, "Main");
    let t = named::<Text>(&a, "Text A");
    assert!(
        !t.with_downcast::<Text, _>(|t| t.may_have_listening_runs())
            .unwrap()
    );
}
#[test]
fn runs_on_one_handle_share_instance() {
    let f = load("text_style_global_listeners.riv");
    let s = LinkScene::new(&f);
    let one = s.run(0);
    let two = s.run(1);
    let plain = s.run(2);
    let three = s.run(3);
    assert!(reference(&one) == reference(&two));
    assert!(reference(&three) != reference(&one));
    let h = reference(&one).authored().unwrap();
    assert!(h.is_type_of(TextStyleGlobal::TYPE_KEY));
    assert_eq!(paint(&one), resolved(&h));
    assert_eq!(paint(&two), resolved(&h));
    assert_ne!(paint(&three), paint(&one));
    assert!(machine_eq(&one, &two));
    assert!(!machine_eq(&three, &one));
    assert!(reference(&plain).run_state_machine().is_none());
    assert!(s.machine.with_instance(|m| m.has_listeners()));
    assert_eq!(run_machines(&s.links).len(), 2);
    assert_eq!(font(&one), 30.0);
    assert_eq!(font(&three), 30.0);
    assert_eq!(color(&one), IDLE);
    assert_eq!(color(&three), IDLE);
}
#[test]
fn hovering_pressing_run_restyles_handle() {
    let f = load("text_style_global_listeners.riv");
    let s = LinkScene::new(&f);
    let one = s.run(0);
    let two = s.run(1);
    let three = s.run(3);
    s.move_at(1);
    assert!(input(&one, "hover"));
    assert!(!input(&three, "hover"));
    s.frame();
    assert_eq!(color(&one), HOVER);
    assert_eq!(color(&two), HOVER);
    assert_eq!(color(&three), IDLE);
    s.move_at(5);
    assert!(input(&two, "hover"));
    s.frame();
    assert_eq!(color(&one), HOVER);
    s.move_at(15);
    assert!(!input(&one, "hover"));
    assert!(input(&three, "hover"));
    s.frame();
    assert_eq!(color(&one), IDLE);
    assert_eq!(color(&two), IDLE);
    assert_eq!(color(&three), HOVER);
    s.move_at(9);
    assert!(!input(&one, "hover"));
    assert!(!input(&three, "hover"));
    s.move_to(Vec2D::new(590.0, 390.0));
    s.frame();
    assert_eq!(color(&one), IDLE);
    assert_eq!(color(&three), IDLE);
    click(&s.machine, glyph(&s.links, 5));
    s.frame();
    assert!(input(&one, "clicked"));
    assert!(!input(&three, "clicked"));
}
#[test]
fn nested_runs_listen() {
    let f = load("text_style_global_listeners.riv");
    let s = LinkScene::new(&f);
    let t = named::<Text>(&card(&s.artboard), "Card Link");
    assert_eq!(t.with_downcast::<Text, _>(|t| t.runs().len()).unwrap(), 1);
    let r = run(&t, 0);
    s.move_to(glyph(&t, 1));
    assert!(input(&r, "hover"));
    assert!(!input(&s.run(0), "hover"));
    s.frame();
    assert_eq!(color(&r), HOVER);
    s.move_to(Vec2D::new(590.0, 390.0));
    assert!(!input(&r, "hover"));
}
#[test]
fn exposed_inputs_reach_shared_instance() {
    let f = load("text_style_global_listeners.riv");
    let s = LinkScene::new(&f);
    let one = s.run(0);
    let two = s.run(1);
    let three = s.run(3);
    s.move_at(1);
    s.frame();
    assert!(input(&one, "hover"));
    let before = glyph(&s.links, 5).x;
    number(&s.values, "linkSize", 44.0);
    s.frame();
    assert_eq!(font(&one), 44.0);
    assert_eq!(font(&two), 44.0);
    assert!(glyph(&s.links, 5).x > before);
    assert_eq!(font(&three), 30.0);
    assert!(input(&one, "hover"));
    assert_eq!(color(&one), HOVER);
}
#[test]
fn swapping_listening_style_rebuilds_listeners() {
    let f = load("text_style_global_listeners.riv");
    let s = LinkScene::new(&f);
    let style = property(&s.values, "linkStyle");
    s.move_at(1);
    assert!(input(&s.run(0), "hover"));
    style_index(&style, index(&f, "Plain"));
    s.frame();
    s.frame();
    assert!(reference(&s.run(0)).run_state_machine().is_none());
    assert_eq!(
        Some(paint(&s.run(0))),
        reference(&s.run(0)).resolved_paint()
    );
    s.move_at(5);
    style_index(&style, index(&f, "Link"));
    s.frame();
    s.frame();
    s.move_to(Vec2D::new(590.0, 390.0));
    assert!(!input(&s.run(0), "hover"));
    s.move_at(5);
    assert!(input(&s.run(0), "hover"));
    assert!(input(&s.run(1), "hover"));
}
#[test]
fn listening_instances_teardown_any_order() {
    for section in 0..3 {
        let f = load("text_style_global_listeners.riv");
        if section < 2 {
            let s = LinkScene::new(&f);
            s.move_at(if section == 0 { 1 } else { 5 });
            let LinkScene {
                artboard,
                machine,
                values,
                links,
            } = s;
            if section == 0 {
                drop(machine);
                drop(artboard);
            } else {
                drop(artboard);
                drop(machine);
            }
            drop(values);
            drop(links);
        } else {
            let a = instance(&f, "Main");
            frame(&a);
            let t = named::<Text>(&a, "Links");
            assert_eq!(run_machines(&t).len(), 2);
            drop(a);
        }
        assert_eq!(live(&f), 0);
    }
}
