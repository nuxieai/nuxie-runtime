//! All three sections of runtime/grid_item_placement_bind_test.cpp at 3f4047a8.
use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    source::{
        artboard::Artboard,
        core::CoreType,
        layout::grid_item_placement::GridItemPlacement,
        layout_component::LayoutComponent,
        viewmodel::runtime::{
            viewmodel_instance_number_runtime::ViewModelInstanceNumberRuntime,
            viewmodel_instance_runtime::RuntimeViewModelInstanceHandle,
        },
    },
};
use std::path::PathBuf;

struct Fixture {
    artboard: RuntimeArtboardInstanceHandle,
    column: ViewModelInstanceNumberRuntime,
    span: ViewModelInstanceNumberRuntime,
    cell: CoreHandle,
    _instance: RuntimeViewModelInstanceHandle,
    _file: RuntimeFileHandle,
    _factory: PersistentFactory<NullFactory>,
}

fn fixture() -> Fixture {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests/assets/layout/grid_placement_bound.riv");
    let bytes = std::fs::read(&path).expect("pinned grid placement fixture");
    let mut factory = PersistentFactory::new(NullFactory);
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("import grid fixture");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard instance");
    let view_model = file
        .with_file(|file| file.view_model_by_name("vm"))
        .expect("view model vm");
    let instance = view_model.create_default_instance();
    artboard.bind_view_model_instance(Some(instance.instance()));
    let column = instance.property_number("column").expect("number column");
    let span = instance.property_number("span").expect("number span");
    // Match upstream's last explicit-placement layout; exported names are absent.
    let mut cell = None;
    for layout in artboard.with_artboard(|artboard| artboard.find_all_handles::<LayoutComponent>())
    {
        if !layout.is_type_of(Artboard::TYPE_KEY)
            && layout
                .with(|object| {
                    GridItemPlacement::from(Some(
                        object.as_layout_component().expect("LayoutComponent"),
                    ))
                    .is_some()
                })
                .expect("live layout")
        {
            cell = Some(layout);
        }
    }
    Fixture {
        artboard,
        column,
        span,
        cell: cell.expect("explicitly placed cell"),
        _instance: instance,
        _file: file,
        _factory: factory,
    }
}

fn layout(fixture: &Fixture) -> (f32, f32) {
    fixture
        .cell
        .with(|object| {
            let cell = object.as_layout_component().expect("LayoutComponent");
            (cell.layout_x(), cell.layout_width())
        })
        .expect("live cell")
}

#[test]
fn a_bound_span_widens_the_item() {
    let fixture = fixture();
    fixture.span.set_value(2.0);
    fixture.artboard.advance_default(0.0);
    assert_eq!(layout(&fixture).1, 200.0);
}

#[test]
fn a_bound_column_places_the_item() {
    let fixture = fixture();
    fixture.column.set_value(2.0);
    fixture.span.set_value(1.0);
    fixture.artboard.advance_default(0.0);
    assert_eq!(layout(&fixture), (100.0, 100.0));
}

#[test]
fn a_negative_bound_column_counts_back_from_the_last_cell() {
    let fixture = fixture();
    fixture.column.set_value(-1.0);
    fixture.span.set_value(1.0);
    fixture.artboard.advance_default(0.0);
    assert_eq!(layout(&fixture).0, 100.0);
}
