#![cfg(feature = "scripting")]

use std::{collections::BTreeMap, path::PathBuf};

use nuxie::{
    FileImportLimits, PersistentFactory, ScriptExecutionLimits, ViewModelInstanceRuntime,
    import_unsigned_scripted,
};
use nuxie_render_api::SerializingFactory;
use silver_corpus::{OpKind, Value, parse_sriv};

fn pinned_fixture(relative: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(relative);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned scripted fixture {}: {error}", path.display()))
}

fn pinned_silver(relative: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/silvers")
        .join(relative);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned scripted silver {}: {error}", path.display()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PaintSnapshot {
    style: u64,
    color: u64,
    thickness: i64,
    join: u64,
    cap: u64,
    feather: i64,
    blend_mode: u64,
}

impl Default for PaintSnapshot {
    fn default() -> Self {
        Self {
            style: 1,
            color: 0xff00_0000,
            thickness: quantize(1.0f32.to_bits()),
            join: 0,
            cap: 0,
            feather: quantize(0.0f32.to_bits()),
            blend_mode: 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DrawSnapshot {
    frame: usize,
    transforms: Vec<[i64; 6]>,
    fill_rule: u64,
    path: Vec<CanonicalValue>,
    paint: PaintSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CanonicalValue {
    Uint(u64),
    Float(i64),
    Vec2(i64, i64),
}

fn quantize(bits: u32) -> i64 {
    (f64::from(f32::from_bits(bits)) * 1_000.0).round() as i64
}

fn uint_field(operation: &silver_corpus::Operation, index: usize) -> u64 {
    match operation.fields.get(index).map(|field| &field.value) {
        Some(Value::Uint(value)) => *value,
        value => panic!(
            "expected uint field {index} in {:?}, got {value:?}",
            operation.kind
        ),
    }
}

fn float_field(operation: &silver_corpus::Operation, index: usize) -> i64 {
    match operation.fields.get(index).map(|field| &field.value) {
        Some(Value::Float(value)) => quantize(*value),
        value => panic!(
            "expected float field {index} in {:?}, got {value:?}",
            operation.kind
        ),
    }
}

fn semantic_draws(bytes: &[u8]) -> Vec<DrawSnapshot> {
    let sriv = parse_sriv(bytes).expect("valid SRIV v1 stream");
    let mut paths = BTreeMap::<u64, Vec<CanonicalValue>>::new();
    let mut fill_rules = BTreeMap::<u64, u64>::new();
    let mut paints = BTreeMap::<u64, PaintSnapshot>::new();
    let mut transforms = Vec::<[i64; 6]>::new();
    let mut transform_stack = Vec::<Vec<[i64; 6]>>::new();
    let mut draws = Vec::new();

    for operation in &sriv.operations {
        match operation.kind {
            OpKind::MakeRenderPath => {
                let id = uint_field(operation, 0);
                paths.insert(id, Vec::new());
                fill_rules.insert(id, 0);
            }
            OpKind::MakeRenderPaint => {
                paints.insert(uint_field(operation, 0), PaintSnapshot::default());
            }
            OpKind::Rewind => {
                paths.insert(uint_field(operation, 0), Vec::new());
            }
            OpKind::AddRawPath => {
                let id = uint_field(operation, 0);
                let values = operation
                    .fields
                    .iter()
                    .skip(2)
                    .map(|field| match field.value {
                        Value::Uint(value) => CanonicalValue::Uint(value),
                        Value::Float(value) => CanonicalValue::Float(quantize(value)),
                        Value::Vec2(x, y) => CanonicalValue::Vec2(quantize(x), quantize(y)),
                        Value::Bytes(_) => panic!("raw paths do not contain bytes"),
                    })
                    .collect();
                paths.insert(id, values);
            }
            OpKind::FillRule => {
                fill_rules.insert(uint_field(operation, 0), uint_field(operation, 1));
            }
            OpKind::Style | OpKind::Color | OpKind::Join | OpKind::Cap | OpKind::BlendMode => {
                let paint = paints
                    .get_mut(&uint_field(operation, 0))
                    .expect("paint mutation references a live paint");
                let value = uint_field(operation, 1);
                match operation.kind {
                    OpKind::Style => paint.style = value,
                    OpKind::Color => paint.color = value,
                    OpKind::Join => paint.join = value,
                    OpKind::Cap => paint.cap = value,
                    OpKind::BlendMode => paint.blend_mode = value,
                    _ => unreachable!(),
                }
            }
            OpKind::Thickness | OpKind::Feather => {
                let paint = paints
                    .get_mut(&uint_field(operation, 0))
                    .expect("paint mutation references a live paint");
                let value = float_field(operation, 1);
                if operation.kind == OpKind::Thickness {
                    paint.thickness = value;
                } else {
                    paint.feather = value;
                }
            }
            OpKind::Save => transform_stack.push(transforms.clone()),
            OpKind::Restore => {
                transforms = transform_stack.pop().expect("balanced serializer restore")
            }
            OpKind::Transform => {
                let mut transform = [0; 6];
                for (index, value) in transform.iter_mut().enumerate() {
                    *value = float_field(operation, index);
                }
                transforms.push(transform);
            }
            OpKind::DrawPath => {
                let path = uint_field(operation, 0);
                let paint = uint_field(operation, 1);
                draws.push(DrawSnapshot {
                    frame: operation.frame,
                    transforms: transforms.clone(),
                    fill_rule: *fill_rules.get(&path).unwrap_or(&0),
                    path: paths
                        .get(&path)
                        .expect("draw references a live path")
                        .clone(),
                    paint: paints
                        .get(&paint)
                        .expect("draw references a live paint")
                        .clone(),
                });
            }
            OpKind::Frame | OpKind::FrameSize => {}
            unexpected => panic!(
                "scripted layout differential does not silently discard {unexpected:?} operations"
            ),
        }
    }
    draws
}

#[test]
fn layout_grid_script_reacts_to_rows_and_columns_like_the_upstream_silver_scenario() {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let scripted = import_unsigned_scripted(
        &pinned_fixture("script_layout_test.riv"),
        &mut factory,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("script_layout_test.riv imports with trusted scripts");
    let file = scripted.native_file();
    let artboard = file
        .with_file(|file| file.artboard_named("LayoutScript"))
        .expect("LayoutScript artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    factory.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("default state machine");
    let model_id = artboard.with_artboard(|artboard| artboard.view_model_id());
    let view_model = file
        .with_file(|file| {
            if model_id == u32::MAX {
                file.create_view_model_instance_for_artboard(artboard.core_handle())
            } else {
                file.create_view_model_instance_at(model_id as usize, 0)
            }
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("layout view model");
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(view_model.instance());
    });
    let rows = view_model.property_number("Rows").expect("Rows number");
    assert_eq!(rows.value(), 5.0);
    let columns = view_model
        .property_number("Columns")
        .expect("Columns number");
    assert_eq!(columns.value(), 5.0);
    machine.advance_and_apply(0.1);
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
    for _ in 0..20 {
        factory.borrow_mut().add_frame();
        machine.advance_and_apply(0.016);
        artboard.draw(&mut renderer);
    }
    rows.set_value(8.0);
    assert_eq!(rows.value(), 8.0);
    columns.set_value(7.0);
    assert_eq!(columns.value(), 7.0);
    for _ in 0..20 {
        factory.borrow_mut().add_frame();
        machine.advance_and_apply(0.016);
        artboard.draw(&mut renderer);
    }
    let actual = semantic_draws(&factory.borrow().bytes());
    let expected = semantic_draws(&pinned_silver("script_layout_grid.sriv"));
    assert_eq!(actual.len(), expected.len(), "scripted layout draw count");
    for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
        assert_eq!(
            actual, expected,
            "scripted layout semantic draw {index} differs"
        );
    }
}
