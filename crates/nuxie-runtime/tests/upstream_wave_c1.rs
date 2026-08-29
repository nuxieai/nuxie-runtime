//! Native owner-flow ports of the pinned in-band asset, instancing and grid cases.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    assets::image_asset::ImageAsset,
    core::{CoreObject, CoreType},
    file_asset_loader::{FileAssetLoader, FileAssetLoaderRef},
    generated::{core_registry::CoreRegistry, layout::grid_track_base::GridTrackBase},
    layout::{
        grid_item_placement::GridItemPlacement, grid_track::GridTrack,
        layout_component_style::LayoutComponentStyle,
    },
    layout_component::LayoutComponent,
    math::aabb::Aabb,
    shapes::shape::Shape,
};
use nuxie_runtime::{Artboard, CoreHandle, File, RuntimeFactoryHandle, RuntimeFileHandle};
use std::{cell::RefCell, path::PathBuf, rc::Rc};
fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}
fn import(name: &str, loader: Option<FileAssetLoaderRef>) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &pinned_fixture(name),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        loader,
        None,
    )
    .expect("native fixture import")
}
struct Fixture {
    artboard: CoreHandle,
    _file: RuntimeFileHandle,
}
impl Fixture {
    fn load(name: &str) -> Self {
        let file = import(name, None);
        let artboard = file
            .with_file(|file| file.artboard())
            .expect("source Artboard");
        Self {
            artboard,
            _file: file,
        }
    }
    fn advance(&self) {
        Artboard::advance_handle(
            &self.artboard,
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
    }
    fn find<T: CoreType>(&self) -> Vec<CoreHandle> {
        self.artboard
            .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<T>())
            .expect("Artboard")
    }
}
fn style(layout: &CoreHandle) -> Option<CoreHandle> {
    layout
        .with(|object| {
            object
                .as_layout_component()
                .expect("LayoutComponent")
                .style_handle()
        })
        .flatten()
}
fn is_grid_layout(layout: &CoreHandle) -> bool {
    style(layout)
        .and_then(|style| {
            style.with_downcast::<LayoutComponentStyle, _>(|style| style.layout_type_value() == 1)
        })
        .unwrap_or(false)
}
fn spans_two_columns(layout: &CoreHandle) -> bool {
    if style(layout).is_none() {
        return false;
    }
    layout
        .with(|object| GridItemPlacement::from(object.as_container_component()))
        .flatten()
        .and_then(|placement| {
            placement.with_downcast::<GridItemPlacement, _>(|placement| {
                placement.grid_column_span() == 2
            })
        })
        .unwrap_or(false)
}
fn retained_layouts(fixture: &Fixture) -> Vec<(CoreHandle, Aabb)> {
    fixture
        .find::<LayoutComponent>()
        .into_iter()
        .filter(|owner| {
            !owner.is_type_of(<Artboard as CoreType>::TYPE_KEY) && style(owner).is_some()
        })
        .map(|owner| {
            let bounds = owner
                .with(|object| object.as_layout_component().unwrap().layout_bounds())
                .expect("live layout");
            (owner, bounds)
        })
        .collect()
}
fn rect(bounds: Aabb) -> (f32, f32, f32, f32) {
    (bounds.left(), bounds.top(), bounds.width(), bounds.height())
}
type Attempt = (bool, String, String, String, String, usize);
struct ClaimLoader(Rc<RefCell<Option<Attempt>>>);
impl FileAssetLoader for ClaimLoader {
    fn load_contents(
        &mut self,
        asset: CoreHandle,
        bytes: &[u8],
        _factory: &RuntimeFactoryHandle,
    ) -> bool {
        let metadata = asset
            .with(|object| {
                let image = object
                    .as_any()
                    .downcast_ref::<ImageAsset>()
                    .expect("ImageAsset");
                let file_asset = object.as_file_asset().expect("FileAsset").file_asset_base();
                (
                    asset.is_type_of(<ImageAsset as CoreType>::TYPE_KEY),
                    file_asset.cdn_uuid_str(),
                    file_asset.cdn_base_url().to_owned(),
                    file_asset.unique_filename(image.file_extension()),
                    image.file_extension().to_owned(),
                    bytes.len(),
                )
            })
            .expect("live FileAsset");
        *self.0.borrow_mut() = Some(metadata);
        true
    }
}
#[test]
fn wave_c1_in_band_asset_002_loader_claims_responsibility() {
    let attempted = Rc::new(RefCell::new(None));
    let file = import(
        "in_band_asset.riv",
        Some(FileAssetLoaderRef::new(Box::new(ClaimLoader(
            attempted.clone(),
        )))),
    );
    let assets = file.with_file(|file| file.assets().to_vec());
    assert_eq!(assets.len(), 1);
    assert_eq!(
        *attempted.borrow(),
        Some((
            true,
            String::new(),
            "https://public.rive.app/cdn/uuid".into(),
            "1x1-45022.png".into(),
            "png".into(),
            308
        ))
    );
    assets[0]
        .with(|object| {
            let image = object
                .as_any()
                .downcast_ref::<ImageAsset>()
                .expect("ImageAsset");
            let asset = object.as_file_asset().expect("FileAsset").file_asset_base();
            assert_eq!(asset.cdn_uuid_str(), "");
            assert_eq!(asset.cdn_base_url(), "https://public.rive.app/cdn/uuid");
            assert_eq!(
                asset.unique_filename(image.file_extension()),
                "1x1-45022.png"
            );
            assert_eq!(image.file_extension(), "png");
            assert!(
                image.render_image().is_none(),
                "claiming loader prevents default decoding"
            );
        })
        .expect("live image");
}
#[test]
fn wave_c1_instancing_001_cloning_an_ellipse_works() {
    let fixture = Fixture::load("circle_clips.riv");
    let original = fixture
        .artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<Shape>("TopEllipse"))
        .flatten()
        .expect("TopEllipse");
    // The pin clones the Shape itself, not an entire execution facade.
    original
        .with_downcast::<Shape, _>(|shape| {
            let cloned = shape.clone_boxed().expect("Shape clone");
            let cloned = cloned.as_shape().expect("cloned Shape");
            assert_eq!(shape.x(), cloned.x(), "the clone-owned Shape preserves x");
            assert_eq!(shape.y(), cloned.y(), "the clone-owned Shape preserves y");
        })
        .expect("Shape");
}
#[test]
fn wave_c1_layout_grid_001_places_cells_from_riv() {
    let fixture = Fixture::load("layout/grid_2x2.riv");
    assert_eq!(fixture.find::<GridTrack>().len(), 4);
    fixture.advance();
    let layouts = retained_layouts(&fixture);
    let grid = layouts
        .iter()
        .find(|(owner, _)| is_grid_layout(owner))
        .expect("grid layout");
    let wide = layouts
        .iter()
        .find(|(owner, _)| spans_two_columns(owner))
        .expect("two-column layout");
    let mut cells = layouts
        .iter()
        .filter(|(owner, _)| owner != &grid.0 && owner != &wide.0)
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 2);
    cells.sort_by(|a, b| a.1.left().total_cmp(&b.1.left()));
    assert_eq!(rect(cells[0].1), (0.0, 0.0, 100.0, 50.0));
    assert_eq!((cells[1].1.left(), cells[1].1.top()), (100.0, 0.0));
    assert_eq!(rect(wide.1), (0.0, 50.0, 200.0, 50.0));
}
#[test]
fn wave_c1_layout_grid_002_auto_rows_size_overflow_cells() {
    let fixture = Fixture::load("layout/grid_auto_rows.riv");
    fixture.advance();
    let mut cells = retained_layouts(&fixture)
        .into_iter()
        .filter(|(owner, _)| !is_grid_layout(owner))
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 5);
    cells.sort_by(|a, b| {
        a.1.top()
            .total_cmp(&b.1.top())
            .then_with(|| a.1.left().total_cmp(&b.1.left()))
    });
    assert_eq!(
        cells
            .iter()
            .map(|(_, bounds)| rect(*bounds))
            .collect::<Vec<_>>(),
        [
            (0.0, 0.0, 100.0, 50.0),
            (100.0, 0.0, 100.0, 50.0),
            (0.0, 50.0, 100.0, 40.0),
            (100.0, 50.0, 100.0, 40.0),
            (0.0, 90.0, 100.0, 40.0),
        ]
    );
}
#[test]
fn wave_c1_layout_grid_003_track_value_reflows_layout() {
    let fixture = Fixture::load("layout/grid_2x2.riv");
    fixture.advance();
    let tracks = fixture.find::<GridTrack>();
    assert_eq!(tracks.len(), 4);
    let first_column = tracks
        .iter()
        .find(|track| {
            track
                .with_downcast::<GridTrack, _>(|track| track.collection() == 0)
                .unwrap()
        })
        .expect("first template column");
    assert!(CoreRegistry::set_double_handle(
        first_column,
        i32::from(GridTrackBase::TRACK_VALUE_PROPERTY_KEY),
        150.0
    ));
    fixture.advance();
    let (_, wide) = retained_layouts(&fixture)
        .into_iter()
        .find(|(owner, _)| spans_two_columns(owner))
        .expect("two-column layout");
    assert_eq!((wide.width(), wide.top()), (250.0, 50.0));
}
#[test]
fn wave_c1_layout_grid_004_track_types_size_columns() {
    let fixture = Fixture::load("layout/grid_track_types.riv");
    fixture.advance();
    let mut cells = retained_layouts(&fixture)
        .into_iter()
        .filter(|(owner, _)| !is_grid_layout(owner))
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 3);
    cells.sort_by(|a, b| a.1.left().total_cmp(&b.1.left()));
    assert_eq!(
        cells
            .iter()
            .map(|(_, bounds)| bounds.width())
            .collect::<Vec<_>>(),
        [60.0, 50.0, 90.0]
    );
}
