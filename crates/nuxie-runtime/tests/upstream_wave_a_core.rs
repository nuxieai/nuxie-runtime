//! Executable ports for the Wave A cases whose first review found only
//! synthetic or nearby evidence.

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_runtime::{
    ArtboardInstance, Mat2D, RuntimeBindableArtboard, RuntimeOwnedViewModelContext,
    RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance, StateMachineInstance,
};
use std::path::PathBuf;

fn asset_path(name: &str) -> PathBuf {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    root.join("tests/unit_tests/assets").join(name)
}

fn import(name: &str) -> (RuntimeFile, GraphFile) {
    let bytes = std::fs::read(asset_path(name))
        .unwrap_or_else(|error| panic!("read pinned fixture {name}: {error}"));
    let runtime = read_runtime_file(&bytes)
        .unwrap_or_else(|error| panic!("import pinned fixture {name}: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&runtime)
        .unwrap_or_else(|error| panic!("graph pinned fixture {name}: {error:#}"));
    (runtime, graphs)
}

fn close(actual: f32, expected: f32, label: &str) {
    let tolerance = expected.abs().mul_add(1.0e-5, 1.0e-5);
    assert!(
        (actual - expected).abs() <= tolerance,
        "{label}: expected {expected}, got {actual}"
    );
}

fn component_local(graph: &ArtboardGraph, name: &str, type_name: &str) -> usize {
    graph
        .components
        .iter()
        .find(|component| {
            component.name.as_deref() == Some(name)
                && nuxie_schema::definition_by_name(component.type_name).is_some_and(|definition| {
                    definition.name == type_name || definition.ancestors.contains(&type_name)
                })
        })
        .unwrap_or_else(|| panic!("missing {type_name} named {name}"))
        .local_id
}

#[test]
#[ignore = "expected-red: Shape::computeLocalBounds has no callable Rust owner"]
fn upstream_background_shape_bounds_call_the_world_and_local_owners() {
    let (runtime, graphs) = import("background_measure.riv");
    let graph = graphs.artboards.first().expect("background artboard");
    let background = component_local(graph, "background", "Shape");
    assert!(graph.components.iter().any(|component| {
        component.name.as_deref() == Some("nameRun") && component.type_name == "TextValueRun"
    }));
    let mut artboard =
        ArtboardInstance::from_graph_with_artboards(&runtime, graph, &graphs.artboards)
            .expect("background instance");
    artboard.update_pass();

    let initial = artboard
        .object_world_bounds(background)
        .expect("initial Shape::computeWorldBounds analogue");
    close(initial.width(), 42.010925, "initial width");
    close(initial.height(), 29.995453, "initial height");

    assert_eq!(
        artboard.set_root_text_value_run("nameRun", b"much much longer".to_vec()),
        Some(true)
    );
    artboard.update_pass();
    let extended = artboard
        .object_world_bounds(background)
        .expect("extended Shape::computeWorldBounds analogue");
    close(extended.width(), 138.01093, "extended width");
    close(extended.height(), 29.995453, "extended height");

    artboard.debug_update_pass_with_root_transform(Mat2D([0.5, 0.0, 0.0, 0.5, 0.0, 0.0]));
    let scaled = artboard
        .object_world_bounds(background)
        .expect("scaled Shape::computeWorldBounds analogue");
    close(scaled.width(), 138.01093 / 2.0, "scaled width");
    close(scaled.height(), 29.995453 / 2.0, "scaled height");

    panic!(
        "the remaining pinned action is background.computeLocalBounds(); deriving it by dividing world bounds is not parity evidence"
    );
}

#[test]
#[ignore = "expected-red: the generic Component::localBounds matrix has no callable Rust owner"]
fn upstream_local_bounds_executes_the_complete_object_matrix() {
    let (runtime, graphs) = import("local_bounds.riv");
    let graph = graphs.artboards.first().expect("local-bounds artboard");
    let mut artboard =
        ArtboardInstance::from_graph_with_artboards(&runtime, graph, &graphs.artboards)
            .expect("local-bounds instance");
    artboard.update_pass();

    // Preserve every REQUIRE and every concrete owner in the pinned case.
    let matrix = [
        ("Shape1", "Shape"),
        ("Shape2", "Shape"),
        ("Shape3", "Shape"),
        ("Text1", "Text"),
        ("Text2", "Text"),
        ("Group1", "Node"),
        ("Image1", "Image"),
        ("NSlice2", "NSlicedNode"),
        ("CustomShape1", "Shape"),
        ("CustomPath1", "Path"),
        ("LayoutContainer", "LayoutComponent"),
        ("LayoutCellLeft", "LayoutComponent"),
    ];
    for (name, type_name) in matrix {
        let local = component_local(graph, name, type_name);
        // Execute the retained object's real bounds dispatch. This is not
        // accepted as local-bounds proof: transformed world bounds are a
        // distinct owner, which is why the test remains expected-red.
        let _ = artboard.object_world_bounds(local);
    }
    assert!(
        runtime
            .file_assets()
            .iter()
            .any(|asset| asset.type_name.contains("ImageAsset")),
        "Image1 retains a concrete image-asset owner"
    );

    panic!(
        "Rust must expose and execute each concrete localBounds owner before the 48 pinned edge assertions can be translated"
    );
}

#[test]
#[ignore = "expected-red: Artboard children<T>/objects<T> typed iterators are absent in Rust"]
fn upstream_child_typed_iterators_execute_the_iterator_owners() {
    let (runtime, graphs) = import("juice.riv");
    let graph = graphs.artboards.first().expect("default juice artboard");
    let mut artboard =
        ArtboardInstance::from_graph_with_artboards(&runtime, graph, &graphs.artboards)
            .expect("juice instance");
    artboard.update_pass();

    // Do not replace children<Node>, children<ShapePaint>, or objects<ShapePaint>
    // with graph filtering. The missing callable owner is the parity failure.
    panic!(
        "execute children::<Node>(), children::<ShapePaint>(), and objects::<ShapePaint>() here; expected counts are 1, 1, and 20"
    );
}

struct StatefulFixture {
    runtime: RuntimeFile,
    graphs: GraphFile,
    artboard: ArtboardInstance,
    machine: StateMachineInstance,
    context: RuntimeOwnedViewModelContext,
}

impl StatefulFixture {
    fn load(asset: &str, artboard_name: &str) -> Self {
        let (runtime, graphs) = import(asset);
        let (artboard_index, graph) = graphs
            .artboards
            .iter()
            .enumerate()
            .find(|(_, graph)| graph.name.as_deref() == Some(artboard_name))
            .unwrap_or_else(|| panic!("missing artboard {artboard_name} in {asset}"));
        let mut artboard =
            ArtboardInstance::from_graph_with_artboards(&runtime, graph, &graphs.artboards)
                .unwrap_or_else(|error| panic!("instantiate {artboard_name}: {error:#}"));
        let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
        let view_model_index = runtime
            .artboard(artboard_index)
            .and_then(|object| object.uint_property("viewModelId"))
            .and_then(|value| usize::try_from(value).ok())
            .expect("artboard view-model id");
        let main = RuntimeOwnedViewModelInstance::from_instance(&runtime, view_model_index, 0)
            .or_else(|| RuntimeOwnedViewModelInstance::new(&runtime, view_model_index))
            .expect("main view-model instance");
        let mut context = RuntimeOwnedViewModelContext::from_main(main);
        context.complete_for_artboard(&runtime, artboard_index);
        artboard.bind_owned_view_model_artboard_contexts(&runtime, &context);
        assert!(machine.bind_owned_view_model_contexts(&context));
        machine.advance_data_context();
        let mut fixture = Self {
            runtime,
            graphs,
            artboard,
            machine,
            context,
        };
        fixture.frames(1, 0.0);
        fixture
    }

    fn root(&self) -> RuntimeOwnedViewModelHandle {
        self.context
            .main_handle()
            .expect("main view-model handle")
            .clone()
    }

    fn frames(&mut self, count: usize, elapsed: f32) {
        for _ in 0..count {
            StateMachineInstance::advance_and_apply_state_machines_with_view_models(
                &mut self.artboard,
                std::slice::from_mut(&mut self.machine),
                elapsed,
                true,
                || false,
            )
            .expect("stateful frame advances");
        }
    }

    fn source(&self, name: &str) -> (u32, RuntimeBindableArtboard) {
        let graph = self
            .graphs
            .artboards
            .iter()
            .find(|graph| graph.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("missing source artboard {name}"));
        let instance = ArtboardInstance::from_graph_with_artboards(
            &self.runtime,
            graph,
            &self.graphs.artboards,
        )
        .unwrap_or_else(|error| panic!("instantiate source {name}: {error:#}"));
        (
            graph.global_id,
            RuntimeBindableArtboard::new_with_artboard_instance(name, &instance),
        )
    }

    fn nested_graphs(&mut self) -> Vec<u32> {
        let mut ids = Vec::new();
        self.artboard
            .try_visit_artboard_tree_instances_mut(&mut |_, global_id, _| {
                ids.push(global_id);
                Ok::<_, ()>(())
            })
            .expect("tree visit");
        ids
    }
}

#[test]
#[ignore = "expected-red: replacement StrokedButton lacks the pinned owned strokeWidth VMI context"]
fn upstream_stateful_component_dynamic_artboard_swap_replays_the_complete_fixture() {
    let mut fixture = StatefulFixture::load("stateful_artboard_swap.riv", "Main");
    let (button_id, button) = fixture.source("Button");
    let (stroked_id, stroked) = fixture.source("StrokedButton");
    let root = fixture.root();

    fixture.frames(2, 0.016);
    assert!(!fixture.nested_graphs().contains(&button_id));
    assert!(
        root.borrow_mut()
            .set_runtime_artboard_by_property_name("buttonArtboard", Some(button.clone()))
    );
    fixture.frames(5, 0.016);
    assert!(fixture.nested_graphs().contains(&button_id));

    assert!(
        root.borrow_mut()
            .set_runtime_artboard_by_property_name("buttonArtboard", Some(stroked))
    );
    fixture.frames(5, 0.016);
    assert!(fixture.nested_graphs().contains(&stroked_id));
    let mut saw_stroke_width = false;
    fixture
        .artboard
        .try_visit_artboard_tree_instances_mut(&mut |_, global_id, child| {
            if global_id == stroked_id
                && let Some(main) = child
                    .owned_view_model_context()
                    .and_then(RuntimeOwnedViewModelContext::main_handle)
            {
                saw_stroke_width = main
                    .borrow_mut()
                    .set_number_by_property_name("strokeWidth", 8.0);
            }
            Ok::<_, ()>(())
        })
        .expect("stroked child visit");
    assert!(saw_stroke_width);
    fixture.frames(5, 0.016);

    assert!(
        root.borrow_mut()
            .set_runtime_artboard_by_property_name("buttonArtboard", Some(button.clone()))
    );
    fixture.frames(5, 0.016);
    assert!(fixture.nested_graphs().contains(&button_id));
    assert!(
        root.borrow_mut()
            .set_runtime_artboard_by_property_name("buttonArtboard", None)
    );
    fixture.frames(5, 0.016);
    assert!(!fixture.nested_graphs().contains(&button_id));
    assert!(
        root.borrow_mut()
            .set_runtime_artboard_by_property_name("buttonArtboard", Some(button))
    );
    fixture.frames(5, 0.016);
    assert!(fixture.nested_graphs().contains(&button_id));
}

#[test]
#[ignore = "expected-red: active nested VMI pointer identity is not observable through the Rust test surface"]
fn upstream_stateful_nested_source_switch_replays_matching_and_different_vm_lifetimes() {
    let mut fixture = StatefulFixture::load("stateful_source_switch.riv", "ParentArtboard");
    let (matching_id, matching) = fixture.source("MatchingArtboardA");
    let (different_id, different) = fixture.source("DifferentArtboardB");
    let root = fixture.root();
    fixture.frames(5, 0.016);

    assert!(
        root.borrow_mut()
            .set_runtime_artboard_by_property_name("sourceArtboard", Some(matching.clone()))
    );
    fixture.frames(5, 0.016);
    assert!(fixture.nested_graphs().contains(&matching_id));
    assert!(
        root.borrow_mut()
            .set_string_by_property_name("labelInput", b"Matching A")
    );
    fixture.frames(10, 0.016);

    assert!(
        root.borrow_mut()
            .set_runtime_artboard_by_property_name("sourceArtboard", Some(different))
    );
    fixture.frames(5, 0.016);
    assert!(fixture.nested_graphs().contains(&different_id));
    assert!(
        root.borrow_mut()
            .set_string_by_property_name("labelInput", b"Different B")
    );
    fixture.frames(10, 0.016);

    assert!(
        root.borrow_mut()
            .set_runtime_artboard_by_property_name("sourceArtboard", Some(matching))
    );
    fixture.frames(5, 0.016);
    assert!(fixture.nested_graphs().contains(&matching_id));
    assert!(
        root.borrow_mut()
            .set_string_by_property_name("labelInput", b"Matching A Again")
    );
    fixture.frames(10, 0.016);

    panic!(
        "the remaining pinned assertions compare the active nested VMI pointer with the retained stateful-child pointer before and after the round trip"
    );
}

#[test]
#[ignore = "expected-red: stateful list bridge lifecycle does not yet complete on the pinned fixture"]
fn upstream_stateful_component_list_bridge_replays_add_remove_click_readd_and_clear() {
    let mut fixture = StatefulFixture::load("stateful_list_props.riv", "Main");
    let root = fixture.root();
    let button_schema = fixture
        .runtime
        .view_models()
        .iter()
        .position(|candidate| candidate.object.string_property("name") == Some("ButtonVM"))
        .expect("ButtonVM schema");
    let make_button = |label: &[u8], tint: u32| {
        let mut button = RuntimeOwnedViewModelInstance::new(&fixture.runtime, button_schema)
            .expect("ButtonVM instance");
        assert!(button.set_string_by_property_name("label", label));
        assert!(button.set_color_by_property_name("tint", tint));
        RuntimeOwnedViewModelHandle::new(button)
    };
    let alpha = make_button(b"Alpha", 0xffff_3344);
    let beta = make_button(b"Beta", 0xff33_aaff);
    let gamma = make_button(b"Gamma", 0xff44_cc55);
    for button in [&alpha, &beta, &gamma] {
        let index = root
            .list_item_count_by_property_name_path("buttons")
            .expect("buttons list");
        assert!(root.insert_list_item_by_property_name_path("buttons", index, button));
    }
    fixture.frames(3, 0.016);
    assert_eq!(
        root.list_item_count_by_property_name_path("buttons"),
        Some(3)
    );

    assert!(root.remove_list_item_by_property_name_path("buttons", 1));
    fixture.frames(5, 0.016);
    assert_eq!(
        root.list_item_count_by_property_name_path("buttons"),
        Some(2)
    );
    assert_eq!(
        gamma.borrow().boolean_value_by_property_name("clicked"),
        Some(false)
    );
    fixture
        .machine
        .pointer_down(&mut fixture.artboard, 50.0, 73.0, 1);
    fixture
        .machine
        .pointer_up(&mut fixture.artboard, 50.0, 73.0, 1);
    fixture.frames(1, 0.016);
    let gamma_clicked_after_first = gamma.borrow().boolean_value_by_property_name("clicked");
    let removed_beta_clicked_after_first = beta.borrow().boolean_value_by_property_name("clicked");
    fixture.frames(3, 0.016);

    let index = root
        .list_item_count_by_property_name_path("buttons")
        .expect("buttons list");
    assert!(root.insert_list_item_by_property_name_path("buttons", index, &beta));
    fixture.frames(5, 0.016);
    assert_eq!(
        root.list_item_count_by_property_name_path("buttons"),
        Some(3)
    );
    fixture
        .machine
        .pointer_down(&mut fixture.artboard, 50.0, 118.0, 1);
    fixture
        .machine
        .pointer_up(&mut fixture.artboard, 50.0, 118.0, 1);
    fixture.frames(1, 0.016);
    let beta_clicked_after_readd = beta.borrow().boolean_value_by_property_name("clicked");
    fixture.frames(3, 0.016);

    assert!(root.clear_list_items_by_property_name_path("buttons"));
    fixture.frames(5, 0.016);
    assert_eq!(
        root.list_item_count_by_property_name_path("buttons"),
        Some(0)
    );

    // Upstream uses non-fatal CHECK for click propagation, so retain every
    // later action before reporting the first behavioral mismatch.
    assert_eq!(gamma_clicked_after_first, Some(true));
    assert_eq!(removed_beta_clicked_after_first, Some(false));
    assert_eq!(beta_clicked_after_readd, Some(true));
}
