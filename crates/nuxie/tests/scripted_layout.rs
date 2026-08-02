#![cfg(feature = "scripting")]

use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use nuxie::{File, OwnedArtboardInstance, PersistentFactory};
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
    let file = Arc::new(
        File::import_with_unsigned_scripts(&pinned_fixture("script_layout_test.riv"))
            .expect("script_layout_test.riv imports with trusted scripts"),
    );
    let artboard_index = file
        .artboard_named("LayoutScript")
        .expect("LayoutScript artboard")
        .index();
    let mut instance =
        OwnedArtboardInstance::instantiate(Arc::clone(&file), artboard_index).expect("instance");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("default state machine");
    let mut view_model = instance
        .instantiate_default_view_model_instance()
        .or_else(|| instance.instantiate_view_model())
        .expect("layout view model");
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let mut renderer = factory.borrow().make_renderer();

    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.1,
            &mut view_model,
            &mut factory,
        )
        .expect("initialize layout script");
    factory.borrow_mut().frame_size(500, 500);
    instance
        .draw(&mut factory, &mut renderer)
        .expect("draw initialized grid");
    for _ in 0..20 {
        factory.borrow_mut().add_frame();
        instance
            .try_advance_with_state_machines_and_view_model_and_factory(
                std::slice::from_mut(&mut machine),
                0.016,
                &mut view_model,
                &mut factory,
            )
            .expect("advance initial grid");
        instance
            .draw(&mut factory, &mut renderer)
            .expect("draw initial grid frame");
    }
    assert!(view_model.set_number("Rows", 8.0));
    assert!(view_model.set_number("Columns", 7.0));
    for _ in 0..20 {
        factory.borrow_mut().add_frame();
        instance
            .try_advance_with_state_machines_and_view_model_and_factory(
                std::slice::from_mut(&mut machine),
                0.016,
                &mut view_model,
                &mut factory,
            )
            .expect("advance resized grid");
        instance
            .draw(&mut factory, &mut renderer)
            .expect("draw resized grid frame");
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
