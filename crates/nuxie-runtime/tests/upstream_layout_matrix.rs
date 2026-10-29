//! Data-driven translation of runtime/layout_matrix_test.cpp at 78b07a3f.
#![cfg(feature = "tools")]

#[path = "support/css_layout_matrix.rs"]
mod css_layout_matrix;

use nuxie_render_api as render;
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    artboard_component_list::ArtboardComponentList,
    component::Component,
    core::CoreObject,
    file_asset_loader::{FileAssetLoader, FileAssetLoaderRef},
    generated::core_registry::CoreField,
    layout_component::LayoutComponent,
    math::aabb::Aabb,
    nested_artboard_layout::NestedArtboardLayout,
    shapes::image::Image,
};
use nuxie_runtime::{CoreHandle, File, RuntimeFactoryHandle};
use std::{
    cell::Cell,
    collections::BTreeMap,
    path::{Path, PathBuf},
    rc::Rc,
};

#[derive(Default)]
struct Check {
    legacy: bool,
    known_defect: bool,
    component: String,
    field: String,
    values: Vec<f32>,
}
#[derive(Default)]
struct Manifest {
    fixture: String,
    artboard: String,
    known_defect: String,
    checks: Vec<Check>,
}

fn parse_manifests(path: &Path) -> Vec<Manifest> {
    let text = std::fs::read_to_string(path).expect("read expectation manifest");
    let mut out: Vec<Manifest> = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut words = line.split_whitespace();
        let tag = words.next().unwrap_or("");
        if tag == "F" {
            out.push(Manifest {
                fixture: words.next().unwrap_or("").into(),
                ..Default::default()
            });
            continue;
        }
        let m = out.last_mut().expect("manifest starts with F");
        match tag {
            "A" => m.artboard = words.next().unwrap_or("").into(),
            "D" => m.known_defect = line.trim_start().strip_prefix('D').unwrap().into(),
            "C" | "L" | "X" => {
                let component = words.next().unwrap_or("").into();
                let field = words.next().unwrap_or("").into();
                let values = words.map_while(|v| v.parse::<f32>().ok()).collect();
                m.checks.push(Check {
                    legacy: tag == "L",
                    known_defect: tag == "X",
                    component,
                    field,
                    values,
                });
            }
            _ => {}
        }
    }
    out
}

#[derive(serde::Deserialize)]
struct MatrixOverrides {
    overrides: Vec<CaseOverride>,
}

#[derive(serde::Deserialize)]
struct CaseOverride {
    id: String,
    fields: BTreeMap<String, BTreeMap<usize, f32>>,
}

fn apply_overrides(
    matrices: &mut [Vec<Manifest>],
    overrides: &[CaseOverride],
) -> Result<(), String> {
    for entry in overrides {
        let manifest = matrices
            .iter_mut()
            .flatten()
            .find(|manifest| manifest.fixture == entry.id)
            .ok_or_else(|| format!("stale matrix override: missing case {}", entry.id))?;
        for (field, values) in &entry.fields {
            let check = manifest
                .checks
                .iter_mut()
                .find(|check| format!("{}.{}", check.component, check.field) == *field)
                .ok_or_else(|| {
                    format!("stale matrix override: missing field {} {field}", entry.id)
                })?;
            for (&index, &value) in values {
                let slot = check.values.get_mut(index).ok_or_else(|| {
                    format!(
                        "stale matrix override: missing value {} {field}[{index}]",
                        entry.id
                    )
                })?;
                *slot = value;
            }
        }
    }
    Ok(())
}

#[test]
fn matrix_overrides_reject_missing_cases_and_fields() {
    let entry = CaseOverride {
        id: "stack-case".into(),
        fields: BTreeMap::from([("Subject.world".into(), BTreeMap::from([(4, 130.0)]))]),
    };
    let mut matrices = vec![vec![]];
    assert_eq!(
        apply_overrides(&mut matrices, std::slice::from_ref(&entry)),
        Err("stale matrix override: missing case stack-case".into())
    );
    matrices[0].push(Manifest {
        fixture: entry.id.clone(),
        ..Default::default()
    });
    assert_eq!(
        apply_overrides(&mut matrices, std::slice::from_ref(&entry)),
        Err("stale matrix override: missing field stack-case Subject.world".into())
    );
    matrices[0][0].checks.push(Check {
        component: "Subject".into(),
        field: "world".into(),
        values: vec![1.0],
        ..Default::default()
    });
    assert_eq!(
        apply_overrides(&mut matrices, &[entry]),
        Err("stale matrix override: missing value stack-case Subject.world[4]".into())
    );
}

const MARGIN: f32 = 1e-3;
fn matches_within(actual: &[f32], expected: &[f32]) -> bool {
    actual.len() == expected.len()
        && actual.iter().zip(expected).all(|(a, e)| {
            let d = (a - e).abs();
            d.is_finite() && d <= MARGIN
        })
}
// Catch Approx stores doubles; its default epsilon is 100 * float epsilon.
fn approx(actual: f32, expected: f32) -> bool {
    let a = f64::from(actual);
    let e = f64::from(expected);
    let within = |m: f64| e + m >= a && a + m >= e;
    let scale = if e.is_infinite() { 0.0 } else { e.abs() };
    within(f64::from(MARGIN)) || within(f64::from(f32::EPSILON * 100.0) * scale)
}
fn bounds(b: Aabb) -> Vec<f32> {
    vec![b.min_x, b.min_y, b.width(), b.height()]
}

// Preserve C++ virtual localBounds dispatch rather than reading the embedded
// TransformComponent's default rectangle for every derived owner.
fn local_bounds(object: &dyn CoreObject) -> Option<Aabb> {
    object.as_transform_component()?;
    if let Some(image) = object.as_any().downcast_ref::<Image>() {
        return Some(image.local_bounds());
    }
    object.semantic_provider_local_bounds()
}

fn read_field(component: &CoreHandle, container: &CoreHandle, field: &str) -> Option<Vec<f32>> {
    if field == "collected" {
        return container
            .with(|c| {
                c.as_layout_component()
                    .map(|c| vec![u8::from(c.collects_for_layout(component)) as f32])
            })
            .flatten();
    }
    if field == "layoutBounds" {
        if let Some(b) = component
            .with(|o| o.as_layout_component().map(|c| c.layout_bounds()))
            .flatten()
        {
            return Some(bounds(b));
        }
        let provider =
            nuxie_runtime::source::layout::layout_node_provider::from_component(component)?;
        return provider
            .with(|o| {
                o.as_layout_node_provider()
                    .map(|p| bounds(p.layout_bounds()))
            })
            .flatten();
    }
    component
        .with_mut(|object| {
            if field == "world" {
                return object.as_world_transform_component().map(|c| {
                    let w = c.world_transform();
                    (0..6).map(|i| w[i]).collect()
                });
            }
            if field == "contentSize" {
                return local_bounds(object).map(|b| vec![b.width(), b.height()]);
            }
            if let Some(list) = object.as_any().downcast_ref::<ArtboardComponentList>() {
                if field == "itemCount" {
                    return Some(vec![list.artboard_count() as f32]);
                }
                if field.len() > 4 && field.starts_with("item") {
                    let index = i32::from(field.as_bytes()[4]) - i32::from(b'0');
                    if index < 0 || index as usize >= list.artboard_count() {
                        return None;
                    }
                    match &field[5..] {
                        "Bounds" => {
                            return Some(bounds(list.layout_bounds_for_node(index as usize)));
                        }
                        "Pos" => {
                            let p = list.item_position(index);
                            return Some(vec![p.x, p.y]);
                        }
                        _ => {}
                    }
                }
            }
            if field == "isRow" || field == "isStack" {
                let row = field == "isRow";
                if let Some(n) = object.as_any().downcast_ref::<NestedArtboardLayout>() {
                    return Some(vec![
                        u8::from(if row { n.is_row() } else { n.is_stack() }) as f32
                    ]);
                }
                if let Some(l) = object.as_any().downcast_ref::<ArtboardComponentList>() {
                    return Some(vec![u8::from(if row {
                        l.main_axis_is_row()
                    } else {
                        l.is_stack()
                    }) as f32]);
                }
                return None;
            }
            if field.starts_with("computed") {
                object.as_node()?;
                let (x, y) = match field {
                    "computedLocal" => {
                        (CoreField::NodeComputedLocalX, CoreField::NodeComputedLocalY)
                    }
                    "computedWorld" => {
                        (CoreField::NodeComputedWorldX, CoreField::NodeComputedWorldY)
                    }
                    "computedRoot" => (CoreField::NodeComputedRootX, CoreField::NodeComputedRootY),
                    "computedSize" => (CoreField::NodeComputedWidth, CoreField::NodeComputedHeight),
                    _ => return None,
                };
                return Some(vec![object.get_double(x), object.get_double(y)]);
            }
            match field {
                "localBounds" => local_bounds(object).map(bounds),
                "worldBounds" => {
                    if let Some(c) = object.as_layout_component() {
                        return Some(bounds(c.world_bounds()));
                    }
                    object.as_shape_mut().map(|s| bounds(s.world_bounds()))
                }
                _ => None,
            }
        })
        .flatten()
}

#[derive(Clone)]
struct SizedImage {
    size_state: nuxie_render_api::RenderImageSizeState,
    width: u32,
    height: u32,
    identity: Rc<()>,
}
impl render::RenderImage for SizedImage {
    fn image_size_state(&self) -> &nuxie_render_api::RenderImageSizeState {
        &self.size_state
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn retain_image(&self) -> Rc<dyn render::RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.identity) as usize
    }
    fn width(&self) -> u32 {
        self.size_state.visible_width(self.width)
    }
    fn height(&self) -> u32 {
        self.size_state.visible_height(self.height)
    }
}
#[derive(Default)]
struct MatrixFactory(RecordingFactory);
impl render::Factory for MatrixFactory {
    fn make_render_buffer(
        &mut self,
        ty: render::RenderBufferType,
        flags: render::RenderBufferFlags,
        size: usize,
    ) -> Box<dyn render::RenderBuffer> {
        self.0.make_render_buffer(ty, flags, size)
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[render::ColorInt],
        stops: &[f32],
    ) -> Box<dyn render::RenderShader> {
        self.0.make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }
    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[render::ColorInt],
        stops: &[f32],
    ) -> Box<dyn render::RenderShader> {
        self.0.make_radial_gradient(cx, cy, radius, colors, stops)
    }
    fn make_render_path(
        &mut self,
        path: render::RawPath,
        rule: render::FillRule,
    ) -> Box<dyn render::RenderPath> {
        self.0.make_render_path(path, rule)
    }
    fn make_empty_render_path(&mut self) -> Box<dyn render::RenderPath> {
        self.0.make_empty_render_path()
    }
    fn make_render_paint(&mut self) -> Box<dyn render::RenderPaint> {
        self.0.make_render_paint()
    }
    fn decode_font(
        &mut self,
        bytes: &[u8],
    ) -> Result<render::DecodedFont, render::FontDecodeError> {
        self.0.decode_font(bytes)
    }
    fn decode_image(
        &mut self,
        bytes: &[u8],
    ) -> Result<Box<dyn render::RenderImage>, render::ImageDecodeError> {
        let bitmap = nuxie_image_codec::decode_image_rgba_unbounded(bytes)
            .ok_or(render::ImageDecodeError)?;
        Ok(Box::new(SizedImage {
            size_state: Default::default(),
            width: bitmap.width,
            height: bitmap.height,
            identity: Rc::new(()),
        }))
    }
}

struct MatrixAssetLoader {
    root: PathBuf,
    attempted: Rc<Cell<usize>>,
    resolved: Rc<Cell<usize>>,
}
impl FileAssetLoader for MatrixAssetLoader {
    fn load_contents(
        &mut self,
        asset: CoreHandle,
        _: &[u8],
        factory: &RuntimeFactoryHandle,
    ) -> bool {
        self.attempted.set(self.attempted.get() + 1);
        let names = asset
            .with(|a| {
                let a = a.as_file_asset().expect("FileAsset");
                let base = a.file_asset_base();
                [
                    format!("{}.{}", base.base.name(), a.file_extension()),
                    base.unique_filename(a.file_extension()),
                ]
            })
            .expect("live asset");
        for name in names {
            let Ok(mut bytes) = std::fs::read(self.root.join(name)) else {
                continue;
            };
            let decoded = asset
                .with_mut(|a| {
                    a.as_file_asset_mut()
                        .unwrap()
                        .file_asset_decode(&mut bytes, factory)
                })
                .unwrap();
            if decoded {
                self.resolved.set(self.resolved.get() + 1);
            }
            return decoded;
        }
        false
    }
}

#[test]
fn the_generated_layout_matrix_conforms() {
    let upstream = std::env::var_os("RIVE_RUNTIME_DIR");
    let (dir, assets) = upstream.as_ref().map_or_else(
        || {
            let root = PathBuf::from(
                option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
            )
            .join("../../fixtures/sync/layout");
            (root.join("matrix"), root.join("assets"))
        },
        |root| {
            let root = PathBuf::from(root).join("tests/unit_tests/assets");
            (root.join("layout/matrix"), root)
        },
    );
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .expect("matrix directory")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "expect"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty());
    let mut matrices: Vec<_> = paths.iter().map(|path| parse_manifests(path)).collect();
    if upstream.is_some() {
        let overrides: MatrixOverrides = serde_json::from_str(include_str!(
            "../../../fixtures/sync/layout/matrix_overrides.json"
        ))
        .expect("parse matrix overrides");
        apply_overrides(&mut matrices, &overrides.overrides)
            .unwrap_or_else(|message| panic!("{message}"));
    }
    let mut checked = 0;
    let mut excluded_aspect_ratio_cases = 0;
    let mut excluded_aspect_ratio_fields = 0;
    let mut known_defects_confirmed = 0;
    let mut layout_divergences_seen = 0;
    let mut assets_attempted = 0;
    let mut assets_resolved = 0;
    let mut unknown_fields = Vec::new();
    let mut failures = Vec::new();
    for (path, manifests) in paths.iter().zip(matrices) {
        let riv_path = path.with_extension("riv");
        let attempted = Rc::new(Cell::new(0));
        let resolved = Rc::new(Cell::new(0));
        let loader = FileAssetLoaderRef::new(Box::new(MatrixAssetLoader {
            root: assets.clone(),
            attempted: attempted.clone(),
            resolved: resolved.clone(),
        }));
        // Decode the bitmap but retain only dimensions, as upstream does.
        // FontAsset uses the normal real HbFont shaping implementation.
        let mut factory = PersistentFactory::new(MatrixFactory::default());
        let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        let bytes = std::fs::read(&riv_path).expect("matrix riv");
        let file = File::import(&bytes, factory, None, Some(loader), None).expect("matrix imports");
        if resolved.get() != attempted.get() {
            failures.push(format!(
                "{}: assets {}/{}",
                riv_path.display(),
                resolved.get(),
                attempted.get()
            ));
        }
        assets_attempted += attempted.get();
        assets_resolved += resolved.get();
        for manifest in manifests {
            let artboard = file
                .with_file(|f| f.artboard_named(&manifest.artboard))
                .expect("named matrix artboard");
            if let Some(vm) = file.with_file(|f| {
                f.create_default_view_model_instance_for_artboard(artboard.core_handle())
            }) {
                artboard.bind_view_model_instance(Some(vm));
            }
            artboard.advance_default(0.0);
            if css_layout_matrix::EXCLUDED_ASPECT_RATIO_CASES.contains(&manifest.fixture.as_str()) {
                // Import and named-artboard lookup above deliberately remain mandatory:
                // the new fixture contains unknown uint property 546, which the
                // existing header-ToC fallback must skip without losing artboards.
                // Its Yoga geometry is not our approved Taffy/CSS contract.
                excluded_aspect_ratio_cases += 1;
                excluded_aspect_ratio_fields += manifest.checks.len();
                eprintln!(
                    "CSS exclusion (Yoga aspect-ratio policy): {}: imported and advanced; {} geometry fields excluded",
                    manifest.fixture,
                    manifest.checks.len()
                );
                continue;
            }
            let container = artboard
                .with_artboard(|a| a.find_handle::<LayoutComponent>("Container"))
                .unwrap_or_else(|| artboard.core_handle());
            let layout_divergence = css_layout_matrix::divergence(&manifest.fixture);
            let mut first_difference = None;
            for check in manifest.checks {
                let context = format!("{} {}.{}", manifest.fixture, check.component, check.field);
                let component = artboard
                    .with_artboard(|a| a.find_handle::<Component>(&check.component))
                    .unwrap_or_else(|| panic!("missing component: {context}"));
                let Some(actual) = read_field(&component, &container, &check.field) else {
                    unknown_fields.push(check.field.clone());
                    failures.push(format!("no reader for {context}"));
                    continue;
                };
                assert_eq!(actual.len(), check.values.len(), "{context}");
                if check.known_defect {
                    if matches_within(&actual, &check.values) {
                        failures.push(format!("known defect no longer reproduces ({}): {context}, actual {actual:?} == correct {:?}; clear marker", manifest.known_defect, check.values));
                    } else {
                        eprintln!(
                            "known defect confirmed ({}): {context} actual {actual:?} correct {:?}",
                            manifest.known_defect, check.values
                        );
                        known_defects_confirmed += 1;
                    }
                } else if check.legacy {
                    if matches_within(&actual, &check.values) {
                        failures.push(format!("legacy value reproduced: {context} {actual:?}"));
                    }
                } else {
                    for (i, (&a, &e)) in actual.iter().zip(&check.values).enumerate() {
                        if !approx(a, e) && first_difference.is_none() {
                            first_difference = Some(format!(
                                "{}.{}[{i}]: {a} != {e}",
                                check.component, check.field
                            ));
                        }
                        let expected = css_layout_matrix::expected(
                            layout_divergence,
                            &check.component,
                            &check.field,
                            i,
                            e,
                        );
                        if !approx(a, expected) {
                            failures.push(format!("{context}[{i}]: {a} != {expected} (C++ {e})"));
                        }
                    }
                }
                checked += 1;
            }
            if let Some(divergence) = layout_divergence {
                layout_divergences_seen += 1;
                if first_difference.as_deref() != Some(divergence.first_difference) {
                    failures.push(format!(
                        "layout first difference moved: {}: {:?} != {:?}",
                        manifest.fixture, first_difference, divergence.first_difference
                    ));
                }
                eprintln!(
                    "{}: {}: {}",
                    divergence.reason, manifest.fixture, divergence.first_difference
                );
            }
        }
    }
    assert_eq!(
        layout_divergences_seen,
        css_layout_matrix::LAYOUT_DIVERGENCES.len()
    );
    assert!(checked > 0);
    eprintln!(
        "layout matrix: {} files, {checked} fields checked, {} unknown fields, {known_defects_confirmed} known defects, {assets_resolved}/{assets_attempted} referenced assets resolved, {excluded_aspect_ratio_cases} imported CSS-excluded aspect-ratio cases ({excluded_aspect_ratio_fields} geometry fields)",
        paths.len(),
        unknown_fields.len()
    );
    assert!(
        failures.is_empty(),
        "{} nonfatal failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
