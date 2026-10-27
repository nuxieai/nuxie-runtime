//! Full public semantic_nested_collapse_test.cpp at upstream 73d7b1c2.
//! Accessibility assertions retain upstream expectations under the CSS layout adaptation.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::RuntimeArtboardInstanceHandle,
    artboard_component_list::ArtboardComponentList,
    core::CoreHandle,
    drawable_flag::DrawableFlag,
    file::RuntimeFileHandle,
    generated::core_registry::CoreRegistry,
    semantic::{
        semantic_data::SemanticData,
        semantic_dirt::SemanticDirt,
        semantic_manager::RuntimeSemanticManagerHandle,
        semantic_snapshot::{SemanticsDiff, SemanticsDiffNode},
    },
    solo::Solo,
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    rc::Rc,
};
const FIXTURE: &str = "semantic/nested_collapse.riv";
fn shown() -> BTreeSet<String> {
    labels(&[
        "Hide or show",
        "Screen",
        "Outer",
        "Solo active",
        "Row one",
        "Row two",
        "Tie A",
        "Tie B",
        "Bare header",
        "Bare leaf",
        "Bare footer",
        "Middle",
        "Inner",
        "Root A",
        "Root B",
        "Deep leaf",
        "Sibling A",
        "Sibling B",
    ])
}
fn labels(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|s| (*s).into()).collect()
}
fn without(mut values: BTreeSet<String>, drop: &[&str]) -> BTreeSet<String> {
    for s in drop {
        values.remove(*s);
    }
    values
}
fn with(mut values: BTreeSet<String>, add: &[&str]) -> BTreeSet<String> {
    values.extend(add.iter().map(|s| (*s).to_owned()));
    values
}
#[derive(Default)]
struct LiveTree {
    nodes: BTreeMap<u32, SemanticsDiffNode>,
}
impl LiveTree {
    fn apply(&mut self, diff: SemanticsDiff) {
        for id in diff.removed {
            self.nodes.remove(&id);
        }
        for node in diff.added {
            assert!(
                !self.nodes.contains_key(&node.id),
                "duplicate add: {}",
                node.label
            );
            self.nodes.insert(node.id, node);
        }
        for node in diff.moved.into_iter().chain(diff.updated_semantic) {
            self.nodes.insert(node.id, node);
        }
        for geometry in diff.updated_geometry {
            if let Some(node) = self.nodes.get_mut(&geometry.id) {
                node.set_bounds(geometry.bounds());
            }
        }
    }
    fn labels(&self) -> BTreeSet<String> {
        self.nodes.values().map(|n| n.label.clone()).collect()
    }
    fn find(&self, label: &str) -> Option<&SemanticsDiffNode> {
        self.nodes.values().find(|n| n.label == label)
    }
    fn parents(&self) -> BTreeMap<String, String> {
        self.nodes
            .values()
            .map(|n| {
                (
                    n.label.clone(),
                    self.nodes
                        .get(&(n.parent_id as u32))
                        .map(|p| p.label.clone())
                        .unwrap_or_else(|| "<root>".into()),
                )
            })
            .collect()
    }
}
fn read_file(path: &str) -> RuntimeFileHandle {
    let root =
        PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR"));
    let bytes = std::fs::read(root.join("tests/unit_tests/assets").join(path)).unwrap();
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
fn property(vm: &CoreHandle, name: &str) -> CoreHandle {
    vm.with_downcast::<ViewModelInstance, _>(|vm| vm.property_value_named(name))
        .flatten()
        .unwrap()
}
fn boolean(vm: &CoreHandle, name: &str) -> CoreHandle {
    let value = property(vm, name);
    assert!(
        value
            .with_downcast::<ViewModelInstanceBoolean, _>(|_| ())
            .is_some()
    );
    value
}
fn set_boolean(vm: &CoreHandle, name: &str, value: bool) {
    boolean(vm, name)
        .with_downcast_mut::<ViewModelInstanceBoolean, _>(|b| b.set_value(value))
        .unwrap();
}
fn outer_vm(vm: &CoreHandle) -> CoreHandle {
    property(vm, "outer")
        .with_downcast::<ViewModelInstanceViewModel, _>(|p| p.reference_view_model_instance())
        .flatten()
        .unwrap()
}
fn advance(sm: &RuntimeStateMachineInstanceHandle) {
    for _ in 0..10 {
        sm.advance_and_apply(0.1);
    }
}
fn manager(sm: &RuntimeStateMachineInstanceHandle) -> RuntimeSemanticManagerHandle {
    sm.with_instance(|sm| sm.semantic_manager()).unwrap()
}
fn settle_diff(sm: &RuntimeStateMachineInstanceHandle) -> SemanticsDiff {
    advance(sm);
    manager(sm).with_semantic_manager_mut(|m| m.drain_diff())
}
struct Fixture {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    sm: RuntimeStateMachineInstanceHandle,
    vmi: CoreHandle,
}
impl Fixture {
    fn load(path: &str) -> Self {
        let file = read_file(path);
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        let sm = artboard.state_machine_instance_handle(0).unwrap();
        sm.with_instance_mut(|sm| sm.enable_semantics());
        let vmi = file
            .with_file(|f| {
                f.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .unwrap();
        artboard.bind_view_model_instance(Some(vmi.clone()));
        sm.with_instance_mut(|sm| sm.bind_view_model_instance(vmi.clone()));
        advance(&sm);
        Self {
            _file: file,
            artboard,
            sm,
            vmi,
        }
    }
}
struct CollapseScene {
    f: Fixture,
    mgr: RuntimeSemanticManagerHandle,
    outer: CoreHandle,
    tree: LiveTree,
}
impl CollapseScene {
    fn new() -> Self {
        let f = Fixture::load(FIXTURE);
        let mgr = manager(&f.sm);
        let outer = outer_vm(&f.vmi);
        let mut tree = LiveTree::default();
        tree.apply(mgr.with_semantic_manager_mut(|m| m.drain_diff()));
        Self {
            f,
            mgr,
            outer,
            tree,
        }
    }
    fn set_visible(&mut self, value: bool) {
        set_boolean(&self.f.vmi, "visible", value);
        self.settle();
    }
    fn set_outer(&mut self, name: &str, value: bool) {
        set_boolean(&self.outer, name, value);
        self.settle();
    }
    fn settle(&mut self) {
        self.tree.apply(settle_diff(&self.f.sm));
    }
    fn outer_instance(&self) -> RuntimeArtboardInstanceHandle {
        let hosts = self
            .f
            .artboard
            .with_artboard(|a| a.base.nested_artboards().to_vec());
        assert_eq!(hosts.len(), 1);
        hosts[0]
            .with(|h| {
                h.as_nested_artboard()
                    .and_then(|h| h.artboard_instance_default())
            })
            .flatten()
            .unwrap()
    }
    fn data(&self, label: &str) -> CoreHandle {
        let id = self.tree.find(label).unwrap().id;
        let node = self
            .mgr
            .with_semantic_manager(|m| m.node_by_id(id))
            .unwrap();
        let data = node.borrow().semantic_data.clone().unwrap();
        data
    }
    fn order(&self) -> Vec<String> {
        self.mgr
            .with_semantic_manager(|m| m.bounds_snapshot())
            .iter()
            .map(|b| self.tree.nodes.get(&b.id).unwrap().label.clone())
            .collect()
    }
    fn count(&self) -> usize {
        self.mgr.with_semantic_manager(|m| m.node_count())
    }
}
fn find_data(artboard: &RuntimeArtboardInstanceHandle, label: &str) -> Option<CoreHandle> {
    let objects = artboard.with_artboard(|a| a.base.objects().to_vec());
    for object in objects.into_iter().flatten() {
        if object
            .with_downcast::<SemanticData, _>(|d| d.base.label() == label)
            .unwrap_or(false)
        {
            return Some(object);
        }
    }
    let hosts = artboard.with_artboard(|a| a.base.nested_artboards().to_vec());
    for host in hosts {
        if let Some(nested) = host
            .with(|h| {
                h.as_nested_artboard()
                    .and_then(|h| h.artboard_instance_default())
            })
            .flatten()
        {
            if let Some(data) = find_data(&nested, label) {
                return Some(data);
            }
        }
    }
    None
}
fn read_before(order: &[String], first: &str, second: &str) -> bool {
    matches!((order.iter().position(|s| s == first), order.iter().position(|s| s == second)), (Some(a), Some(b)) if a < b)
}
fn hidden(data: &CoreHandle, value: bool) {
    assert!(CoreRegistry::set_bool_handle(data, 1004, value));
}
fn collapse(host: &CoreHandle, value: bool) {
    host.with_mut(|h| h.component_collapse(value)).unwrap();
}
fn nested(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    artboard
        .with_artboard(|a| a.base.nested_artboard_handle(name))
        .unwrap()
}
#[test]
fn initial_tree_holds_what_is_shown_in_visual_order() {
    let s = CollapseScene::new();
    assert_eq!(s.tree.labels(), shown());
    let order = s.order();
    assert!(read_before(&order, "Bare header", "Bare leaf"));
    assert!(read_before(&order, "Bare leaf", "Bare footer"));
}
#[test]
fn showing_hidden_layout_restores_tree_exactly() {
    let mut s = CollapseScene::new();
    let parents = s.tree.parents();
    let order = s.order();
    let registered = s.count();
    for cycle in 0..2 {
        s.set_visible(false);
        assert_eq!(s.tree.labels(), labels(&["Hide or show"]), "cycle {cycle}");
        assert_eq!(
            s.mgr.with_semantic_manager(|m| m.root_id()),
            s.tree.find("Hide or show").unwrap().id
        );
        s.set_visible(true);
        assert_eq!(s.tree.labels(), shown());
        assert_eq!(s.tree.parents(), parents);
        assert_eq!(s.order(), order);
        assert_eq!(s.mgr.with_semantic_manager(|m| m.root_id()), 0);
        assert_eq!(s.count(), registered);
        for entry in s.tree.nodes.values() {
            assert!(entry.max_x > entry.min_x, "{}", entry.label);
            assert!(entry.max_y > entry.min_y, "{}", entry.label);
        }
    }
}
#[test]
fn uncollapsing_host_directly_restores_tree_exactly() {
    let mut s = CollapseScene::new();
    let parents = s.tree.parents();
    let order = s.order();
    let hosts =
        s.f.artboard
            .with_artboard(|a| a.base.nested_artboards().to_vec());
    assert_eq!(hosts.len(), 1);
    for _ in 0..2 {
        collapse(&hosts[0], true);
        s.settle();
        assert_eq!(s.tree.labels(), labels(&["Hide or show", "Screen"]));
        collapse(&hosts[0], false);
        s.settle();
        assert_eq!(s.tree.labels(), shown());
        assert_eq!(s.tree.parents(), parents);
        assert_eq!(s.order(), order);
    }
}
#[test]
fn semantic_data_sharing_component_are_siblings_under_real_parent() {
    let mut s = CollapseScene::new();
    for _ in 0..2 {
        let parents = s.tree.parents();
        for (child, parent) in [
            ("Root A", "Middle"),
            ("Root B", "Middle"),
            ("Sibling A", "Inner"),
            ("Sibling B", "Inner"),
            ("Deep leaf", "Middle"),
        ] {
            assert_eq!(parents[child], parent);
        }
        s.set_visible(false);
        s.set_visible(true);
    }
}
#[test]
fn nodes_hidden_for_own_reason_return_when_reason_clears() {
    // Each upstream SECTION starts with a fresh scene.
    for section in 0..6 {
        let mut s = CollapseScene::new();
        match section {
            0 => {
                s.set_visible(false);
                s.set_visible(true);
                s.set_outer("details", true);
                assert_eq!(s.tree.labels(), with(shown(), &["Outer details"]));
                s.set_outer("details", false);
                assert_eq!(s.tree.labels(), shown());
            }
            1 => {
                let header = s.data("Bare header");
                s.set_visible(false);
                hidden(&header, true);
                s.settle();
                hidden(&header, false);
                s.settle();
                s.set_visible(true);
                assert_eq!(s.tree.labels(), shown());
            }
            2 => {
                let header = s.data("Bare header");
                s.set_visible(false);
                hidden(&header, true);
                s.set_visible(true);
                assert_eq!(s.tree.labels(), without(shown(), &["Bare header"]));
                hidden(&header, false);
                s.settle();
                assert_eq!(s.tree.labels(), shown());
            }
            3 => {
                s.set_outer("soloSwitched", true);
                let switched = with(without(shown(), &["Solo active"]), &["Solo inactive"]);
                assert_eq!(s.tree.labels(), switched);
                s.set_visible(false);
                s.set_visible(true);
                assert_eq!(s.tree.labels(), switched);
                s.set_outer("soloSwitched", false);
                assert_eq!(s.tree.labels(), shown());
            }
            4 => {
                let solo = s
                    .outer_instance()
                    .with_artboard(|a| a.base.find_handle::<Solo>("Solo"))
                    .unwrap();
                let registered = s.count();
                s.set_visible(false);
                solo.with_downcast_mut::<Solo, _>(|s| s.update_by_name("Inactive"))
                    .unwrap();
                s.settle();
                assert_eq!(s.tree.labels(), labels(&["Hide or show"]));
                s.set_visible(true);
                assert_eq!(s.tree.labels(), shown());
                assert_eq!(s.count(), registered);
            }
            5 => {
                s.set_outer("listShown", false);
                let no_rows = without(shown(), &["Row one", "Row two"]);
                assert_eq!(s.tree.labels(), no_rows);
                s.set_visible(false);
                s.set_visible(true);
                assert_eq!(s.tree.labels(), no_rows);
                s.set_outer("listShown", true);
                assert_eq!(s.tree.labels(), shown());
            }
            _ => unreachable!(),
        }
    }
}
#[test]
fn hiding_host_hides_artboards_without_semantic_data_above() {
    let mut s = CollapseScene::new();
    let order = s.order();
    s.set_outer("bare", false);
    assert_eq!(
        s.tree.labels(),
        without(shown(), &["Bare header", "Bare leaf", "Bare footer"])
    );
    s.set_outer("bare", true);
    assert_eq!(s.tree.labels(), shown());
    assert_eq!(s.order(), order);
}
#[test]
fn semantic_actions_do_not_reach_nodes_under_hidden_host() {
    let mut s = CollapseScene::new();
    let outer_id = s.tree.find("Outer").unwrap().id;
    let tapped = boolean(&s.outer, "tapped");
    s.set_visible(false);
    s.f.sm.fire_semantic_action(outer_id, 0);
    s.settle();
    s.set_visible(true);
    assert!(
        !tapped
            .with_downcast::<ViewModelInstanceBoolean, _>(|v| v.base.property_value())
            .unwrap()
    );
    s.f.sm.fire_semantic_action(outer_id, 0);
    s.settle();
    assert!(
        tapped
            .with_downcast::<ViewModelInstanceBoolean, _>(|v| v.base.property_value())
            .unwrap()
    );
}
#[test]
fn list_row_removed_while_host_hidden_stays_unreachable_then_released() {
    let mut s = CollapseScene::new();
    let row_id = s.tree.find("Row one").unwrap().id;
    let rows = property(&s.outer, "rows");
    assert!(
        rows.with_downcast::<ViewModelInstanceList, _>(|_| ())
            .is_some()
    );
    s.set_visible(false);
    rows.with_downcast_mut::<ViewModelInstanceList, _>(|r| r.remove_item_at(0))
        .unwrap();
    s.settle();
    assert!(
        !s.mgr
            .with_semantic_manager(|m| m.is_in_tree(m.node_by_id(row_id).as_ref()))
    );
    s.set_visible(true);
    assert!(
        s.mgr
            .with_semantic_manager(|m| m.node_by_id(row_id).is_none())
    );
    assert_eq!(s.tree.labels(), without(shown(), &["Row one"]));
}
#[test]
fn cleaning_hidden_artboard_releases_every_node_below() {
    let mut s = CollapseScene::new();
    let below: Vec<_> = s
        .tree
        .nodes
        .values()
        .filter(|n| n.label != "Hide or show" && n.label != "Screen")
        .map(|n| n.id)
        .collect();
    assert_eq!(below.len(), shown().len() - 2);
    let outer = s.outer_instance();
    s.set_visible(false);
    outer.cleanup_semantic_tree();
    for id in below {
        assert!(s.mgr.with_semantic_manager(|m| m.node_by_id(id).is_none()));
    }
    assert_eq!(s.count(), 2);
}
#[test]
fn hidden_node_stays_out_when_own_layout_hides_and_shows() {
    let mut s = CollapseScene::new();
    s.set_outer("details", true);
    assert!(s.tree.find("Outer details").is_some());
    hidden(&s.data("Outer details"), true);
    s.settle();
    assert_eq!(s.tree.labels(), shown());
    s.set_outer("details", false);
    s.set_outer("details", true);
    assert_eq!(s.tree.labels(), shown());
}
#[test]
fn anchor_hidden_for_own_reason_is_released_on_cleanup() {
    let mut s = CollapseScene::new();
    let data = s.data("Outer");
    let id = s.tree.find("Outer").unwrap().id;
    let outer = s.outer_instance();
    hidden(&data, true);
    s.settle();
    let middle = nested(&outer, "Middle");
    collapse(&middle, true);
    s.settle();
    collapse(&middle, false);
    s.settle();
    outer.cleanup_semantic_tree();
    assert!(s.mgr.with_semantic_manager(|m| m.node_by_id(id).is_none()));
}
#[test]
fn hiding_one_of_two_placements_at_same_spot_keeps_order() {
    let mut s = CollapseScene::new();
    let order = s.order();
    assert!(read_before(&order, "Tie A", "Tie B"));
    let tie = nested(&s.outer_instance(), "Tie A host");
    for _ in 0..2 {
        collapse(&tie, true);
        s.settle();
        assert!(s.tree.find("Tie A").is_none());
        collapse(&tie, false);
        s.settle();
        assert_eq!(s.order(), order);
    }
}
#[test]
fn request_focus_refuses_node_under_hidden_host() {
    let mut s = CollapseScene::new();
    let id = s.tree.find("Middle").unwrap().id;
    assert!(s.mgr.request_focus(id));
    s.set_visible(false);
    assert!(!s.mgr.request_focus(id));
}
struct PooledList {
    _f: Fixture,
    list: CoreHandle,
    mgr: RuntimeSemanticManagerHandle,
}
impl PooledList {
    fn new() -> Self {
        let f = Fixture::load("component_list_virtualized.riv");
        let mgr = manager(&f.sm);
        let list = f
            .artboard
            .with_artboard(|a| a.base.find_handle::<ArtboardComponentList>("List"))
            .unwrap();
        let p = Self { _f: f, list, mgr };
        assert_eq!(p.realized_in_tree(), 5);
        p
    }
    fn row_in_tree(&self, index: i32) -> bool {
        let Some(row) = self
            .list
            .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(index))
            .flatten()
        else {
            return false;
        };
        let Some(boundary) = row.with_artboard(|a| a.base.semantic_boundary_node()) else {
            return false;
        };
        let id = boundary.borrow().id();
        self.mgr
            .with_semantic_manager(|m| m.node_by_id(id).is_some_and(|n| Rc::ptr_eq(&n, &boundary)))
            && !row.with_artboard(|a| a.base.is_host_collapsed())
    }
    fn realized_in_tree(&self) -> usize {
        let count = self
            .list
            .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_count())
            .unwrap();
        (0..count).filter(|i| self.row_in_tree(*i as i32)).count()
    }
}
#[test]
fn destroying_pooled_list_row_leaves_tree_safe_to_flatten() {
    let p = PooledList::new();
    p.list
        .with_downcast_mut::<ArtboardComponentList, _>(|l| {
            l.remove_virtualizable(4);
            l.clear();
        })
        .unwrap();
    let diff = p.mgr.with_semantic_manager_mut(|m| {
        m.mark_dirty(SemanticDirt::STRUCTURE);
        m.drain_diff()
    });
    assert!(diff.added.is_empty());
}
#[test]
fn row_pooled_while_list_hidden_comes_back_when_reused() {
    let p = PooledList::new();
    collapse(&p.list, true);
    ArtboardComponentList::add_virtualizable_occurrence(&p.list, 10);
    assert!(!p.row_in_tree(10));
    p.list
        .with_downcast_mut::<ArtboardComponentList, _>(|l| l.remove_virtualizable(10))
        .unwrap();
    collapse(&p.list, false);
    ArtboardComponentList::add_virtualizable_occurrence(&p.list, 11);
    assert!(p.row_in_tree(11));
}
fn replacement_scene() -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle, CoreHandle) {
    let file = read_file(FIXTURE);
    let artboard = file.with_file(|f| f.artboard_default()).unwrap();
    let vmi = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(artboard.core_handle()))
        .unwrap();
    artboard.bind_view_model_instance(Some(vmi.clone()));
    {
        let first = artboard.state_machine_instance_handle(0).unwrap();
        first.with_instance_mut(|m| {
            m.enable_semantics();
            m.bind_view_model_instance(vmi.clone());
        });
        advance(&first);
    }
    (file, artboard, vmi)
}
fn new_machine(
    artboard: &RuntimeArtboardInstanceHandle,
    vmi: &CoreHandle,
) -> RuntimeStateMachineInstanceHandle {
    let sm = artboard.state_machine_instance_handle(0).unwrap();
    sm.with_instance_mut(|m| {
        m.enable_semantics();
        m.bind_view_model_instance(vmi.clone());
    });
    sm
}
#[test]
fn node_hidden_by_own_layout_survives_new_state_machine() {
    let (_file, artboard, vmi) = replacement_scene();
    let sm = new_machine(&artboard, &vmi);
    let mut tree = LiveTree::default();
    tree.apply(settle_diff(&sm));
    assert_eq!(tree.labels(), shown());
    let mgr = manager(&sm);
    for label in ["Outer details", "Inner details"] {
        let data = find_data(&artboard, label).unwrap();
        let node = data
            .with_downcast::<SemanticData, _>(|d| {
                assert!(d.has_semantic_node());
                d.existing_semantic_node().unwrap()
            })
            .unwrap();
        assert!(node.borrow().manager().unwrap().ptr_eq(&mgr));
        let id = node.borrow().id();
        assert!(mgr.with_semantic_manager(|m| Rc::ptr_eq(&m.node_by_id(id).unwrap(), &node)));
        assert!(!mgr.with_semantic_manager(|m| m.is_in_tree(Some(&node))));
    }
    set_boolean(&outer_vm(&vmi), "details", true);
    tree.apply(settle_diff(&sm));
    assert_eq!(tree.labels(), with(shown(), &["Outer details"]));
}
#[test]
fn layout_shown_with_no_state_machine_is_shown_in_next_one() {
    let (_file, artboard, vmi) = replacement_scene();
    set_boolean(&outer_vm(&vmi), "details", true);
    artboard.advance_default(0.0);
    let sm = new_machine(&artboard, &vmi);
    let mut tree = LiveTree::default();
    tree.apply(settle_diff(&sm));
    assert_eq!(tree.labels(), with(shown(), &["Outer details"]));
}
fn approx(actual: f32, expected: f32) {
    // Catch Approx's default epsilon is 100*f32::EPSILON, scale is zero.
    assert!(
        (f64::from(actual) - f64::from(expected)).abs()
            <= f64::from(100.0 * f32::EPSILON) * f64::from(expected).abs(),
        "{actual} != Approx({expected})"
    );
}
#[test]
fn nodes_keep_tracking_position_while_ancestor_hidden() {
    let mut reference = CollapseScene::new();
    reference.set_outer("bare", false);
    let mut s = CollapseScene::new();
    let outer = s.data("Outer");
    hidden(&outer, true);
    s.settle();
    s.set_outer("bare", false);
    hidden(&outer, false);
    s.settle();
    for label in ["Middle", "Inner", "Root A", "Deep leaf"] {
        let expected = reference.tree.find(label).unwrap();
        let actual = s.tree.find(label).unwrap();
        approx(actual.min_y, expected.min_y);
        approx(actual.max_y, expected.max_y);
    }
}
#[test]
fn drawable_flagged_hidden_at_runtime_keeps_node() {
    let mut s = CollapseScene::new();
    let data = s.data("Deep leaf");
    let drawable = data
        .with(|d| d.as_component().unwrap().parent_handle())
        .flatten()
        .unwrap();
    let flags = drawable
        .with(|d| d.as_drawable().unwrap().base.drawable_flags())
        .unwrap();
    assert!(CoreRegistry::set_uint_handle(
        &drawable,
        129,
        u32::from(flags | DrawableFlag::HIDDEN.0)
    ));
    assert!(
        drawable
            .with(|d| d.as_drawable().unwrap().is_hidden())
            .unwrap()
    );
    s.mgr
        .with_semantic_manager_mut(|m| m.mark_dirty(SemanticDirt::STRUCTURE));
    s.settle();
    assert_eq!(s.tree.labels(), shown());
    assert!(CoreRegistry::set_uint_handle(&drawable, 129, flags.into()));
    s.mgr
        .with_semantic_manager_mut(|m| m.mark_dirty(SemanticDirt::STRUCTURE));
    s.settle();
    assert_eq!(s.tree.labels(), shown());
}
