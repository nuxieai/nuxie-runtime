//! Exact behavior ports for pinned runtime Wave C1 files 51-54 and 56.

use std::collections::BTreeMap;
use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, RuntimeObject, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_render_api::{Factory, RecordingFactory};
use nuxie_runtime::{ArtboardInstance, RuntimeFileAssetKind, RuntimeFileAssetOwners};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn load_graph(name: &str) -> (RuntimeFile, GraphFile) {
    let file = read_runtime_file(&pinned_fixture(name))
        .unwrap_or_else(|error| panic!("{name} imports: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("{name} graph builds: {error:#}"));
    (file, graphs)
}

fn graph_and_instance(
    name: &str,
    artboard_name: Option<&str>,
) -> (RuntimeFile, GraphFile, usize, ArtboardInstance) {
    let (file, graphs) = load_graph(name);
    let graph_index = graphs
        .artboards
        .iter()
        .position(|graph| artboard_name.is_none_or(|name| graph.name.as_deref() == Some(name)))
        .unwrap_or_else(|| panic!("missing requested artboard in {name}"));
    let instance = ArtboardInstance::from_graph_with_artboards(
        &file,
        &graphs.artboards[graph_index],
        &graphs.artboards,
    )
    .unwrap_or_else(|error| panic!("{name} artboard instantiates: {error:#}"));
    (file, graphs, graph_index, instance)
}

fn named_local(graph: &ArtboardGraph, name: &str, type_name: &str) -> usize {
    graph
        .local_objects
        .iter()
        .find(|object| object.name.as_deref() == Some(name) && object.type_name == Some(type_name))
        .unwrap_or_else(|| panic!("missing {type_name} named {name}"))
        .local_id
}

fn object_at_local<'a>(
    file: &'a RuntimeFile,
    graph: &ArtboardGraph,
    local_id: usize,
) -> &'a RuntimeObject {
    let global_id = graph
        .local_objects
        .get(local_id)
        .unwrap_or_else(|| panic!("missing local object {local_id}"))
        .global_id;
    file.object(global_id as usize)
        .unwrap_or_else(|| panic!("missing global object {global_id}"))
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("schema definition");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("ancestor definition")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("property {type_name}.{property_name}"))
        .key
        .int
}

fn layout_style_local(file: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Option<usize> {
    usize::try_from(object_at_local(file, graph, local_id).uint_property("styleId")?).ok()
}

fn is_grid_layout(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    local_id: usize,
) -> bool {
    let Some(style_local) = layout_style_local(file, graph, local_id) else {
        return false;
    };
    instance.debug_uint_property(
        style_local,
        property_key("LayoutComponentStyle", "layoutTypeValue"),
    ) == Some(1)
}

fn spans_two_columns(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    layout_local: usize,
) -> bool {
    graph.components.iter().any(|component| {
        component.type_name == "GridItemPlacement"
            && component.parent_local == Some(layout_local)
            && instance.debug_uint_property(
                component.local_id,
                property_key("GridItemPlacement", "gridColumnSpan"),
            ) == Some(2)
            && object_at_local(file, graph, component.local_id).type_name == "GridItemPlacement"
    })
}

#[test]
fn wave_c1_image_mesh_001_image_with_mesh_loads_correctly() {
    let (file, graphs, graph_index, instance) = graph_and_instance("tape.riv", None);
    let graph = &graphs.artboards[graph_index];
    let image_local = named_local(graph, "Tape body.png", "Image");
    let image = object_at_local(&file, graph, image_local);
    let image_asset = file
        .resolved_file_asset_for_referencer(image)
        .expect("Tape body.png resolves its ImageAsset");
    let encoded = file
        .imported_file_asset_contents(image_asset.id)
        .expect("Tape body.png has in-band bytes");
    let mesh = graph.meshes.first().expect("Tape body.png owns a Mesh");
    let indices = file
        .object(mesh.global_id as usize)
        .and_then(RuntimeObject::mesh_triangle_indices)
        .expect("Tape body mesh indices");

    assert_eq!(encoded.len(), 70_903);
    assert_eq!(mesh.vertices.len(), 24);
    assert_eq!(indices.len(), 31 * 3);
    assert_eq!(graph.local_objects[image_local].type_name, Some("Image"),);
    drop(instance);
}

#[test]
fn wave_c1_image_mesh_002_duplicating_a_mesh_shares_the_indices() {
    let (file, graphs) = load_graph("tape.riv");
    let graph = graphs.artboards.first().expect("tape artboard");
    let image_local = named_local(graph, "Tape body.png", "Image");
    let mesh = graph.meshes.first().expect("Tape body.png owns a Mesh");
    let index_bytes = file
        .object(mesh.global_id as usize)
        .and_then(|mesh| mesh.bytes_property("triangleIndexBytes"))
        .expect("shared encoded index owner");
    let instances = (0..3)
        .map(|_| {
            ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
                .expect("tape artboard instantiates")
        })
        .collect::<Vec<_>>();

    for mut instance in instances {
        assert_eq!(
            file.object(mesh.global_id as usize)
                .and_then(RuntimeObject::mesh_triangle_indices)
                .expect("mesh indices")
                .len(),
            31 * 3,
        );
        assert!(std::ptr::eq(
            index_bytes,
            file.object(mesh.global_id as usize)
                .and_then(|mesh| mesh.bytes_property("triangleIndexBytes"))
                .expect("same encoded index owner"),
        ));
        instance.update_pass();
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        instance
            .draw_artboard(
                &file,
                graph,
                &graphs.artboards,
                &mut factory,
                &mut renderer,
                &BTreeMap::new(),
                None,
                true,
            )
            .expect("each instance draws through the shared mesh definition");
        assert_eq!(graph.local_objects[image_local].type_name, Some("Image"));
    }
}

fn assert_in_band_asset_metadata(asset: &RuntimeObject) {
    assert_eq!(asset.type_name, "ImageAsset");
    assert_eq!(asset.file_asset_cdn_uuid_string().as_deref(), Some(""));
    assert_eq!(
        asset.string_property("cdnBaseUrl"),
        Some("https://public.rive.app/cdn/uuid"),
    );
    assert_eq!(
        asset.file_asset_unique_filename().as_deref(),
        Some("1x1-45022.png"),
    );
    assert_eq!(asset.file_asset_extension(), Some("png"));
}

#[test]
fn wave_c1_in_band_asset_001_load_asset_with_in_band_image() {
    let file = read_runtime_file(&pinned_fixture("in_band_asset.riv")).expect("fixture imports");
    let assets = file.file_assets();
    assert_eq!(assets.len(), 1);
    assert_in_band_asset_metadata(assets[0]);
    let contents = file
        .imported_file_asset_contents(assets[0].id)
        .expect("in-band image contents");
    assert_eq!(contents.len(), 308);

    let mut factory = RecordingFactory::new();
    let mut loader =
        |_asset: &nuxie_runtime::RuntimeFileAsset, _bytes: &[u8], _factory: &mut dyn Factory| false;
    let owners = RuntimeFileAssetOwners::import_with_loader(&file, None, &mut factory, &mut loader);
    assert!(owners.image_assets().get(assets[0].id).is_some());
}

#[test]
fn wave_c1_in_band_asset_002_loader_claims_responsibility() {
    let file = read_runtime_file(&pinned_fixture("in_band_asset.riv")).expect("fixture imports");
    let assets = file.file_assets();
    assert_eq!(assets.len(), 1);
    let mut attempted = None;
    let mut loader =
        |asset: &nuxie_runtime::RuntimeFileAsset, bytes: &[u8], _factory: &mut dyn Factory| {
            attempted = Some((
                asset.kind(),
                asset.descriptor().file_asset_cdn_uuid_string(),
                asset
                    .descriptor()
                    .string_property("cdnBaseUrl")
                    .map(str::to_owned),
                asset.descriptor().file_asset_unique_filename(),
                asset.descriptor().file_asset_extension().map(str::to_owned),
                bytes.len(),
            ));
            true
        };
    let mut factory = RecordingFactory::new();
    let owners = RuntimeFileAssetOwners::import_with_loader(&file, None, &mut factory, &mut loader);

    assert_in_band_asset_metadata(assets[0]);
    assert_eq!(
        attempted,
        Some((
            RuntimeFileAssetKind::Image,
            Some(String::new()),
            Some("https://public.rive.app/cdn/uuid".to_owned()),
            Some("1x1-45022.png".to_owned()),
            Some("png".to_owned()),
            308,
        )),
    );
    assert!(owners.image_assets().get(assets[0].id).is_none());
}

#[test]
fn wave_c1_in_band_asset_003_loader_rejection_uses_in_band_fallback() {
    let file = read_runtime_file(&pinned_fixture("in_band_asset.riv")).expect("fixture imports");
    let asset = file.file_assets()[0];
    let mut attempted_bytes = 0;
    let mut loader =
        |_asset: &nuxie_runtime::RuntimeFileAsset, bytes: &[u8], _factory: &mut dyn Factory| {
            attempted_bytes = bytes.len();
            false
        };
    let mut factory = RecordingFactory::new();
    let owners = RuntimeFileAssetOwners::import_with_loader(&file, None, &mut factory, &mut loader);

    assert_eq!(attempted_bytes, 308);
    assert!(owners.image_assets().get(asset.id).is_some());
}

#[test]
fn wave_c1_instancing_001_cloning_an_ellipse_works() {
    let (file, graphs, graph_index, artboard) = graph_and_instance("circle_clips.riv", None);
    let graph = &graphs.artboards[graph_index];
    let node = named_local(graph, "TopEllipse", "Shape");
    let cloned = artboard.clone();
    for property in ["x", "y"] {
        let key = property_key("Shape", property);
        assert_eq!(
            artboard.double_property(node, key),
            cloned.double_property(node, key),
            "the clone-owned Shape preserves {property}",
        );
    }
    drop(file);
}

#[test]
fn wave_c1_instancing_002_artboard_clone_preserves_clipping_properties() {
    let (file, graphs, graph_index, mut artboard) = graph_and_instance("circle_clips.riv", None);
    let graph = &graphs.artboards[graph_index];
    let node = named_local(graph, "TopEllipse", "Shape");
    let clipping_sources = graph
        .clipping_shapes
        .iter()
        .filter(|clipping| clipping.clipped_drawable_locals.contains(&node))
        .map(|clipping| {
            graph.local_objects[clipping.source_local.expect("clipping source")]
                .name
                .as_deref()
                .expect("clipping source name")
        })
        .collect::<Vec<_>>();
    assert_eq!(clipping_sources, ["ClipRect2", "BabyEllipse"]);

    artboard.update_pass();
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    artboard
        .draw_artboard(
            &file,
            graph,
            &graphs.artboards,
            &mut factory,
            &mut renderer,
            &BTreeMap::new(),
            None,
            true,
        )
        .expect("clipped artboard draws");
}

#[test]
fn wave_c1_instancing_003_artboard_instances_share_animation_definitions() {
    let (file, graphs) = load_graph("juice.riv");
    let graph = graphs.artboards.first().expect("juice artboard");
    let animation_count = graph.animations.len();
    assert!(animation_count > 0);
    let first_animation = &graph.animations[0] as *const _;
    let artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("juice artboard instantiates");

    assert_eq!(graph.animations.len(), animation_count);
    assert_eq!(&graph.animations[0] as *const _, first_animation);
    drop(artboard);
    assert_eq!(graph.animations.len(), animation_count);
    assert_eq!(&graph.animations[0] as *const _, first_animation);
}

#[test]
fn wave_c1_joystick_flags_001_flags_and_inverted_axes_match() {
    const INVERT_X: u64 = 1 << 0;
    const INVERT_Y: u64 = 1 << 1;
    const WORLD_SPACE: u64 = 1 << 2;

    let (_file, graphs, graph_index, mut artboard) =
        graph_and_instance("joystick_flag_test.riv", None);
    let graph = &graphs.artboards[graph_index];
    let invert_x = named_local(graph, "Invert X Joystick", "Joystick");
    let invert_y = named_local(graph, "Invert Y Joystick", "Joystick");
    let world = named_local(graph, "World Joystick", "Joystick");
    let normal = named_local(graph, "Normal Joystick", "Joystick");
    assert_eq!(artboard.debug_joystick_flags(invert_x), Some(INVERT_X));
    assert_eq!(artboard.debug_joystick_flags(invert_y), Some(INVERT_Y));
    assert_eq!(artboard.debug_joystick_flags(world), Some(WORLD_SPACE));
    assert_eq!(artboard.debug_joystick_flags(normal), Some(0));

    for (value, expected_x) in [(0.0, 350.0), (1.0, 300.0), (-1.0, 400.0)] {
        artboard.set_double_property(invert_x, property_key("Joystick", "x"), value);
        artboard.update_pass();
        assert_eq!(
            artboard.double_property(
                named_local(graph, "invert_x_rect", "Shape"),
                property_key("Shape", "x"),
            ),
            Some(expected_x),
        );
    }
    for (value, expected_x) in [(0.0, 425.0), (1.0, 400.0), (-1.0, 450.0)] {
        artboard.set_double_property(invert_y, property_key("Joystick", "y"), value);
        artboard.update_pass();
        assert_eq!(
            artboard.double_property(
                named_local(graph, "invert_y_ellipse", "Shape"),
                property_key("Shape", "x"),
            ),
            Some(expected_x),
        );
    }
}

fn settled_layout(
    fixture: &str,
) -> (
    RuntimeFile,
    GraphFile,
    usize,
    ArtboardInstance,
    Vec<nuxie_runtime::RuntimeLayoutBoundsReport>,
) {
    let (file, graphs, graph_index, mut artboard) = graph_and_instance(fixture, None);
    artboard.update_pass();
    let report = artboard
        .debug_taffy_layout_bounds_report(&file, &graphs.artboards[graph_index])
        .expect("layout report");
    (file, graphs, graph_index, artboard, report)
}

#[test]
fn wave_c1_layout_grid_001_places_cells_from_riv() {
    let (file, graphs, graph_index, artboard, report) = settled_layout("layout/grid_2x2.riv");
    let graph = &graphs.artboards[graph_index];
    assert_eq!(
        graph
            .components
            .iter()
            .filter(|c| c.type_name == "GridTrack")
            .count(),
        4,
    );
    let layouts = report
        .iter()
        .filter(|entry| entry.type_name == "LayoutComponent")
        .collect::<Vec<_>>();
    let grid = layouts
        .iter()
        .find(|entry| is_grid_layout(&file, graph, &artboard, entry.local_id))
        .expect("grid layout");
    let wide = layouts
        .iter()
        .find(|entry| spans_two_columns(&file, graph, &artboard, entry.local_id))
        .expect("two-column layout");
    let mut cells = layouts
        .iter()
        .filter(|entry| entry.local_id != grid.local_id && entry.local_id != wide.local_id)
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 2);
    cells.sort_by(|a, b| a.x.total_cmp(&b.x));
    assert_eq!(
        (cells[0].x, cells[0].y, cells[0].width, cells[0].height),
        (0.0, 0.0, 100.0, 50.0)
    );
    assert_eq!((cells[1].x, cells[1].y), (100.0, 0.0));
    assert_eq!(
        (wide.x, wide.y, wide.width, wide.height),
        (0.0, 50.0, 200.0, 50.0)
    );
}

#[test]
fn wave_c1_layout_grid_002_auto_rows_size_overflow_cells() {
    let (file, graphs, graph_index, artboard, report) = settled_layout("layout/grid_auto_rows.riv");
    let graph = &graphs.artboards[graph_index];
    let mut cells = report
        .iter()
        .filter(|entry| {
            entry.type_name == "LayoutComponent"
                && !is_grid_layout(&file, graph, &artboard, entry.local_id)
        })
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 5);
    cells.sort_by(|a, b| a.y.total_cmp(&b.y).then_with(|| a.x.total_cmp(&b.x)));
    assert_eq!(
        cells
            .iter()
            .map(|cell| (cell.x, cell.y, cell.width, cell.height))
            .collect::<Vec<_>>(),
        [
            (0.0, 0.0, 100.0, 50.0),
            (100.0, 0.0, 100.0, 50.0),
            (0.0, 50.0, 100.0, 40.0),
            (100.0, 50.0, 100.0, 40.0),
            (0.0, 90.0, 100.0, 40.0),
        ],
    );
}

#[test]
fn wave_c1_layout_grid_003_track_value_reflows_layout() {
    let (file, graphs, graph_index, mut artboard) = graph_and_instance("layout/grid_2x2.riv", None);
    let graph = &graphs.artboards[graph_index];
    artboard.update_pass();
    let tracks = graph
        .components
        .iter()
        .filter(|c| c.type_name == "GridTrack")
        .collect::<Vec<_>>();
    assert_eq!(tracks.len(), 4);
    let first_column = tracks
        .iter()
        .find(|track| {
            artboard.debug_uint_property(track.local_id, property_key("GridTrack", "collection"))
                == Some(0)
        })
        .expect("first template column");
    assert!(artboard.set_double_property(
        first_column.local_id,
        property_key("GridTrack", "trackValue"),
        150.0,
    ));
    artboard.update_pass();
    let report = artboard
        .debug_taffy_layout_bounds_report(&file, graph)
        .expect("layout report");
    let wide = report
        .iter()
        .find(|entry| {
            entry.type_name == "LayoutComponent"
                && spans_two_columns(&file, graph, &artboard, entry.local_id)
        })
        .expect("two-column layout");
    assert_eq!((wide.width, wide.y), (250.0, 50.0));
}

#[test]
fn wave_c1_layout_grid_004_track_types_size_columns() {
    let (file, graphs, graph_index, artboard, report) =
        settled_layout("layout/grid_track_types.riv");
    let graph = &graphs.artboards[graph_index];
    let mut cells = report
        .iter()
        .filter(|entry| {
            entry.type_name == "LayoutComponent"
                && !is_grid_layout(&file, graph, &artboard, entry.local_id)
        })
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 3);
    cells.sort_by(|a, b| a.x.total_cmp(&b.x));
    assert_eq!(
        cells.iter().map(|cell| cell.width).collect::<Vec<_>>(),
        [60.0, 50.0, 90.0]
    );
}

#[test]
#[ignore = "expected-red: settled runtime layout exposes no retained grid-line query owner"]
fn wave_c1_layout_grid_005_line_offsets_are_exposed_after_layout() {
    let (file, graphs, graph_index, artboard, report) = settled_layout("layout/grid_2x2.riv");
    let graph = &graphs.artboards[graph_index];
    let grid = report
        .iter()
        .find(|entry| {
            entry.type_name == "LayoutComponent"
                && is_grid_layout(&file, graph, &artboard, entry.local_id)
        })
        .expect("grid layout");
    let grid_line_entries = report
        .iter()
        .filter(|entry| entry.parent_local == Some(grid.local_id) && entry.type_name == "GridLine")
        .collect::<Vec<_>>();

    assert_eq!(
        grid_line_entries
            .iter()
            .filter(|entry| entry.name.as_deref() == Some("column"))
            .map(|entry| entry.x)
            .collect::<Vec<_>>(),
        [0.0, 100.0, 200.0],
        "the settled grid must expose retained column-line offsets",
    );
    assert_eq!(
        grid_line_entries
            .iter()
            .filter(|entry| entry.name.as_deref() == Some("row"))
            .map(|entry| entry.y)
            .collect::<Vec<_>>(),
        [0.0, 50.0, 100.0],
        "the settled grid must expose retained row-line offsets",
    );
    assert!(
        report
            .iter()
            .filter(|entry| entry.type_name == "Artboard")
            .all(|entry| entry.parent_local.is_none()),
        "the non-grid artboard owner exposes no grid lines",
    );
}
