use anyhow::{Context, Result};
use nuxie::{
    ArtboardSpec, FillSpec, LayoutComponentSpec, LayoutComponentStyleSpec, NodeSpec, ObjectId,
    Parent, RecordingFactory, RectangleSpec, Scene, ShapeSpec, SolidColorSpec, props,
};
use nuxie_render_stream::{Command as RenderCommand, RenderStream};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const SCALE_X_KEY: u16 = 16;
const TRANSLATE_X_KEY: u16 = 13;
const TRANSLATE_Y_KEY: u16 = 14;
const ROTATION_KEY: u16 = 15;
const SCALE_Y_KEY: u16 = 17;
const OPACITY_KEY: u16 = 18;

#[derive(Clone, Copy)]
struct LiveWrite {
    cursor: nuxie::Cursor<f32>,
    local_id: usize,
    property_key: u16,
    value: f32,
    observed: ObjectId,
    observed_local_id: usize,
}

struct RustSnapshot {
    property_value: f32,
    world_transform: [f32; 6],
}

struct TransformFixtureObjects {
    artboard: nuxie::ArtboardId,
    parent: ObjectId,
    child: ObjectId,
    layout_descendant: ObjectId,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cpp_probe_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("RIVE_CPP_PROBE") {
        let path = PathBuf::from(path);
        return Some(if path.is_absolute() {
            path
        } else {
            repo_root().join(path)
        });
    }

    let os = match std::env::consts::OS {
        "macos" => "macosx",
        other => other,
    };
    let path = repo_root()
        .join("tools/cpp-probe/build")
        .join(os)
        .join("bin/debug/rive_cpp_probe");
    path.exists().then_some(path)
}

fn transform_fixture_path() -> PathBuf {
    repo_root().join("fixtures/univ-1275/transform_live_write.riv")
}

fn cpp_live_writes(probe: &Path, writes: &[LiveWrite]) -> Result<Value> {
    let mut command = Command::new(probe);
    command.args(["--file"]).arg(transform_fixture_path());
    for write in writes {
        command.arg("--runtime-settle-double");
        command.arg(write.local_id.to_string());
        command.arg(write.property_key.to_string());
        command.arg(write.value.to_string());
        command.arg(write.observed_local_id.to_string());
    }
    let output = command
        .output()
        .context("run the C++ transform live-write probe")?;
    anyhow::ensure!(
        output.status.success(),
        "C++ transform live-write probe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).context("parse C++ transform live-write report")
}

fn create_transform_scene() -> Result<(Scene, TransformFixtureObjects)> {
    let mut scene = Scene::new();
    let ((artboard, parent, child, layout_descendant), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "UNIV-1275 transform live writes".into(),
            width: 320.0,
            height: 240.0,
        })?;
        let parent = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Transform parent".into(),
                x: 20.0,
                y: 30.0,
                opacity: 0.8,
                rotation: 0.25,
                scale_x: 1.5,
                scale_y: 0.75,
            }),
        )?;
        let child = tx.create(
            Parent::Object(parent),
            NodeSpec::Shape(ShapeSpec {
                name: "Transform child".into(),
                x: 5.0,
                y: 7.0,
                opacity: 0.5,
                rotation: -0.1,
                scale_x: 1.2,
                scale_y: 0.8,
            }),
        )?;
        tx.create(
            Parent::Object(child),
            NodeSpec::Rectangle(RectangleSpec::new("Transform rectangle", 20.0, 10.0)),
        )?;
        let fill = tx.create(
            Parent::Object(child),
            NodeSpec::Fill(FillSpec {
                name: "Transform fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Transform color".into(),
                color: 0xff33_6699,
            }),
        )?;
        let layout_root = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::LayoutComponent(LayoutComponentSpec {
                name: "Layout root".into(),
                x: 40.0,
                y: 25.0,
                opacity: 1.0,
                rotation: 0.15,
                scale_x: 1.1,
                scale_y: 0.9,
                clip: false,
                width: 160.0,
                height: 100.0,
                fractional_width: 1.0,
                fractional_height: 1.0,
                style: LayoutComponentStyleSpec::default(),
            }),
        )?;
        let layout_nested = tx.create(
            Parent::Object(layout_root),
            NodeSpec::LayoutComponent(LayoutComponentSpec {
                name: "Layout nested".into(),
                x: 12.0,
                y: 8.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                clip: false,
                width: 80.0,
                height: 60.0,
                fractional_width: 1.0,
                fractional_height: 1.0,
                style: LayoutComponentStyleSpec::default(),
            }),
        )?;
        let layout_descendant = tx.create(
            Parent::Object(layout_nested),
            NodeSpec::Shape(ShapeSpec {
                name: "Layout descendant".into(),
                x: 3.0,
                y: 4.0,
                opacity: 1.0,
                rotation: -0.2,
                scale_x: 0.9,
                scale_y: 1.3,
            }),
        )?;
        tx.create(
            Parent::Object(layout_descendant),
            NodeSpec::Rectangle(RectangleSpec::new("Layout rectangle", 12.0, 8.0)),
        )?;
        let layout_fill = tx.create(
            Parent::Object(layout_descendant),
            NodeSpec::Fill(FillSpec {
                name: "Layout fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(layout_fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Layout color".into(),
                color: 0xff99_6633,
            }),
        )?;
        Ok((artboard, parent, child, layout_descendant))
    })?;
    Ok((
        scene,
        TransformFixtureObjects {
            artboard,
            parent,
            child,
            layout_descendant,
        },
    ))
}

fn rendered_colors(scene: &mut Scene, instance: nuxie::InstanceId) -> Result<Vec<u32>> {
    scene.reset_renderer(instance)?;
    let mut factory = RecordingFactory::new();
    let mut cache = scene.new_draw_token(instance)?;
    let mut renderer = factory.make_renderer();
    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut cache)?;
    let stream = RenderStream::parse(&factory.stream()).context("parse Rust render stream")?;
    Ok(stream
        .frames
        .iter()
        .flat_map(|frame| &frame.commands)
        .filter_map(|command| match command {
            RenderCommand::DrawPath { paint, .. } => Some(paint.color),
            _ => None,
        })
        .collect())
}

fn assert_matrix_close(rust: [f32; 6], cpp: &[Value]) {
    assert_eq!(cpp.len(), 6, "C++ world matrix field count");
    for (field, (rust, cpp)) in rust.into_iter().zip(cpp).enumerate() {
        let cpp = cpp.as_f64().expect("C++ matrix field is numeric") as f32;
        assert!(
            (rust - cpp).abs() <= 0.000_01,
            "world matrix field {field} differs: Rust {rust}, C++ {cpp}"
        );
    }
}

fn assert_scalar_close(rust: f32, cpp: &Value, label: &str) {
    let cpp = cpp.as_f64().expect("C++ scalar is numeric") as f32;
    assert!(
        (rust - cpp).abs() <= 0.000_01,
        "{label} differs: Rust {rust}, C++ {cpp}"
    );
}

#[test]
fn settled_transform_live_writes_match_cpp() -> Result<()> {
    let Some(probe) = cpp_probe_path() else {
        eprintln!("skipping C++ transform live-write comparison; set RIVE_CPP_PROBE to enable");
        return Ok(());
    };

    let (mut scene, objects) = create_transform_scene()?;
    let instance = scene.instantiate(objects.artboard)?;
    let writes = [
        LiveWrite {
            cursor: scene.cursor(instance, objects.child, props::SCALE_X)?,
            local_id: 2,
            property_key: SCALE_X_KEY,
            value: 0.0,
            observed: objects.child,
            observed_local_id: 2,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.child, props::SCALE_X)?,
            local_id: 2,
            property_key: SCALE_X_KEY,
            value: 1.25,
            observed: objects.child,
            observed_local_id: 2,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.child, props::SCALE_Y)?,
            local_id: 2,
            property_key: SCALE_Y_KEY,
            value: 0.0,
            observed: objects.child,
            observed_local_id: 2,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.child, props::SCALE_Y)?,
            local_id: 2,
            property_key: SCALE_Y_KEY,
            value: 0.6,
            observed: objects.child,
            observed_local_id: 2,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.child, props::ROTATION)?,
            local_id: 2,
            property_key: ROTATION_KEY,
            value: 0.7,
            observed: objects.child,
            observed_local_id: 2,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.child, props::TRANSLATE_X)?,
            local_id: 2,
            property_key: TRANSLATE_X_KEY,
            value: 13.0,
            observed: objects.child,
            observed_local_id: 2,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.child, props::TRANSLATE_Y)?,
            local_id: 2,
            property_key: TRANSLATE_Y_KEY,
            value: -9.0,
            observed: objects.child,
            observed_local_id: 2,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.child, props::TRANSLATE_X)?,
            local_id: 2,
            property_key: TRANSLATE_X_KEY,
            value: -4.0,
            observed: objects.child,
            observed_local_id: 2,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.parent, props::SCALE_X)?,
            local_id: 1,
            property_key: SCALE_X_KEY,
            value: 0.5,
            observed: objects.child,
            observed_local_id: 2,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.parent, props::ROTATION)?,
            local_id: 1,
            property_key: ROTATION_KEY,
            value: -0.35,
            observed: objects.child,
            observed_local_id: 2,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.layout_descendant, props::SCALE_X)?,
            local_id: 8,
            property_key: SCALE_X_KEY,
            value: 0.0,
            observed: objects.layout_descendant,
            observed_local_id: 8,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.layout_descendant, props::SCALE_X)?,
            local_id: 8,
            property_key: SCALE_X_KEY,
            value: 1.1,
            observed: objects.layout_descendant,
            observed_local_id: 8,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.layout_descendant, props::ROTATION)?,
            local_id: 8,
            property_key: ROTATION_KEY,
            value: 0.2,
            observed: objects.layout_descendant,
            observed_local_id: 8,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.parent, props::WORLD_OPACITY)?,
            local_id: 1,
            property_key: OPACITY_KEY,
            value: 0.5,
            observed: objects.child,
            observed_local_id: 2,
        },
        LiveWrite {
            cursor: scene.cursor(instance, objects.child, props::WORLD_OPACITY)?,
            local_id: 2,
            property_key: OPACITY_KEY,
            value: 0.4,
            observed: objects.child,
            observed_local_id: 2,
        },
    ];
    let mut events = Vec::new();
    let mut frame = scene.frame();

    let _ = frame.advance(instance, 0.0, &mut events);
    let initial = frame
        .world_transform(instance, objects.child)
        .context("Rust child has an initial world transform")?;
    let mut rust_snapshots = Vec::new();
    for write in writes {
        assert!(frame.set(write.cursor, write.value)?);
        let _ = frame.advance(instance, 0.0, &mut events);
        let property_value = frame.get(write.cursor)?;
        assert_eq!(
            property_value, write.value,
            "the live property held after settlement"
        );
        let world_transform = frame
            .world_transform(instance, write.observed)
            .context("Rust observed object retains a world transform")?;
        rust_snapshots.push(RustSnapshot {
            property_value,
            world_transform: world_transform.0,
        });
    }
    drop(frame);

    let cpp = cpp_live_writes(&probe, &writes)?;
    let reports = cpp["artboards"][0]["runtimeSettledDoubleMutations"]
        .as_array()
        .context("C++ settled mutation reports")?;
    assert_eq!(reports.len(), writes.len());
    let report = &reports[0];
    assert_scalar_close(1.2, &report["beforeProperty"], "initial scaleX");
    assert_matrix_close(
        initial.0,
        report["beforeWorldTransform"]
            .as_array()
            .context("C++ initial world matrix")?,
    );

    for (index, ((write, rust), report)) in
        writes.iter().zip(&rust_snapshots).zip(reports).enumerate()
    {
        assert_eq!(report["applied"].as_bool(), Some(true));
        assert_eq!(
            report["propertyKey"].as_u64(),
            Some(u64::from(write.property_key))
        );
        assert_eq!(
            report["observedLocalId"].as_u64(),
            Some(write.observed_local_id as u64)
        );
        assert_scalar_close(
            rust.property_value,
            &report["propertyValue"],
            &format!("settled property at action {index}"),
        );
        assert_matrix_close(
            rust.world_transform,
            report["worldTransform"]
                .as_array()
                .context("C++ settled world matrix")?,
        );
    }

    assert_eq!(&rust_snapshots[0].world_transform[..2], &[0.0, 0.0]);
    assert_eq!(&rust_snapshots[2].world_transform[2..4], &[0.0, 0.0]);
    let rotated = rust_snapshots[4].world_transform;
    assert!((rotated[0] * rotated[3] - rotated[1] * rotated[2]).abs() > 0.001);
    assert_eq!(writes[10].observed, objects.layout_descendant);
    assert_eq!(&rust_snapshots[10].world_transform[..2], &[0.0, 0.0]);

    assert_scalar_close(
        0.25,
        &reports[13]["renderOpacity"],
        "parent-composed opacity",
    );
    assert_eq!(rust_snapshots[13].property_value, 0.5);
    assert_scalar_close(
        0.5,
        &reports[13]["propertyValue"],
        "parent opacity property",
    );
    assert_scalar_close(0.2, &reports[14]["renderOpacity"], "child render opacity");
    assert_eq!(rust_snapshots[14].property_value, 0.4);
    assert_scalar_close(0.4, &reports[14]["propertyValue"], "child opacity property");
    let colors = rendered_colors(&mut scene, instance)?;
    assert!(
        colors.contains(&0x3333_6699),
        "Rust render output must apply child render opacity 0.2: {colors:#x?}"
    );
    Ok(())
}
