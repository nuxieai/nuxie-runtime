use super::*;
use crate::source::{
    assets::image_asset::ImageAsset,
    component_dirt::ComponentDirt,
    generated::core_registry::CoreField,
    math::{aabb::IAabb, mat2d::Mat2D},
    shapes::image::Image,
};
use nuxie_render_api::{
    BlendMode, ImageSampler, RenderBuffer, RenderImage, RenderPaint, RenderPath, Renderer,
};
use std::rc::Rc;

#[path = "layout_picture_attachment_tests.rs"]
mod attachment_tests;

#[derive(Clone)]
struct Pixels(u32, u32, Rc<()>, nuxie_render_api::RenderImageSizeState);
impl RenderImage for Pixels {
    fn image_size_state(&self) -> &nuxie_render_api::RenderImageSizeState {
        &self.3
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn retain_image(&self) -> Rc<dyn RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.2) as usize
    }
    fn width(&self) -> u32 {
        self.3.visible_width(self.0)
    }
    fn height(&self) -> u32 {
        self.3.visible_height(self.1)
    }
}

#[derive(Default)]
struct Capture {
    matrix: Mat2D,
    stack: Vec<Mat2D>,
    rect: Option<[f32; 4]>,
}
impl Renderer for Capture {
    fn save(&mut self) {
        self.stack.push(self.matrix);
    }
    fn restore(&mut self) {
        self.matrix = self.stack.pop().unwrap();
    }
    fn transform(&mut self, m: nuxie_render_api::Mat2D) {
        self.matrix = self.matrix * Mat2D::new(m.0[0], m.0[1], m.0[2], m.0[3], m.0[4], m.0[5]);
    }
    fn draw_path(&mut self, _: &dyn RenderPath, _: &dyn RenderPaint) {}
    fn clip_path(&mut self, _: &dyn RenderPath) {}
    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: BlendMode,
        _: f32,
    ) {
        let image = image.unwrap();
        let m = self.matrix;
        self.rect = Some([
            m[4],
            m[5],
            m[0] * image.width() as f32,
            m[3] * image.height() as f32,
        ]);
    }
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        _: BlendMode,
        _: f32,
    ) {
        panic!("unexpected mesh");
    }
    fn modulate_opacity(&mut self, _: f32) {}
}

fn host(participant: &CoreHandle) -> CoreHandle {
    participant
        .with_downcast::<LayoutParticipant, _>(LayoutParticipant::measurement_host_handle)
        .flatten()
        .unwrap()
}
fn declare(host: &CoreHandle, width: f32, height: f32) {
    let asset = host
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .unwrap();
    asset.with_mut(|asset| {
        asset.set_double(CoreField::DrawableAssetWidth, width);
        asset.set_double(CoreField::DrawableAssetHeight, height);
    });
}
fn decode(host: &CoreHandle, width: u32, height: u32) {
    let asset = host
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .unwrap();
    ImageAsset::set_render_image_occurrence(
        &asset,
        Some(Rc::new(Pixels(
            width,
            height,
            Rc::new(()),
            Default::default(),
        ))),
    );
}
fn draw(host: &CoreHandle, occurrence: bool) -> [f32; 4] {
    crate::source::transform_component::update_occurrence::<Image>(
        host,
        ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM,
    );
    let mut capture = Capture::default();
    if occurrence {
        Image::draw_occurrence(host, &mut capture);
    } else {
        host.with_downcast_mut::<Image, _>(|image| image.draw(&mut capture));
    }
    capture.rect.unwrap()
}
fn close(actual: [f32; 4], expected: [f32; 4]) {
    for (a, e) in actual.into_iter().zip(expected) {
        assert!((a - e).abs() < 0.002, "{actual:?} != {expected:?}");
    }
}

fn receiver_picture_fixture(css: bool) -> (CoreArena, CoreHandle, CoreHandle, CoreHandle) {
    receiver_picture_fixture_with_parent_mode(css, false)
}

fn receiver_picture_fixture_with_parent_mode(
    css: bool,
    parent_css: bool,
) -> (CoreArena, CoreHandle, CoreHandle, CoreHandle) {
    let (arena, root, participant) = if css {
        picture_participant_fixture(false, &[], &[(61021, 1)], &[(61022, 100.0)])
    } else {
        let arena = CoreArena::default();
        let root = layout(&arena, None, Some(200.0), true);
        if parent_css {
            let record = arena.insert(crate::css_layout::LayoutCssStyle::default());
            attach(&arena, &root, &record);
            root.with_downcast_mut::<LayoutComponent, _>(|root| {
                root.add_layout_style_applier(record)
            });
        }
        let image = arena.insert(Image::default());
        attach(&arena, &root, &image);
        let mut participant = LayoutParticipant::default();
        participant
            .base
            .set_layout_width_scale_type_value(LayoutScaleType::Fill as u8);
        participant
            .base
            .set_layout_height_scale_type_value(LayoutScaleType::Fill as u8);
        let participant = arena.insert(participant);
        attach(&arena, &image, &participant);
        let mut context = ParentContext {
            arena: &arena,
            parent: image,
        };
        assert_eq!(
            LayoutParticipant::on_added_clean_occurrence(&participant, &mut context),
            StatusCode::Ok
        );
        LayoutComponent::sync_style_occurrence(&root);
        LayoutComponent::sync_layout_children_occurrence(&root);
        (arena, root, participant)
    };
    let image = host(&participant);
    crate::source::layout::picture_layout_mode::attached(image.clone());
    if !css {
        Image::set_asset_occurrence(&image, Some(arena.insert(ImageAsset::default())));
    }
    declare(&image, 480.0, 270.0);
    // Keep the translated image's automatic minimum below its Fill slots.
    decode(&image, 160, 90);
    (arena, root, participant, image)
}

#[test]
fn picture_receiver_update_bounds_does_not_reborrow_participant() {
    for css in [false, true] {
        let (_arena, root, participant, image) = receiver_picture_fixture(css);
        LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
        participant.with_downcast_mut::<LayoutParticipant, _>(|participant| {
            participant.update_layout_bounds(false)
        });
        let width = image
            .with_downcast::<Image, _>(Image::computed_width)
            .unwrap();
        near(width, if css { 100.0 } else { 200.0 });
    }
}

#[test]
fn picture_receiver_advance_does_not_reborrow_participant() {
    use crate::source::{
        advance_flags::AdvanceFlags,
        layout::layout_enums::{LayoutDirection, LayoutStyleInterpolation},
    };
    for css in [false, true] {
        let (_arena, root, participant, image) = receiver_picture_fixture(css);
        LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
        LayoutComponent::update_layout_bounds_occurrence(&root, false);
        participant.with_downcast_mut::<LayoutParticipant, _>(|participant| {
            participant.cascade_layout_style(
                LayoutStyleInterpolation::Linear,
                None,
                1.0,
                LayoutDirection::Ltr,
            );
        });
        if css {
            let record = participant
                .with_downcast::<LayoutParticipant, _>(|participant| {
                    participant.css_record().unwrap()
                })
                .unwrap();
            CssTextLeaf::write(&record, &[], &[(61022, 200.0)]);
            LayoutParticipant::sync_style_changes_occurrence(&participant, None);
        } else {
            root.with_downcast_mut::<LayoutComponent, _>(|root| root.base.set_width_value(300.0));
            LayoutComponent::sync_style_occurrence(&root);
        }
        LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
        participant.with_downcast_mut::<LayoutParticipant, _>(|participant| {
            participant.update_layout_bounds(true);
            for _ in 0..3 {
                participant.advance_component(0.5, AdvanceFlags::NEW_FRAME | AdvanceFlags::ANIMATE);
            }
        });
        near(
            image
                .with_downcast::<Image, _>(Image::computed_width)
                .unwrap(),
            if css { 200.0 } else { 300.0 },
        );
    }
}

fn clean_picture_transform(arena: &CoreArena, image: &CoreHandle) {
    let parent = image
        .with(|image| image.component_parent_handle())
        .flatten()
        .unwrap();
    let mut context = ParentContext { arena, parent };
    assert_eq!(
        image.with_mut(|image| image
            .as_transform_component_mut()
            .unwrap()
            .on_added_clean(&mut context)),
        Some(StatusCode::Ok)
    );
}

fn assert_padded_picture_paint(fit: u32) {
    for wrapped in [true, false] {
        for (padding, border) in [(10.0, 0.0), (6.0, 4.0)] {
            let (arena, root, item, participant) = picture_layout_fixture(
                false,
                wrapped,
                &[],
                &[(61001, 1), (61021, 1)],
                &[
                    (61022, 100.0),
                    (61063, padding),
                    (61065, padding),
                    (61067, padding),
                    (61069, padding),
                    (61071, border),
                    (61072, border),
                    (61073, border),
                    (61074, border),
                ],
            );
            let image = host(&participant);
            clean_picture_transform(&arena, &image);
            image.with_downcast_mut::<Image, _>(|image| image.base.set_fit_value(fit));
            decode(&image, 480, 270);
            assert_picture_size(&root, &item, 100.0, 65.0);
            LayoutComponent::update_layout_bounds_occurrence(&root, false);
            for occurrence in [false, true] {
                close(draw(&image, occurrence), [10.0, 10.0, 80.0, 45.0]);
            }
        }
    }
}

#[test]
fn css_padded_picture_fill_paints_its_content_box() {
    assert_padded_picture_paint(7);
}

#[test]
fn css_padded_picture_contain_paints_its_content_box() {
    assert_padded_picture_paint(1);
}

#[test]
fn css_padded_picture_cover_paints_its_content_box() {
    assert_padded_picture_paint(2);
}

#[test]
fn css_padded_picture_declared_size_ignores_decode_resolution() {
    for wrapped in [true, false] {
        for fit in [7, 1, 2] {
            let (arena, root, item, participant) = picture_layout_fixture(
                false,
                wrapped,
                &[],
                &[(61001, 1), (61021, 1)],
                &[
                    (61022, 100.0),
                    (61063, 10.0),
                    (61065, 10.0),
                    (61067, 10.0),
                    (61069, 10.0),
                ],
            );
            let image = host(&participant);
            clean_picture_transform(&arena, &image);
            declare(&image, 6000.0, 4000.0);
            image.with_downcast_mut::<Image, _>(|image| image.base.set_fit_value(fit));
            decode(&image, 6000, 4000);
            assert_picture_size(&root, &item, 100.0, 73.33333);
            LayoutComponent::update_layout_bounds_occurrence(&root, false);
            let full = draw(&image, true);
            close(full, [10.0, 10.0, 80.0, 53.33333]);
            decode(&image, 4096, 2731);
            for occurrence in [false, true] {
                close(draw(&image, occurrence), full);
            }
        }
    }
}

#[test]
fn css_declared_picture_keeps_natural_size_after_decode() {
    let (_arena, root, item) = picture_participant_fixture(false, &[], &[], &[]);
    let image = host(&item);
    declare(&image, 6000.0, 4000.0);
    assert_picture_size(&root, &item, 6000.0, 4000.0);
    decode(&image, 4096, 2731);
    assert_picture_size(&root, &item, 6000.0, 4000.0);
    LayoutComponent::update_layout_bounds_occurrence(&root, false);
    for occurrence in [false, true] {
        close(draw(&image, occurrence), [0.0, 0.0, 6000.0, 4000.0]);
    }
    image.with_downcast::<Image, _>(|image| {
        let bounds = image.local_bounds();
        assert_eq!([bounds.width(), bounds.height()], [6000.0, 4000.0]);
        let hit = crate::source::hit_info::HitInfo {
            area: IAabb {
                left: 5500,
                top: 3500,
                right: 5502,
                bottom: 3502,
            },
            mounts: Vec::new(),
        };
        assert!(image.hit_test(&hit, Mat2D::default()).is_some());
    });
}

#[test]
fn css_declared_picture_percentage_width_keeps_declared_ratio() {
    let (_arena, root, item) =
        picture_participant_fixture(false, &[], &[(61021, 7)], &[(61023, 100.0)]);
    let image = host(&item);
    declare(&image, 6000.0, 4000.0);
    decode(&image, 4096, 2731);
    assert_picture_size(&root, &item, 328.0, 328.0 * 2.0 / 3.0);
}

#[test]
fn css_declared_picture_all_fits_ignore_decode_resolution() {
    for fit in 0..=7 {
        let (_arena, root, item) = picture_participant_fixture(
            false,
            &[],
            &[(61021, 1), (61024, 1)],
            &[(61022, 400.0), (61025, 200.0)],
        );
        let image = host(&item);
        declare(&image, 6000.0, 4000.0);
        image.with_downcast_mut::<Image, _>(|image| image.base.set_fit_value(fit));
        decode(&image, 6000, 4000);
        assert_picture_size(&root, &item, 400.0, 200.0);
        LayoutComponent::update_layout_bounds_occurrence(&root, false);
        let full = draw(&image, true);
        decode(&image, 4096, 2731);
        close(draw(&image, true), full);
        close(draw(&image, false), full);
        if fit == 2 {
            close(full, [0.0, -100.0 / 3.0, 400.0, 800.0 / 3.0]);
        }
    }
}

#[test]
fn css_undeclared_picture_uses_decoded_size() {
    for declared in [(0.0, 0.0), (6000.0, 0.0), (0.0, 4000.0)] {
        let (_arena, root, item) = picture_participant_fixture(false, &[], &[], &[]);
        let image = host(&item);
        declare(&image, declared.0, declared.1);
        decode(&image, 4096, 2731);
        assert_picture_size(&root, &item, 4096.0, 2731.0);
        LayoutComponent::update_layout_bounds_occurrence(&root, false);
        close(draw(&image, true), [0.0, 0.0, 4096.0, 2731.0]);
    }
}

#[test]
fn translated_declared_picture_uses_decoded_size() {
    let arena = CoreArena::default();
    let root = layout(&arena, None, Some(8000.0), true);
    let image = arena.insert(Image::default());
    attach(&arena, &root, &image);
    let asset = arena.insert(ImageAsset::default());
    Image::set_asset_occurrence(&image, Some(asset));
    declare(&image, 6000.0, 4000.0);
    decode(&image, 4096, 2731);
    image.with_downcast::<Image, _>(|image| {
        assert_eq!([image.width(), image.height()], [4096.0, 2731.0]);
        let measured = image.measure_layout(
            0.0,
            crate::source::layout::layout_measure_mode::LayoutMeasureMode::Undefined,
            0.0,
            crate::source::layout::layout_measure_mode::LayoutMeasureMode::Undefined,
        );
        assert_eq!([measured.x, measured.y], [4096.0, 2731.0]);
    });
    close(draw(&image, true), [-2048.0, -1365.5, 4096.0, 2731.0]);
}

#[test]
fn css_declared_picture_content_measures_and_draws_declared_size() {
    for grouped in [false, true] {
        let arena = CoreArena::default();
        let root = layout(&arena, None, Some(8000.0), true);
        let item = layout(&arena, Some(&root), None, false);
        for (owner, enums, numbers) in [
            (
                &root,
                vec![(61000, 1), (61013, 4), (61021, 1)],
                vec![(61022, 8000.0)],
            ),
            (&item, vec![], vec![]),
        ] {
            let record = arena.insert(crate::css_layout::LayoutCssStyle::default());
            attach(&arena, owner, &record);
            CssTextLeaf::write(&record, &enums, &numbers);
            owner.with_downcast_mut::<LayoutComponent, _>(|owner| {
                owner.add_layout_style_applier(record)
            });
        }
        let image = arena.insert(Image::default());
        let parent = if grouped {
            let group = arena.insert(crate::source::node::Node::default());
            attach(&arena, &item, &group);
            group
        } else {
            item.clone()
        };
        attach(&arena, &parent, &image);
        image.with_downcast_mut::<Image, _>(|image| {
            image.base.set_origin_x_value(0.0);
            image.base.set_origin_y_value(0.0);
        });
        crate::source::layout::picture_layout_mode::attached(image.clone());
        let asset = arena.insert(ImageAsset::default());
        Image::set_asset_occurrence(&image, Some(asset));
        declare(&image, 6000.0, 4000.0);
        LayoutComponent::sync_style_occurrence(&root);
        LayoutComponent::sync_style_occurrence(&item);
        LayoutComponent::sync_layout_children_occurrence(&item);
        LayoutComponent::sync_layout_children_occurrence(&root);
        assert_picture_size(&root, &item, 6000.0, 4000.0);
        decode(&image, 4096, 2731);
        assert_picture_size(&root, &item, 6000.0, 4000.0);
        close(draw(&image, true), [0.0, 0.0, 6000.0, 4000.0]);
    }
}

#[test]
fn css_declared_picture_mesh_geometry_ignores_decode_resolution() {
    use crate::source::shapes::{mesh::Mesh, mesh_host::MeshHost, mesh_vertex::MeshVertex};
    let (arena, root, item) = picture_participant_fixture(
        false,
        &[],
        &[(61021, 1), (61024, 1)],
        &[(61022, 400.0), (61025, 200.0)],
    );
    let image = host(&item);
    declare(&image, 6000.0, 4000.0);
    image.with_downcast_mut::<Image, _>(|image| image.base.set_fit_value(2));
    let mesh = arena.insert(Mesh::default());
    let mut context = ParentContext {
        arena: &arena,
        parent: image.clone(),
    };
    mesh.with_downcast_mut::<Mesh, _>(|mesh| {
        mesh.base.set_parent_id_value(1);
        assert_eq!(mesh.on_added_dirty(&mut context), StatusCode::Ok);
        mesh.decode_triangle_index_bytes(&[0, 1, 2]);
    });
    for (x, y) in [(-3000.0, -2000.0), (3000.0, -2000.0), (3000.0, 2000.0)] {
        let mut vertex = MeshVertex::default();
        vertex.base.set_x_value(x);
        vertex.base.set_y_value(y);
        let vertex = arena.insert(vertex);
        mesh.with_downcast_mut::<Mesh, _>(|mesh| mesh.add_vertex(vertex));
    }
    decode(&image, 6000, 4000);
    assert_picture_size(&root, &item, 400.0, 200.0);
    LayoutComponent::update_layout_bounds_occurrence(&root, false);
    let triangles = || {
        crate::source::transform_component::update_occurrence::<Image>(
            &image,
            ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM,
        );
        mesh.with_downcast::<Mesh, _>(Mesh::rendered_triangles)
            .flatten()
            .unwrap()
    };
    let full = triangles();
    decode(&image, 4096, 2731);
    let small = triangles();
    for (actual, expected) in small[0].iter().zip(full[0].iter()) {
        close(
            [actual.x, actual.y, 0.0, 0.0],
            [expected.x, expected.y, 0.0, 0.0],
        );
    }
    assert_eq!(
        MeshHost::from_handle(Some(image)).unwrap().uv_transform(),
        nuxie_render_api::Mat2D::IDENTITY
    );
}

#[test]
fn declared_picture_hit_area_uses_the_active_artboards_mode() {
    use crate::source::artboard::Artboard;
    for css in [false, true] {
        let arena = CoreArena::default();
        let root = arena.insert(Artboard::default());
        if css {
            let record = arena.insert(crate::css_layout::LayoutCssStyle::default());
            attach(&arena, &root, &record);
            root.with_mut(|root| {
                root.as_layout_component_mut()
                    .unwrap()
                    .add_layout_style_applier(record)
            });
        }
        let image = arena.insert(Image::default());
        attach(&arena, &root, &image);
        image.with_downcast_mut::<Image, _>(|image| {
            image.base.set_origin_x_value(0.0);
            image.base.set_origin_y_value(0.0);
        });
        crate::source::layout::picture_layout_mode::attached(image.clone());
        let asset = arena.insert(ImageAsset::default());
        Image::set_asset_occurrence(&image, Some(asset));
        declare(&image, 6000.0, 4000.0);
        decode(&image, 4096, 2731);
        let info = crate::source::hit_info::HitInfo {
            area: IAabb {
                left: 5500,
                top: 3500,
                right: 5502,
                bottom: 3502,
            },
            mounts: Vec::new(),
        };
        root.with_downcast_mut::<Artboard, _>(|_root| {
            assert_eq!(
                image.with_downcast::<Image, _>(|image| {
                    image.hit_test(&info, Mat2D::default()).is_some()
                }),
                Some(css)
            );
        });
    }
}

// Expected border boxes are captured by the adjacent local Chrome oracle.
// Chrome quantizes layout to 1/64 px; Taffy retains floating point precision.
#[test]
fn css_picture_chrome_sizing_table() {
    let table: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/css_picture_sizing_chrome.json"
    ))
    .unwrap();
    let pairs_u32 = |value: &serde_json::Value| -> Vec<(u16, u32)> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|pair| {
                (
                    pair[0].as_u64().unwrap() as u16,
                    pair[1].as_u64().unwrap() as u32,
                )
            })
            .collect()
    };
    let pairs_f32 = |value: &serde_json::Value| -> Vec<(u16, f32)> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|pair| {
                (
                    pair[0].as_u64().unwrap() as u16,
                    pair[1].as_f64().unwrap() as f32,
                )
            })
            .collect()
    };
    let mut failures = Vec::new();
    for row in table["cases"].as_array().unwrap() {
        if row["siblings"].is_array() {
            continue;
        }
        for wrapped in [false, true] {
            for video in [false, true] {
                // The no-natural-dimensions cases use Image's missing asset size.
                if video && row["natural"][0].as_f64() == Some(0.0) {
                    continue;
                }
                let (_arena, root, item, participant) = picture_layout_fixture_with_natural_size(
                    video,
                    wrapped,
                    &pairs_u32(&row["parent_enums"]),
                    &pairs_u32(&row["enums"]),
                    &pairs_f32(&row["numbers"]),
                    (
                        row["natural"][0].as_f64().unwrap() as f32,
                        row["natural"][1].as_f64().unwrap() as f32,
                    ),
                );
                let host = host(&participant);
                if !video && let Some(decode_size) = row["decode"].as_array() {
                    decode(
                        &host,
                        decode_size[0].as_u64().unwrap() as u32,
                        decode_size[1].as_u64().unwrap() as u32,
                    );
                }
                let record = root
                    .with_downcast::<LayoutComponent, _>(|root| root.css_record().unwrap())
                    .unwrap();
                CssTextLeaf::write(&record, &[], &[(61022, 1000.0)]);
                CssTextLeaf::write(&record, &[], &pairs_f32(&row["parent_numbers"]));
                // The HTML parent has an auto height unless its CSS sets one.
                // Do not supply the helper's default 800px native viewport.
                root.with_downcast_mut::<LayoutComponent, _>(|root| {
                    root.with_style_mut(|style| {
                        style.base.set_intrinsically_sized_value_value(true)
                    });
                });
                LayoutComponent::sync_style_occurrence(&root);
                LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
                let bounds = solved(&item);
                let expected = [
                    row["expected"][0].as_f64().unwrap() as f32,
                    row["expected"][1].as_f64().unwrap() as f32,
                ];
                if (bounds.width() - expected[0]).abs() > 0.02
                    || (bounds.height() - expected[1]).abs() > 0.02
                {
                    failures.push(format!(
                        "{} wrapped={wrapped} video={video}: {:?}, expected {expected:?}; {}",
                        row["name"],
                        [bounds.width(), bounds.height()],
                        row["rule"]
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn css_picture_chrome_sizing_table_resolver() {
    use taffy::{
        AvailableSpace, Size,
        tree::{LayoutInput, RunMode, SizingMode},
    };
    let table: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/css_picture_sizing_chrome.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for row in table["cases"].as_array().unwrap() {
        // Explicit zero minima exercise flex's distribution in the layout
        // table. Other flex rows also exercise the resolver's intrinsic query.
        if row["name"].as_str().unwrap().ends_with("-zero") || row["siblings"].is_array() {
            continue;
        }
        let enums: Vec<_> = row["enums"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (p[0].as_u64().unwrap() as u16, p[1].as_u64().unwrap() as u32))
            .collect();
        let numbers: Vec<_> = row["numbers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (p[0].as_u64().unwrap() as u16, p[1].as_f64().unwrap() as f32))
            .collect();
        let (_arena, _root, participant) =
            picture_participant_fixture(false, &[], &enums, &numbers);
        let style = participant
            .with_downcast::<LayoutParticipant, _>(|p| {
                p.native_layout_data().unwrap().style().clone()
            })
            .unwrap();
        let mut known = Size {
            width: row["known"][0].as_f64().map(|value| value as f32),
            height: row["known"][1].as_f64().map(|value| value as f32),
        };
        // Ask for the intrinsic main size before flex distributes space.
        match row["name"].as_str().unwrap() {
            "row-shrink" => known.width = None,
            "column-shrink" => known.height = None,
            _ => {}
        }
        let input = LayoutInput {
            run_mode: RunMode::ComputeSize,
            sizing_mode: if matches!(row["name"].as_str(), Some("row-shrink" | "column-shrink")) {
                SizingMode::ContentSize
            } else {
                SizingMode::InherentSize
            },
            known_dimensions: known,
            parent_size: Size {
                width: Some(1000.0),
                height: None,
            },
            available_space: Size {
                width: if row["query"] == "min" {
                    AvailableSpace::MinContent
                } else {
                    AvailableSpace::MaxContent
                },
                height: AvailableSpace::MaxContent,
            },
            ..LayoutInput::HIDDEN
        };
        for available in [
            AvailableSpace::MinContent,
            AvailableSpace::MaxContent,
            AvailableSpace::Definite(1000.0),
        ] {
            let input = LayoutInput {
                available_space: Size {
                    width: available,
                    height: available,
                },
                ..input
            };
            let output = crate::css_layout::replaced::resolve_replaced_size(
                &style,
                input,
                Size {
                    width: row["natural"][0].as_f64().unwrap() as f32,
                    height: row["natural"][1].as_f64().unwrap() as f32,
                },
            );
            let expected = [
                row["expected"][0].as_f64().unwrap() as f32,
                row["expected"][1].as_f64().unwrap() as f32,
            ];
            if (output.width - expected[0]).abs() > 0.02
                || (output.height - expected[1]).abs() > 0.02
            {
                failures.push(format!(
                    "{}: {:?}, expected {expected:?}; {}",
                    row["name"], output, row["rule"]
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn content_picture_fixture(
    video: bool,
    css: bool,
    nested: bool,
) -> (CoreArena, CoreHandle, CoreHandle) {
    let arena = CoreArena::default();
    let root = layout(&arena, None, Some(200.0), true);
    let parent = if nested {
        let container = arena.insert(crate::source::shapes::shape::Shape::default());
        attach(&arena, &root, &container);
        container
    } else {
        root.clone()
    };
    let image = if video {
        let asset = arena.insert(crate::video::VideoAsset::default());
        asset.with_mut(|asset| {
            asset.set_double(CoreField::DrawableAssetWidth, 480.0);
            asset.set_double(CoreField::DrawableAssetHeight, 270.0);
        });
        let video = arena.insert(crate::video::Video::default());
        assert_eq!(
            video.with_mut(|video| video.file_asset_referencer_set_asset(asset)),
            Some(true)
        );
        video
    } else {
        arena.insert(Image::default())
    };
    attach(&arena, &parent, &image);
    if !video {
        Image::set_asset_occurrence(&image, Some(arena.insert(ImageAsset::default())));
        declare(&image, 480.0, 270.0);
        decode(&image, 160, 90);
    }
    if css {
        let record = arena.insert(crate::css_layout::LayoutCssStyle::default());
        attach(&arena, &root, &record);
        CssTextLeaf::write(&record, &[(61021, 1)], &[(61022, 200.0)]);
        root.with_downcast_mut::<LayoutComponent, _>(|root| root.add_layout_style_applier(record));
    }
    // Mirror the load boundary after all children and records are attached.
    crate::source::layout::picture_layout_mode::attached(image.clone());
    if nested {
        crate::source::layout::picture_layout_mode::attached(parent);
    }
    LayoutComponent::sync_style_occurrence(&root);
    LayoutComponent::sync_layout_children_occurrence(&root);
    (arena, root, image)
}

fn picture_width(host: &CoreHandle) -> f32 {
    let image =
        crate::source::shapes::mesh_host::MeshHost::from_handle(Some(host.clone())).unwrap();
    image.width() * image.scale_x()
}

#[test]
fn picture_content_receiver_measure_does_not_reborrow_owner() {
    for css in [false, true] {
        let (_arena, root, _image) = content_picture_fixture(false, css, false);
        let measured = root
            .with_downcast_mut::<LayoutComponent, _>(|root| {
                root.measure_layout(
                    f32::NAN,
                    LayoutMeasureMode::Undefined,
                    f32::NAN,
                    LayoutMeasureMode::Undefined,
                )
            })
            .unwrap();
        let expected = if css { (480.0, 270.0) } else { (160.0, 90.0) };
        near(measured.x, expected.0);
        near(measured.y, expected.1);
    }
}

#[test]
fn picture_participant_mode_is_independent_of_ancestor_mode() {
    let (_arena, _root, _participant, image) =
        receiver_picture_fixture_with_parent_mode(false, true);
    near(
        image
            .with_downcast::<Image, _>(|image| image.geometry_size().x)
            .unwrap(),
        160.0,
    );
}

#[test]
fn picture_nested_box_keeps_its_own_mode() {
    let (arena, root, _outer_image) = content_picture_fixture(false, true, false);
    let nested = layout(&arena, Some(&root), Some(200.0), true);
    let image = arena.insert(Image::default());
    attach(&arena, &nested, &image);
    Image::set_asset_occurrence(&image, Some(arena.insert(ImageAsset::default())));
    declare(&image, 480.0, 270.0);
    decode(&image, 160, 90);
    LayoutComponent::sync_style_occurrence(&root);
    crate::source::layout::picture_layout_mode::attached(image);
    let size = nested
        .with_downcast_mut::<LayoutComponent, _>(|nested| {
            nested.measure_layout(
                f32::NAN,
                LayoutMeasureMode::Undefined,
                f32::NAN,
                LayoutMeasureMode::Undefined,
            )
        })
        .unwrap();
    near(size.x, 160.0);
    near(size.y, 90.0);
}

#[test]
fn picture_content_receiver_bounds_does_not_reborrow_owner() {
    for video in [false, true] {
        for nested in [false, true] {
            for css in [false, true] {
                for receiver in [true, false] {
                    let (_arena, root, image) = content_picture_fixture(video, css, nested);
                    LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
                    if receiver {
                        root.with_downcast_mut::<LayoutComponent, _>(|root| {
                            root.update_layout_bounds(false)
                        });
                    } else {
                        LayoutComponent::update_layout_bounds_occurrence(&root, false);
                    }
                    near(picture_width(&image), if css { 480.0 } else { 200.0 });
                }
            }
        }
    }
}

#[test]
fn picture_content_receiver_animation_does_not_reborrow_owner() {
    use crate::source::layout::layout_enums::{LayoutDirection, LayoutStyleInterpolation};
    for video in [false, true] {
        for nested in [false, true] {
            for css in [false, true] {
                for receiver in [true, false] {
                    let (_arena, root, image) = content_picture_fixture(video, css, nested);
                    root.with_downcast_mut::<LayoutComponent, _>(|root| {
                        root.with_style_mut(|style| {
                            style.base.set_animation_style_type_value(
                                crate::source::layout::layout_enums::LayoutAnimationStyle::Custom
                                    as u8,
                            );
                            style.base.set_interpolation_type_value(
                                LayoutStyleInterpolation::Linear as u8,
                            );
                            style.base.set_interpolation_time_value(1.0);
                        });
                    });
                    LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
                    LayoutComponent::update_layout_bounds_occurrence(&root, false);
                    root.with_downcast_mut::<LayoutComponent, _>(|root| {
                        root.cascade_layout_style(
                            LayoutStyleInterpolation::Linear,
                            None,
                            1.0,
                            LayoutDirection::Ltr,
                        );
                        root.base.set_width_value(300.0);
                    });
                    if css {
                        let record = root
                            .with_downcast::<LayoutComponent, _>(|root| root.css_record().unwrap())
                            .unwrap();
                        CssTextLeaf::write(&record, &[], &[(61022, 300.0)]);
                    }
                    LayoutComponent::sync_style_occurrence(&root);
                    LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
                    LayoutComponent::update_layout_bounds_occurrence(&root, true);
                    for _ in 0..3 {
                        if receiver {
                            root.with_downcast_mut::<LayoutComponent, _>(|root| {
                                root.apply_interpolation(0.5, true)
                            });
                        } else {
                            LayoutComponent::apply_interpolation_occurrence(&root, 0.5, true);
                        }
                    }
                    near(picture_width(&image), if css { 480.0 } else { 300.0 });
                }
            }
        }
    }
}

#[test]
fn css_picture_chrome_flex_basis_table() {
    let table: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/css_picture_sizing_chrome.json"
    ))
    .unwrap();
    let enums = |v: &serde_json::Value| -> Vec<(u16, u32)> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|p| (p[0].as_u64().unwrap() as u16, p[1].as_u64().unwrap() as u32))
            .collect()
    };
    let numbers = |v: &serde_json::Value| -> Vec<(u16, f32)> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|p| (p[0].as_u64().unwrap() as u16, p[1].as_f64().unwrap() as f32))
            .collect()
    };
    let mut failures = Vec::new();
    for row in table["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["siblings"].is_array())
    {
        for wrapped in [false, true] {
            for video in [false, true] {
                let (arena, root, first, _) = picture_layout_fixture_with_natural_size(
                    video,
                    wrapped,
                    &enums(&row["parent_enums"]),
                    &enums(&row["enums"]),
                    &numbers(&row["numbers"]),
                    (100.0, 100.0),
                );
                let sibling = &row["siblings"][0];
                let (second, _) = picture_layout_child(
                    &arena,
                    &root,
                    video,
                    wrapped,
                    &enums(&sibling["enums"]),
                    &numbers(&sibling["numbers"]),
                    (100.0, 100.0),
                );
                let record = root
                    .with_downcast::<LayoutComponent, _>(|root| root.css_record().unwrap())
                    .unwrap();
                CssTextLeaf::write(&record, &[], &numbers(&row["parent_numbers"]));
                root.with_downcast_mut::<LayoutComponent, _>(|root| {
                    root.with_style_mut(|style| {
                        style.base.set_intrinsically_sized_value_value(true)
                    })
                });
                LayoutComponent::sync_style_occurrence(&root);
                LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
                for (item, expected) in
                    [(&first, &row["expected"]), (&second, &sibling["expected"])]
                {
                    let actual = solved(item);
                    if (actual.width() - expected[0].as_f64().unwrap() as f32).abs() > 0.02
                        || (actual.height() - expected[1].as_f64().unwrap() as f32).abs() > 0.02
                    {
                        failures.push(format!(
                            "{} wrapped={wrapped} video={video}: {actual:?}, expected {expected}",
                            row["name"]
                        ));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn css_picture_chrome_flex_basis_resolver() {
    use taffy::{
        Size,
        tree::{LayoutInput, RequestedAxis, RunMode, SizingMode},
    };
    let table: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/css_picture_sizing_chrome.json"
    ))
    .unwrap();
    for row in table["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["siblings"].is_array())
    {
        let baseline = &table["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|reference| reference["name"] == row["basis_reference"])
            .unwrap()["expected"];
        let enums: Vec<_> = row["enums"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (p[0].as_u64().unwrap() as u16, p[1].as_u64().unwrap() as u32))
            .collect();
        let numbers: Vec<_> = row["numbers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (p[0].as_u64().unwrap() as u16, p[1].as_f64().unwrap() as f32))
            .collect();
        let (_arena, _root, p) = picture_participant_fixture(false, &[], &enums, &numbers);
        let style = p
            .with_downcast::<LayoutParticipant, _>(|p| {
                p.native_layout_data().unwrap().style().clone()
            })
            .unwrap();
        let axis = if row["name"].as_str().unwrap().contains("column") {
            RequestedAxis::Vertical
        } else {
            RequestedAxis::Horizontal
        };
        let result = crate::css_layout::replaced::resolve_replaced_size(
            &style,
            LayoutInput {
                run_mode: RunMode::ComputeSize,
                sizing_mode: SizingMode::ContentSize,
                axis,
                ..LayoutInput::HIDDEN
            },
            Size {
                width: 100.0,
                height: 100.0,
            },
        );
        near(result.width, baseline[0].as_f64().unwrap() as f32);
        near(result.height, baseline[1].as_f64().unwrap() as f32);
    }
}
