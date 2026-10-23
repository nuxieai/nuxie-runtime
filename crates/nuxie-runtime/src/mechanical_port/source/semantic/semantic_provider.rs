pub use super::semantic_clip::SemanticGeometryError;
use crate::mechanical_port::source::{
    artboard::Artboard,
    artboard_component_list::ArtboardComponentList,
    core::CoreHandle,
    generated::{container_component_base::ContainerComponentBase, node_base::NodeBase},
    math::{aabb::Aabb, mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
    semantic::{
        semantic_clip::SemanticClipRegion,
        semantic_inference_registry::{resolve_inferred_semantics, supports_inferred_semantics},
        semantic_snapshot::Bounds,
    },
    shapes::clipping_shape::ClippingShape,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolvedSemanticData {
    pub has_semantics: bool,
    pub role: u32,
    pub label: String,
}

impl Bounds {
    pub fn for_expansion() -> Self {
        Self {
            min_x: f32::MAX,
            min_y: f32::MAX,
            max_x: -f32::MAX,
            max_y: -f32::MAX,
        }
    }

    pub fn is_empty_or_nan(self) -> bool {
        !(self.max_x - self.min_x > 0.0 && self.max_y - self.min_y > 0.0)
    }

    pub fn expand(&mut self, point: (f32, f32)) {
        self.min_x = if point.0 < self.min_x {
            point.0
        } else {
            self.min_x
        };
        self.min_y = if point.1 < self.min_y {
            point.1
        } else {
            self.min_y
        };
        self.max_x = if self.max_x < point.0 {
            point.0
        } else {
            self.max_x
        };
        self.max_y = if self.max_y < point.1 {
            point.1
        } else {
            self.max_y
        };
    }
}

/// Map occurrence-local geometry using the same transforms as semantic drawing.
/// Exposes the existing mapping for native host overlays without duplicating
/// nested-artboard, origin, or list-layout traversal in each platform adapter.
pub fn root_transform_point(artboard: &CoreHandle, point: Vec2D) -> Option<Vec2D> {
    artboard.with_downcast_mut::<Artboard, _>(|artboard| artboard.semantic_root_transform(point))
}

pub fn root_transform_aabb(artboard: &CoreHandle, bounds: Bounds) -> Bounds {
    let mut transformed = Bounds::for_expansion();
    let points = [
        (bounds.min_x, bounds.min_y),
        (bounds.max_x, bounds.min_y),
        (bounds.max_x, bounds.max_y),
        (bounds.min_x, bounds.max_y),
    ];
    let mapped = artboard.with_downcast_mut::<Artboard, _>(|artboard| {
        points.map(|(x, y)| artboard.root_transform(Vec2D::new(x, y)))
    });
    let Some(mapped) = mapped else {
        return bounds;
    };
    for point in mapped {
        transformed.expand((point.x, point.y));
    }
    transformed
}

pub fn can_infer_semantics(component: Option<&CoreHandle>) -> bool {
    supports_inferred_semantics(component)
}

pub fn resolve_semantic_data(component: Option<&CoreHandle>) -> ResolvedSemanticData {
    let Some(component) = component else {
        return ResolvedSemanticData::default();
    };
    if !component.is_type_of(NodeBase::TYPE_KEY) {
        return ResolvedSemanticData::default();
    }
    let explicit = component
        .with(|component| {
            let node = component.as_node()?;
            node.children().iter().find_map(|child| {
                child
                    .with(|child| {
                        child
                            .as_semantic_data()
                            .map(|semantic_data| ResolvedSemanticData {
                                has_semantics: true,
                                role: semantic_data.base.role(),
                                label: semantic_data.base.label().to_owned(),
                            })
                    })
                    .flatten()
            })
        })
        .flatten();
    if let Some(explicit) = explicit {
        return explicit;
    }
    let mut inferred = ResolvedSemanticData::default();
    resolve_inferred_semantics(Some(component), &mut inferred);
    inferred
}

/// Authored visibility is checked again at action dispatch, before a pending
/// opacity update has necessarily propagated into the rendered component tree.
pub fn semantic_source_is_visible(component: &CoreHandle) -> bool {
    source_ancestors_are_visible(component) && !semantic_is_fully_clipped(component)
}

pub(crate) fn source_ancestors_are_visible(component: &CoreHandle) -> bool {
    let mut current = Some(component.clone());
    let mut visited = std::collections::HashSet::new();
    while let Some(component) = current {
        if !visited.insert(component.clone()) {
            return false;
        }
        let Some((visible, parent)) = component.with(|component| {
            let visible = !component.component_is_collapsed()
                && !(component.as_drawable().is_some() && component.drawable_is_hidden())
                && component
                    .as_world_transform_component()
                    .is_none_or(|transform| {
                        let opacity = transform.opacity();
                        opacity.is_finite() && opacity > 0.0
                    });
            let parent = component
                .component_parent_handle()
                .or_else(|| component.as_artboard().and_then(Artboard::host));
            (visible, parent)
        }) else {
            return false;
        };
        if !visible {
            return false;
        }
        current = parent;
    }
    true
}

fn semantic_is_fully_clipped(component: &CoreHandle) -> bool {
    let geometry = semantic_geometry(component);
    !geometry.is_empty()
        && geometry.into_iter().all(|(owner, polygon)| {
            !geometry_with_rendered_clips(&owner, &polygon, GeometryClipMode::HostQuery)
                .is_visible()
        })
}

#[derive(Clone, Copy)]
enum GeometryClipMode {
    /// Picking, catalogues and selection preserve translated native hits.
    HostQuery,
    /// Drawing visibility includes every rendered artboard frame clip.
    Drawing,
}

fn geometry_with_rendered_clips(
    component: &CoreHandle,
    polygon: &[Vec2D],
    mode: GeometryClipMode,
) -> RenderedGeometry {
    let mut geometry = RenderedGeometry::unclipped(polygon);
    let complete = walk_rendered_clips(component, mode, true, |artboard, clip| {
        let clip = if let Some(artboard) = artboard {
            let Some(clip) = artboard.with_downcast_mut::<Artboard, _>(|artboard| {
                clip.mapped(|point| artboard.semantic_root_transform(point))
            }) else {
                return false;
            };
            clip
        } else {
            clip
        };
        geometry.intersect(clip)
    });
    geometry.finish(complete)
}

/// Full bounds for outlines and semantic frames, the visible intersection,
/// and normalized clips for point tests. All come from one clip walk.
pub(crate) struct RenderedGeometry {
    pub bounds: Bounds,
    visible_region: SemanticClipRegion,
    clips: Vec<GeometryClip>,
}

impl RenderedGeometry {
    pub(crate) fn unclipped(polygon: &[Vec2D]) -> Self {
        let mut bounds = Bounds::for_expansion();
        for point in polygon {
            bounds.expand((point.x, point.y));
        }
        Self {
            bounds,
            visible_region: SemanticClipRegion::from_polygon(polygon),
            clips: Vec::new(),
        }
    }

    fn intersect(&mut self, clip: RenderedClip) -> bool {
        let clip = clip.into_geometry_clip();
        self.visible_region.intersect_region(&clip.region);
        self.clips.push(clip);
        self.is_visible()
    }

    fn finish(mut self, complete: bool) -> Self {
        if !complete && self.is_visible() {
            self.visible_region = SemanticClipRegion::default();
        }
        self
    }

    pub(crate) fn is_visible(&self) -> bool {
        !self.visible_region.is_empty()
    }

    pub(crate) fn contains_point(&self, point: Vec2D) -> bool {
        // Text selection uses its inclusive transformed AABB, as before the
        // clip walk. Its empty rotated corners are not additional clips.
        point.x >= self.bounds.min_x
            && point.x <= self.bounds.max_x
            && point.y >= self.bounds.min_y
            && point.y <= self.bounds.max_y
            && self.is_visible()
            && self.clips.iter().all(|clip| clip.contains_point(point))
    }
}

struct GeometryClip {
    region: SemanticClipRegion,
    frame: Option<[Vec2D; 4]>,
}

impl GeometryClip {
    fn contains_point(&self, point: Vec2D) -> bool {
        if let Some(corners) = self.frame {
            // Solve in the frame's unit square so right and bottom stay
            // exclusive through origins, rotations and hosting transforms.
            let delta = |a: Vec2D, b: Vec2D| {
                [
                    f64::from(a.x) - f64::from(b.x),
                    f64::from(a.y) - f64::from(b.y),
                ]
            };
            let cross = |a: [f64; 2], b: [f64; 2]| a[0] * b[1] - a[1] * b[0];
            let x = delta(corners[1], corners[0]);
            let y = delta(corners[3], corners[0]);
            let p = delta(point, corners[0]);
            let determinant = cross(x, y);
            if !(0.0..1.0).contains(&(cross(p, y) / determinant))
                || !(0.0..1.0).contains(&(cross(x, p) / determinant))
            {
                return false;
            }
        }
        self.region.contains(point)
    }
}

/// A clip in its owning artboard's drawing space. Both host geometry
/// consumers use the same paths and fill rules as rendered semantics.
enum RenderedClip {
    Path(RawPath, nuxie_render_api::FillRule),
    Polygon([Vec2D; 4]),
    Frame {
        corners: [Vec2D; 4],
        path: Option<(RawPath, nuxie_render_api::FillRule)>,
    },
}

impl RenderedClip {
    fn mapped(self, mut map: impl FnMut(Vec2D) -> Vec2D) -> Self {
        match self {
            Self::Path(path, rule) => Self::Path(path.morph(map), rule),
            Self::Polygon(corners) => Self::Polygon(corners.map(&mut map)),
            Self::Frame { corners, path } => Self::Frame {
                corners: corners.map(&mut map),
                path: path.map(|(path, rule)| (path.morph(map), rule)),
            },
        }
    }

    fn into_geometry_clip(self) -> GeometryClip {
        match self {
            Self::Path(path, rule) => GeometryClip {
                region: SemanticClipRegion::from_path(&path, rule),
                frame: None,
            },
            Self::Polygon(corners) => GeometryClip {
                region: SemanticClipRegion::from_polygon(&corners),
                frame: None,
            },
            Self::Frame { corners, path } => GeometryClip {
                region: path.map_or_else(
                    || SemanticClipRegion::from_polygon(&corners),
                    |(path, rule)| SemanticClipRegion::from_path(&path, rule),
                ),
                frame: Some(corners),
            },
        }
    }
}

/// Visit clips through each nested artboard's host, mapping them with the
/// same occurrence transforms the host geometry catalogue draws with.
fn walk_geometry_clips(
    component: &CoreHandle,
    spaces: &[(CoreHandle, Mat2D)],
    mut visit: impl FnMut(RenderedClip) -> bool,
) -> bool {
    let mut hosts = None;
    walk_rendered_clips(
        component,
        GeometryClipMode::HostQuery,
        false,
        |artboard, clip| {
            let clip = if let Some(artboard) = artboard {
                let find = |spaces: &[(CoreHandle, Mat2D)]| {
                    spaces
                        .iter()
                        .find(|(owner, _)| owner == artboard)
                        .map(|(_, transform)| *transform)
                };
                let Some(transform) =
                    find(spaces).or_else(|| find(hosts.get_or_insert_with(|| host_spaces(spaces))))
                else {
                    return false;
                };
                clip.mapped(|point| transform * point)
            } else {
                clip
            };
            visit(clip)
        },
    )
}

/// The drawing spaces of the artboards hosting the query's root (the first
/// of `spaces`), in the query's root space. A query of a hosted artboard
/// climbs into them for their clips. Drawing maps a hosted artboard's space
/// into its host's by the host's transform for it, so each host's space is
/// reached through that transform's inverse.
fn host_spaces(spaces: &[(CoreHandle, Mat2D)]) -> Vec<(CoreHandle, Mat2D)> {
    let mut hosts = Vec::new();
    let Some((mut child, mut child_drawing)) = spaces.first().cloned() else {
        return hosts;
    };
    loop {
        let Some((host, frame, weak)) = child
            .with(|object| {
                let artboard = object.as_artboard()?;
                let frame_offset = if artboard.frame_origin() {
                    Vec2D::new(
                        artboard.layout_width() * artboard.origin_x(),
                        artboard.layout_height() * artboard.origin_y(),
                    )
                } else {
                    Vec2D::default()
                };
                Some((
                    artboard.host()?,
                    Mat2D::from_translation(frame_offset) * artboard.self_transform(),
                    artboard.runtime_weak_handle(),
                ))
            })
            .flatten()
        else {
            break;
        };
        let Some((parent, host_world)) = host
            .with(|object| {
                let world =
                    if let Some(list) = object.as_any().downcast_ref::<ArtboardComponentList>() {
                        list.draw_transform_for_artboard(weak)?
                    } else {
                        object
                            .as_artboard_host()?
                            .world_transform_for_artboard(weak)
                    };
                Some((object.component_artboard_handle()?, world))
            })
            .flatten()
        else {
            break;
        };
        let mut frame_inverse = Mat2D::default();
        let mut world_inverse = Mat2D::default();
        if !frame.invert(&mut frame_inverse) || !host_world.invert(&mut world_inverse) {
            break;
        }
        let parent_drawing = child_drawing * frame_inverse * world_inverse;
        if spaces
            .iter()
            .chain(&hosts)
            .any(|(owner, _)| owner == &parent)
        {
            break;
        }
        hosts.push((parent.clone(), parent_drawing));
        child = parent;
        child_drawing = parent_drawing;
    }
    hosts
}

/// A full host geometry polygon and its visible region in root space.
/// Its own content clip is left out; CSS artboard frame clips stay.
pub(crate) fn rendered_geometry_projection(
    component: &CoreHandle,
    polygon: [Vec2D; 4],
    spaces: &[(CoreHandle, Mat2D)],
) -> RenderedGeometry {
    let mut geometry = RenderedGeometry::unclipped(&polygon);
    let complete = walk_geometry_clips(component, spaces, |clip| geometry.intersect(clip));
    geometry.finish(complete)
}

/// Yield each rendered clip once, crossing hosts into their parent
/// artboards. Own paint precedes the owner's content clip, but stays
/// inside any clipping shapes that cover it. The caller chooses whether
/// translated artboard draw frames apply, independently of CSS frame clips.
fn walk_rendered_clips(
    component: &CoreHandle,
    mode: GeometryClipMode,
    own_clip: bool,
    mut visit: impl FnMut(Option<&CoreHandle>, RenderedClip) -> bool,
) -> bool {
    let mut current = Some(component.clone());
    let mut visited = std::collections::HashSet::new();
    let mut applied_shapes = std::collections::HashSet::new();
    while let Some(owner) = current {
        if !visited.insert(owner.clone()) {
            return false;
        }
        let skip_own = !own_clip && &owner == component;
        let Some((clip, parent, css_artboard_clip)) = owner.with_mut(|object| {
            let is_artboard = object.as_artboard().is_some();
            // A CSS artboard's record clips its content at its padding box,
            // inside its own clip (docs/CSS_LAYOUT.md, 4.2).
            let css_artboard_clip = object
                .as_artboard()
                .filter(|artboard| artboard.css_clips())
                .and_then(|artboard| artboard.base.base.render_paths_handle())
                .map(|paths| {
                    let paths = paths.borrow();
                    (paths.clip.raw_path().clone(), paths.clip.fill_rule())
                });
            let parent = object
                .component_parent_handle()
                .or_else(|| object.as_artboard().and_then(Artboard::host));
            let fallback = if let Some(artboard) = object.as_artboard() {
                (artboard.clip()
                    && (matches!(mode, GeometryClipMode::Drawing)
                        || artboard.base.base.is_css_mode()))
                .then(|| {
                    let bounds = Aabb::from_ltwh(
                        -artboard.layout_width() * artboard.origin_x(),
                        -artboard.layout_height() * artboard.origin_y(),
                        artboard.layout_width(),
                        artboard.layout_height(),
                    );
                    (bounds_corners(bounds), Some(owner.clone()))
                })
            } else if skip_own {
                None
            } else {
                // A CSS box clips by its computed overflow, at its padding
                // box (docs/CSS_LAYOUT.md, 4.2).
                object
                    .as_layout_component()
                    .filter(|layout| layout.clips())
                    .map(|layout| {
                        (
                            bounds_corners(layout.clip_local_bounds())
                                .map(|point| layout.shape_world_transform() * point),
                            layout.artboard_handle(),
                        )
                    })
            };
            let clip = fallback.map(|(corners, artboard)| {
                let layout = object.as_layout_component_mut().expect("layout clip owner");
                let path = if is_artboard {
                    layout.frame_clip_path()
                } else {
                    layout.clip_world_path()
                };
                let path = path.map(|path| (path.raw_path().clone(), path.fill_rule()));
                let clip = if is_artboard {
                    RenderedClip::Frame { corners, path }
                } else {
                    path.map_or_else(
                        || RenderedClip::Polygon(corners),
                        |(path, rule)| RenderedClip::Path(path, rule),
                    )
                };
                (artboard, clip)
            });
            (clip, parent, css_artboard_clip)
        }) else {
            return false;
        };
        if let Some((path, rule)) = css_artboard_clip.filter(|_| !skip_own) {
            if !visit(Some(&owner), RenderedClip::Path(path, rule)) {
                return false;
            }
        }
        let shapes = owner
            .with(|object| {
                // Artboard own paint precedes the drawable list's shape
                // clips. They still cover content and nested hosts.
                if skip_own && object.as_artboard().is_some() {
                    return None;
                }
                object
                    .as_drawable()
                    .map(|drawable| drawable.clipping_shapes().to_vec())
            })
            .flatten()
            .unwrap_or_default();
        for shape in shapes {
            if !applied_shapes.insert(shape.clone()) {
                continue;
            }
            let Some(clip) = shape.with_downcast_mut::<ClippingShape, _>(|shape| {
                if !shape.base.is_visible() {
                    return None;
                }
                let artboard = shape.base.artboard_handle();
                let path = shape
                    .path()
                    .map(|path| (path.raw_path().clone(), path.fill_rule()));
                Some((artboard, path))
            }) else {
                return false;
            };
            let Some((artboard, path)) = clip else {
                continue;
            };
            // A visible clip without a path suppresses drawing altogether.
            let Some((path, rule)) = path else {
                return false;
            };
            if !visit(artboard.as_ref(), RenderedClip::Path(path, rule)) {
                return false;
            }
        }
        if let Some((artboard, clip)) = clip {
            if !visit(artboard.as_ref(), clip) {
                return false;
            }
        }
        current = parent;
    }
    true
}

fn bounds_corners(bounds: Aabb) -> [Vec2D; 4] {
    [
        Vec2D::new(bounds.min_x, bounds.min_y),
        Vec2D::new(bounds.max_x, bounds.min_y),
        Vec2D::new(bounds.max_x, bounds.max_y),
        Vec2D::new(bounds.min_x, bounds.max_y),
    ]
}

fn polygon_has_area(polygon: &[Vec2D]) -> bool {
    let twice_area: f32 = polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
        .map(|(a, b)| a.x * b.y - a.y * b.x)
        .sum();
    twice_area.is_finite() && twice_area != 0.0
}

fn try_node_world_bounds(component: &CoreHandle) -> Option<Bounds> {
    // Upstream filters non-Node descendants before reading their bounds.
    // In Rust this also avoids borrowing the active SemanticData callback owner.
    if !component.is_type_of(NodeBase::TYPE_KEY) {
        return None;
    }
    component.with(|object| {
        let node = object.as_node()?;
        let local = object.semantic_provider_local_bounds()?;
        if local.is_empty_or_nan() {
            return None;
        }
        let world = node.world_transform().map_bounding_box(local);
        if world.is_empty_or_nan() {
            return None;
        }
        Some(Bounds {
            min_x: world.min_x,
            min_y: world.min_y,
            max_x: world.max_x,
            max_y: world.max_y,
        })
    })?
}

/// Native upstream bounds in the outermost artboard's coordinate space.
/// Rendered clipping and host frame-origin projection are separate operations.
pub fn semantic_bounds(component: Option<&CoreHandle>) -> Bounds {
    let Some(component) = component.filter(|component| component.is_type_of(NodeBase::TYPE_KEY))
    else {
        return Bounds::default();
    };
    let (point, artboard) = component
        .with(|object| {
            let node = object.as_node().expect("Node owner");
            (
                Vec2D::new(node.world_transform()[4], node.world_transform()[5]),
                node.artboard_handle(),
            )
        })
        .expect("live Node owner");
    if let Some(bounds) = try_node_world_bounds(component) {
        return artboard
            .as_ref()
            .map_or(bounds, |artboard| root_transform_aabb(artboard, bounds));
    }
    if component.is_type_of(ContainerComponentBase::TYPE_KEY) {
        let mut merged = Bounds::for_expansion();
        let mut has_descendant_bounds = false;
        crate::mechanical_port::source::container_component::ContainerComponent::for_each_child(
            component,
            |child| {
                if let Some(bounds) = try_node_world_bounds(&child) {
                    merged.expand((bounds.min_x, bounds.min_y));
                    merged.expand((bounds.max_x, bounds.max_y));
                    has_descendant_bounds = true;
                }
                true
            },
        );
        if has_descendant_bounds {
            return artboard
                .as_ref()
                .map_or(merged, |artboard| root_transform_aabb(artboard, merged));
        }
    }
    let point = artboard.as_ref().map_or(point, |artboard| {
        artboard
            .with_downcast_mut::<Artboard, _>(|artboard| artboard.root_transform(point))
            .expect("live Artboard owner")
    });
    Bounds {
        min_x: point.x,
        min_y: point.y,
        max_x: point.x,
        max_y: point.y,
    }
}

/// Full bounds of visible host geometry, including frame origins. Rendered
/// clips decide visibility without trimming the reported box.
/// This is not the native SemanticProvider::semanticBounds contract.
pub fn rendered_geometry_bounds(component: Option<&CoreHandle>) -> Bounds {
    let Some(component) = component else {
        return Bounds::default();
    };
    let geometry = semantic_geometry(component);
    if !geometry.is_empty() {
        let mut result = Bounds::for_expansion();
        let mut visible = false;
        for (owner, polygon) in geometry {
            let geometry =
                geometry_with_rendered_clips(&owner, &polygon, GeometryClipMode::HostQuery);
            visible |= geometry.is_visible();
            let bounds = geometry.bounds;
            result.expand((bounds.min_x, bounds.min_y));
            result.expand((bounds.max_x, bounds.max_y));
        }
        return if visible {
            result
        } else {
            Bounds::for_expansion()
        };
    }
    // Nodes without rectangular geometry retain the existing point fallback.
    let Some((point, artboard)) = component
        .with(|object| {
            let node = object.as_node()?;
            Some((
                Vec2D::new(node.world_transform()[4], node.world_transform()[5]),
                node.artboard_handle(),
            ))
        })
        .flatten()
    else {
        return Bounds::default();
    };
    let bounds = Bounds {
        min_x: point.x,
        min_y: point.y,
        max_x: point.x,
        max_y: point.y,
    };
    let Some(artboard) = artboard else {
        return bounds;
    };
    let Some(points) = artboard.with_downcast_mut::<Artboard, _>(|artboard| {
        [point; 4].map(|point| artboard.semantic_root_transform(point))
    }) else {
        return bounds;
    };
    let mut result = Bounds::for_expansion();
    for point in points {
        result.expand((point.x, point.y));
    }
    result
}

/// Test rendered geometry against a root-artboard-space viewport, preserving
/// clip contours and holes rather than intersecting only their bounding boxes.
/// This does not depend on an image or video frame having been decoded.
pub fn rendered_geometry_intersects_viewport(
    component: &CoreHandle,
    viewport: Bounds,
) -> Result<bool, SemanticGeometryError> {
    rendered_geometry_intersects_host_clip(component, viewport, None)
}

/// Intersect scene geometry with an additional host-owned clip in root space.
/// The host must provide the same path and fill rule used while drawing.
/// This drawing query includes translated artboard frame clips, unlike picking
/// and catalogue queries.
pub fn rendered_geometry_intersects_host_clip(
    component: &CoreHandle,
    viewport: Bounds,
    host_clip: Option<(
        &crate::source::math::raw_path::RawPath,
        nuxie_render_api::FillRule,
    )>,
) -> Result<bool, SemanticGeometryError> {
    if ![
        viewport.min_x,
        viewport.min_y,
        viewport.max_x,
        viewport.max_y,
    ]
    .iter()
    .all(|value| value.is_finite())
    {
        return Err(SemanticGeometryError::InvalidPath);
    }
    if viewport.is_empty_or_nan() || !source_ancestors_are_visible(component) {
        return Ok(false);
    }
    let corners = [
        Vec2D::new(viewport.min_x, viewport.min_y),
        Vec2D::new(viewport.max_x, viewport.min_y),
        Vec2D::new(viewport.max_x, viewport.max_y),
        Vec2D::new(viewport.min_x, viewport.max_y),
    ];
    let mesh = component
        .with_downcast::<crate::video::Video, _>(|video| video.image().mesh())
        .flatten();
    let geometry = if let Some(mesh) = mesh {
        let triangles = mesh
            .with_downcast::<crate::source::shapes::mesh::Mesh, _>(|mesh| {
                Some((mesh.rendered_triangles()?, mesh.base.artboard_handle()?))
            })
            .flatten()
            .ok_or(SemanticGeometryError::InvalidPath)?;
        let (triangles, artboard) = triangles;
        artboard
            .with_downcast_mut::<Artboard, _>(|artboard| {
                triangles
                    .into_iter()
                    .map(|triangle| {
                        (
                            component.clone(),
                            triangle
                                .into_iter()
                                .map(|point| artboard.semantic_root_transform(point))
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .ok_or(SemanticGeometryError::InvalidPath)?
    } else {
        semantic_geometry(component)
            .into_iter()
            .map(|(owner, polygon)| (owner, polygon.to_vec()))
            .collect()
    };
    for (owner, polygon) in geometry {
        if !polygon
            .iter()
            .all(|point| point.x.is_finite() && point.y.is_finite())
        {
            return Err(SemanticGeometryError::InvalidPath);
        }
        let mut region = geometry_with_rendered_clips(&owner, &polygon, GeometryClipMode::Drawing)
            .visible_region;
        region.status()?;
        region.intersect_polygon(&corners);
        if let Some((path, fill_rule)) = host_clip {
            region.intersect_path(path, fill_rule);
            region.status()?;
        }
        if !region.is_empty() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn node_root_polygon(component: &CoreHandle) -> Option<[Vec2D; 4]> {
    if !component.is_type_of(NodeBase::TYPE_KEY) {
        return None;
    }
    let (local_bounds, transform, artboard) = component.with(|object| {
        let node = object.as_node()?;
        Some((
            object.semantic_provider_local_bounds()?,
            node.world_transform(),
            node.artboard_handle(),
        ))
    })??;
    if local_bounds.is_empty_or_nan() {
        return None;
    }
    let mut polygon = bounds_corners(local_bounds).map(|point| transform * point);
    if let Some(artboard) = artboard {
        polygon = artboard.with_downcast_mut::<Artboard, _>(|artboard| {
            polygon.map(|point| artboard.semantic_root_transform(point))
        })?;
    }
    polygon_has_area(&polygon).then_some(polygon)
}

/// Whether this node has a box of its own on screen.
pub(crate) fn node_has_area(component: &CoreHandle) -> bool {
    node_root_polygon(component).is_some()
}

fn semantic_geometry(component: &CoreHandle) -> Vec<(CoreHandle, [Vec2D; 4])> {
    if let Some(polygon) = node_root_polygon(component) {
        return vec![(component.clone(), polygon)];
    }
    let mut geometry = Vec::new();
    collect_descendant_geometry(component, &mut geometry);
    geometry
}

fn collect_descendant_geometry(
    component: &CoreHandle,
    geometry: &mut Vec<(CoreHandle, [Vec2D; 4])>,
) {
    if !component.is_type_of(ContainerComponentBase::TYPE_KEY) {
        return;
    }
    let children = component
        .with(|object| {
            object
                .as_container_component()
                .map(|container| container.children().to_vec())
        })
        .flatten()
        .unwrap_or_default();
    for child in children {
        if let Some(polygon) = node_root_polygon(&child) {
            geometry.push((child.clone(), polygon));
        }
        collect_descendant_geometry(&child, geometry);
    }
}

/// Reject incomplete geometry before exposing a semantic capture. Include
/// excluded nodes: an over-budget clip must not masquerade as a hidden control.
pub fn validate_semantic_geometry(root: &CoreHandle) -> Result<(), SemanticGeometryError> {
    let mut pending = vec![root.clone()];
    let mut visited = std::collections::HashSet::new();
    while let Some(handle) = pending.pop() {
        if !visited.insert(handle.clone()) {
            continue;
        }
        let parent = handle
            .with(|object| {
                append_semantic_children(object, &mut pending);
                object
                    .as_semantic_data()
                    .filter(|data| data.has_semantic_node())
                    .and_then(|_| object.component_parent_handle())
            })
            .flatten();
        if let Some(parent) = parent {
            for (owner, polygon) in semantic_geometry(&parent) {
                geometry_with_rendered_clips(&owner, &polygon, GeometryClipMode::HostQuery)
                    .visible_region
                    .status()?;
            }
        }
    }
    Ok(())
}

fn append_semantic_children(
    object: &dyn crate::mechanical_port::source::core::CoreObject,
    pending: &mut Vec<CoreHandle>,
) {
    if let Some(container) = object.as_container_component() {
        pending.extend_from_slice(container.children());
    }
    if let Some(host) = object.as_artboard_host() {
        for index in 0..host.artboard_count() {
            if let Some(instance) = host.artboard_instance(index as i32) {
                pending.push(instance.core_handle());
            }
        }
    }
}
