//! Direct ports of all ten cases in pinned
//! `tests/unit_tests/runtime/semantic_artboard_test.cpp`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::GraphFile;
use nuxie_runtime::{
    ArtboardInstance, RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
    SemanticActionType, SemanticBounds, SemanticRole, SemanticState, SemanticTrait, SemanticsDiff,
    StateMachineInstance, has_semantic_state, has_semantic_trait,
};

const DROPDOWN_LABEL: &str = "Select a fandom";
const FANDOM_LABELS: [&str; 4] = [
    "War of the Stars",
    "Scufflestar Galactica",
    "Galaxy Hike",
    "Dino Planet",
];

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

struct Fixture {
    _file: RuntimeFile,
    _graphs: GraphFile,
    artboard: ArtboardInstance,
    state_machine: StateMachineInstance,
    _view_model: Option<RuntimeOwnedViewModelHandle>,
}

fn load_fixture(name: &str) -> Fixture {
    let file = read_runtime_file(&pinned_fixture(name))
        .unwrap_or_else(|error| panic!("{name} imports: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("{name} graph builds: {error:#}"));
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .unwrap_or_else(|error| panic!("{name} default artboard instantiates: {error:#}"));
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    assert!(state_machine.enable_semantics());
    let view_model = file
        .artboard(0)
        .and_then(|artboard| artboard.uint_property("viewModelId"))
        .and_then(|index| usize::try_from(index).ok())
        .and_then(|view_model_index| {
            let instance_index = file
                .view_model_default_instance(view_model_index)?
                .instance_index;
            RuntimeOwnedViewModelInstance::from_instance(&file, view_model_index, instance_index)
                .map(RuntimeOwnedViewModelHandle::new)
        });
    if let Some(view_model) = &view_model {
        state_machine.bind_owned_view_model_handle(view_model);
    }
    for _ in 0..10 {
        state_machine
            .advance_and_apply(&mut artboard, 0.1)
            .expect("semantic fixture settles");
    }
    Fixture {
        _file: file,
        _graphs: graphs,
        artboard,
        state_machine,
        _view_model: view_model,
    }
}

#[derive(Clone)]
struct Entry {
    id: u32,
    role: u32,
    label: String,
    state_flags: u32,
    trait_flags: u32,
    bounds: SemanticBounds,
}

struct Setup {
    fixture: Fixture,
    entries: BTreeMap<u32, Entry>,
}

impl Setup {
    fn load(name: &str) -> Self {
        let mut fixture = load_fixture(name);
        let diff = fixture
            .state_machine
            .drain_semantics_diff(&mut fixture.artboard)
            .expect("initial semantics diff");
        let mut setup = Self {
            fixture,
            entries: BTreeMap::new(),
        };
        setup.apply_diff(diff);
        setup
    }

    fn apply_diff(&mut self, diff: SemanticsDiff) {
        for id in diff.removed {
            self.entries.remove(&id);
        }
        for node in diff.added {
            let bounds = node.bounds();
            self.entries.insert(
                node.id,
                Entry {
                    id: node.id,
                    role: node.role,
                    label: node.label,
                    state_flags: node.state_flags,
                    trait_flags: node.trait_flags,
                    bounds,
                },
            );
        }
        for node in diff.updated_semantic {
            if let Some(entry) = self.entries.get_mut(&node.id) {
                entry.role = node.role;
                entry.label = node.label;
                entry.state_flags = node.state_flags;
                entry.trait_flags = node.trait_flags;
            }
        }
        for geometry in diff.updated_geometry {
            if let Some(entry) = self.entries.get_mut(&geometry.id) {
                entry.bounds = geometry.bounds();
            }
        }
    }

    fn settle(&mut self) {
        assert!(self.fixture.state_machine.semantic_manager());
        for _ in 0..10 {
            self.fixture
                .state_machine
                .advance_and_apply(&mut self.fixture.artboard, 0.1)
                .expect("semantic fixture settles");
        }
        let diff = self
            .fixture
            .state_machine
            .drain_semantics_diff(&mut self.fixture.artboard)
            .expect("semantic diff drains");
        self.apply_diff(diff);
    }

    fn find_by_label(&self, label: &str) -> Option<&Entry> {
        self.entries.values().find(|entry| entry.label == label)
    }

    fn fandom_labels_present(&self) -> BTreeSet<&str> {
        self.entries
            .values()
            .filter(|entry| {
                entry.role == SemanticRole::Text as u32
                    && FANDOM_LABELS.contains(&entry.label.as_str())
            })
            .map(|entry| entry.label.as_str())
            .collect()
    }

    fn tap_dropdown(&mut self) {
        let button = self
            .find_by_label(DROPDOWN_LABEL)
            .expect("dropdown semantic button");
        let x = (button.bounds.min_x + button.bounds.max_x) * 0.5;
        let y = (button.bounds.min_y + button.bounds.max_y) * 0.5;
        self.fixture
            .state_machine
            .pointer_down(&mut self.fixture.artboard, x, y, 0);
        self.fixture
            .state_machine
            .pointer_up(&mut self.fixture.artboard, x, y, 0);
        self.settle();
    }
}

fn fandom_labels() -> BTreeSet<&'static str> {
    FANDOM_LABELS.into_iter().collect()
}

fn missing_manager_node_by_id(_: u32) -> bool {
    panic!("StateMachineInstance exposes semantic diffs but not the C++ manager nodeById owner")
}

#[test]
#[ignore = "expected-red: Rust omits mounted ArtboardComponentList semantic items before nodeById"]
fn artboard_component_list_items_populate_the_parent_artboards_semantic_manager() {
    let setup = Setup::load("semantic/data_binding_lists.riv");
    assert_eq!(setup.fandom_labels_present(), fandom_labels());
    assert!(setup.fixture.state_machine.semantic_manager());
    for label in FANDOM_LABELS {
        let entry = setup.find_by_label(label).expect("fandom semantic entry");
        assert!(missing_manager_node_by_id(entry.id), "{label}");
    }
}

#[test]
#[ignore = "expected-red: Rust does not retain the upstream initial Expanded semantic state"]
fn dropdown_button_starts_expanded_with_the_expandable_trait_authored() {
    let setup = Setup::load("semantic/data_binding_lists.riv");
    let button = setup
        .find_by_label(DROPDOWN_LABEL)
        .expect("dropdown button");
    assert_eq!(button.role, SemanticRole::Button as u32);
    assert!(has_semantic_trait(
        button.trait_flags,
        SemanticTrait::EXPANDABLE
    ));
    assert!(has_semantic_state(
        button.state_flags,
        SemanticState::EXPANDED
    ));
}

#[test]
#[ignore = "expected-red: Rust omits mounted ArtboardComponentList semantic item bounds"]
fn list_item_semantic_data_have_non_empty_bounds_in_the_unified_coordinate_space() {
    let setup = Setup::load("semantic/data_binding_lists.riv");
    for label in FANDOM_LABELS {
        let entry = setup.find_by_label(label).expect("fandom semantic entry");
        assert!(!entry.bounds.is_empty_or_nan(), "{label}");
        assert!(entry.bounds.max_x - entry.bounds.min_x > 0.0, "{label}");
        assert!(entry.bounds.max_y - entry.bounds.min_y > 0.0, "{label}");
    }
}

#[test]
#[ignore = "expected-red: Rust omits mounted ArtboardComponentList semantic items before nodeById"]
fn collapsing_the_dropdown_removes_list_item_semantic_data_from_the_unified_tree() {
    let mut setup = Setup::load("semantic/data_binding_lists.riv");
    assert_eq!(setup.fandom_labels_present(), fandom_labels());
    let ids = FANDOM_LABELS
        .into_iter()
        .map(|label| setup.find_by_label(label).expect("fandom entry").id)
        .collect::<Vec<_>>();
    setup.tap_dropdown();
    assert!(setup.fandom_labels_present().is_empty());
    let button = setup
        .find_by_label(DROPDOWN_LABEL)
        .expect("dropdown button");
    assert!(!has_semantic_state(
        button.state_flags,
        SemanticState::EXPANDED
    ));
    assert!(setup.fixture.state_machine.semantic_manager());
    for id in ids {
        assert!(!missing_manager_node_by_id(id));
    }
}

#[test]
#[ignore = "expected-red: Rust omits mounted ArtboardComponentList semantic items before nodeById"]
fn re_expanding_the_dropdown_re_registers_list_item_semantic_data() {
    let mut setup = Setup::load("semantic/data_binding_lists.riv");
    setup.tap_dropdown();
    assert!(setup.fandom_labels_present().is_empty());
    setup.tap_dropdown();
    assert_eq!(setup.fandom_labels_present(), fandom_labels());
    assert!(setup.fixture.state_machine.semantic_manager());
    for label in FANDOM_LABELS {
        let entry = setup.find_by_label(label).expect("fandom semantic entry");
        assert!(missing_manager_node_by_id(entry.id), "{label}");
    }
}

#[test]
#[ignore = "expected-red: Rust omits mounted ArtboardComponentList semantic items"]
fn multiple_collapse_uncollapse_cycles_maintain_exactly_four_fandoms() {
    let mut setup = Setup::load("semantic/data_binding_lists.riv");
    assert_eq!(setup.fandom_labels_present(), fandom_labels());
    for cycle in 0..3 {
        setup.tap_dropdown();
        assert!(setup.fandom_labels_present().is_empty(), "cycle {cycle}");
        setup.tap_dropdown();
        assert_eq!(
            setup.fandom_labels_present(),
            fandom_labels(),
            "cycle {cycle}"
        );
        let count = setup
            .entries
            .values()
            .filter(|entry| {
                entry.role == SemanticRole::Text as u32
                    && FANDOM_LABELS.contains(&entry.label.as_str())
            })
            .count();
        assert_eq!(count, 4, "cycle {cycle}");
    }
}

#[test]
#[ignore = "expected-red: Rust does not retain the upstream initial Expanded semantic state"]
fn fire_semantic_action_tap_on_the_dropdown_button_collapses_the_list() {
    let mut setup = Setup::load("semantic/data_binding_lists.riv");
    let button = setup
        .find_by_label(DROPDOWN_LABEL)
        .expect("dropdown button");
    assert!(has_semantic_state(
        button.state_flags,
        SemanticState::EXPANDED
    ));
    assert_eq!(setup.fandom_labels_present(), fandom_labels());
    let id = button.id;
    assert!(
        setup
            .fixture
            .state_machine
            .fire_semantic_action(id, SemanticActionType::Tap as u32)
    );
    setup.settle();
    let button = setup
        .find_by_label(DROPDOWN_LABEL)
        .expect("dropdown button");
    assert!(!has_semantic_state(
        button.state_flags,
        SemanticState::EXPANDED
    ));
    assert!(setup.fandom_labels_present().is_empty());
}

#[test]
fn semantic_tap_and_pointer_down_up_converge_on_the_same_state() {
    let mut pointer = Setup::load("semantic/data_binding_lists.riv");
    let mut semantic = Setup::load("semantic/data_binding_lists.riv");
    pointer.tap_dropdown();
    let id = semantic
        .find_by_label(DROPDOWN_LABEL)
        .expect("dropdown button")
        .id;
    assert!(
        semantic
            .fixture
            .state_machine
            .fire_semantic_action(id, SemanticActionType::Tap as u32)
    );
    semantic.settle();
    assert!(pointer.fandom_labels_present().is_empty());
    assert!(semantic.fandom_labels_present().is_empty());
    let pointer_button = pointer
        .find_by_label(DROPDOWN_LABEL)
        .expect("pointer button");
    let semantic_button = semantic
        .find_by_label(DROPDOWN_LABEL)
        .expect("semantic button");
    assert_eq!(
        has_semantic_state(pointer_button.state_flags, SemanticState::EXPANDED),
        has_semantic_state(semantic_button.state_flags, SemanticState::EXPANDED)
    );
}

#[test]
#[ignore = "expected-red: Rust omits dynamic list-item semantic nodes before nodeById/parent"]
fn dynamic_list_item_artboards_parent_list_items_under_the_enclosing_list_role() {
    let mut fixture = load_fixture("semantic/data_binding_lists_items.riv");
    assert!(fixture.state_machine.semantic_manager());
    let diff = fixture
        .state_machine
        .drain_semantics_diff(&mut fixture.artboard)
        .expect("initial semantics diff");
    let list_ids = diff
        .added
        .iter()
        .filter(|n| n.role == SemanticRole::List as u32)
        .map(|n| n.id)
        .collect::<Vec<_>>();
    let item_ids = diff
        .added
        .iter()
        .filter(|n| n.role == SemanticRole::ListItem as u32)
        .map(|n| n.id)
        .collect::<Vec<_>>();
    assert_eq!(list_ids.len(), 1);
    assert!(!item_ids.is_empty());
    let parents = diff
        .added
        .iter()
        .map(|n| (n.id, n.parent_id))
        .collect::<BTreeMap<_, _>>();
    for id in &item_ids {
        assert!(parents.contains_key(id), "id {id}");
        assert_eq!(parents[id], list_ids[0] as i32, "id {id}");
    }
    for id in item_ids {
        assert!(missing_manager_node_by_id(id), "id {id}");
    }
}

#[test]
#[ignore = "expected-red: Rust omits mounted ArtboardComponentList semantic items"]
fn enabling_semantics_twice_does_not_duplicate_list_item_entries() {
    let file = read_runtime_file(&pinned_fixture("semantic/data_binding_lists.riv"))
        .expect("data_binding_lists imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("data_binding_lists graph builds");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    assert!(machine.enable_semantics());
    assert!(!machine.enable_semantics());
    if let Some(vm_index) = file
        .artboard(0)
        .and_then(|a| a.uint_property("viewModelId"))
        .and_then(|i| usize::try_from(i).ok())
        && let Some(instance_index) = file
            .view_model_default_instance(vm_index)
            .map(|i| i.instance_index)
    {
        let vm = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::from_instance(&file, vm_index, instance_index)
                .expect("default view-model instance builds"),
        );
        machine.bind_owned_view_model_handle(&vm);
    }
    for _ in 0..10 {
        machine
            .advance_and_apply(&mut artboard, 0.1)
            .expect("semantic fixture settles");
    }
    let diff = machine
        .drain_semantics_diff(&mut artboard)
        .expect("initial semantics diff");
    let ids = diff
        .added
        .iter()
        .filter(|n| {
            n.role == SemanticRole::Text as u32 && FANDOM_LABELS.contains(&n.label.as_str())
        })
        .map(|n| n.id)
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), FANDOM_LABELS.len());
}
