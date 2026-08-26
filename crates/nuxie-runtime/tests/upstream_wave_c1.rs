//! Exact behavior ports for pinned runtime Wave C1 files 51-54 and 56.

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

fn retained_layouts(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
) -> Vec<(usize, nuxie_runtime::RuntimeLayoutBounds)> {
    graph
        .components
        .iter()
        .filter(|component| {
            component.type_name != "Artboard"
                && layout_style_local(file, graph, component.local_id).is_some()
                && (component.type_name == "LayoutComponent"
                    || nuxie_schema::definition_by_name(component.type_name).is_some_and(
                        |definition| definition.ancestors.contains(&"LayoutComponent"),
                    ))
        })
        .filter_map(|component| {
            instance
                .layout_bounds(component.local_id)
                .map(|bounds| (component.local_id, bounds))
        })
        .collect()
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

fn settled_layout(fixture: &str) -> (RuntimeFile, GraphFile, usize, ArtboardInstance) {
    let (file, graphs, graph_index, mut artboard) = graph_and_instance(fixture, None);
    artboard.advance(0.0).expect("layout advance");
    (file, graphs, graph_index, artboard)
}

#[test]
fn wave_c1_layout_grid_001_places_cells_from_riv() {
    let (file, graphs, graph_index, artboard) = settled_layout("layout/grid_2x2.riv");
    let graph = &graphs.artboards[graph_index];
    assert_eq!(
        graph
            .components
            .iter()
            .filter(|c| c.type_name == "GridTrack")
            .count(),
        4,
    );
    let layouts = retained_layouts(&file, graph, &artboard);
    let grid = layouts
        .iter()
        .find(|(local_id, _)| is_grid_layout(&file, graph, &artboard, *local_id))
        .expect("grid layout");
    let wide = layouts
        .iter()
        .find(|(local_id, _)| spans_two_columns(&file, graph, &artboard, *local_id))
        .expect("two-column layout");
    let mut cells = layouts
        .iter()
        .filter(|(local_id, _)| *local_id != grid.0 && *local_id != wide.0)
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 2);
    cells.sort_by(|a, b| a.1.x.total_cmp(&b.1.x));
    assert_eq!(
        (
            cells[0].1.x,
            cells[0].1.y,
            cells[0].1.width,
            cells[0].1.height,
        ),
        (0.0, 0.0, 100.0, 50.0)
    );
    assert_eq!((cells[1].1.x, cells[1].1.y), (100.0, 0.0));
    assert_eq!(
        (wide.1.x, wide.1.y, wide.1.width, wide.1.height),
        (0.0, 50.0, 200.0, 50.0)
    );
}

#[test]
fn wave_c1_layout_grid_002_auto_rows_size_overflow_cells() {
    let (file, graphs, graph_index, artboard) = settled_layout("layout/grid_auto_rows.riv");
    let graph = &graphs.artboards[graph_index];
    let mut cells = retained_layouts(&file, graph, &artboard)
        .into_iter()
        .filter(|(local_id, _)| !is_grid_layout(&file, graph, &artboard, *local_id))
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 5);
    cells.sort_by(|a, b| {
        a.1.y
            .total_cmp(&b.1.y)
            .then_with(|| a.1.x.total_cmp(&b.1.x))
    });
    assert_eq!(
        cells
            .iter()
            .map(|(_, cell)| (cell.x, cell.y, cell.width, cell.height))
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
    artboard.advance(0.0).expect("initial layout advance");
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
    artboard.advance(0.0).expect("reflow layout advance");
    let wide = retained_layouts(&file, graph, &artboard)
        .into_iter()
        .find(|(local_id, _)| spans_two_columns(&file, graph, &artboard, *local_id))
        .expect("two-column layout");
    assert_eq!((wide.1.width, wide.1.y), (250.0, 50.0));
}

#[test]
fn wave_c1_layout_grid_004_track_types_size_columns() {
    let (file, graphs, graph_index, artboard) = settled_layout("layout/grid_track_types.riv");
    let graph = &graphs.artboards[graph_index];
    let mut cells = retained_layouts(&file, graph, &artboard)
        .into_iter()
        .filter(|(local_id, _)| !is_grid_layout(&file, graph, &artboard, *local_id))
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 3);
    cells.sort_by(|a, b| a.1.x.total_cmp(&b.1.x));
    assert_eq!(
        cells.iter().map(|(_, cell)| cell.width).collect::<Vec<_>>(),
        [60.0, 50.0, 90.0]
    );
}
