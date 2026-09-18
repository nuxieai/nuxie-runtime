pub use super::semantic_clip::SemanticGeometryError;
use crate::mechanical_port::source::{
    artboard::Artboard,
    core::CoreHandle,
    generated::{container_component_base::ContainerComponentBase, node_base::NodeBase},
    math::{aabb::Aabb, vec2d::Vec2D},
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

pub fn root_transform_aabb(artboard: &CoreHandle, bounds: Bounds) -> Bounds {
    let mut transformed = Bounds::for_expansion();
    let points = [
        (bounds.min_x, bounds.min_y),
        (bounds.max_x, bounds.min_y),
        (bounds.max_x, bounds.max_y),
        (bounds.min_x, bounds.max_y),
    ];
    let mapped = artboard.with_downcast_mut::<Artboard, _>(|artboard| {
        points.map(|(x, y)| artboard.semantic_root_transform(Vec2D::new(x, y)))
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

fn source_ancestors_are_visible(component: &CoreHandle) -> bool {
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
        && geometry
            .into_iter()
            .all(|(owner, polygon)| clip_to_rendered_ancestors(&owner, polygon.to_vec()).is_empty())
}

fn clip_to_rendered_ancestors(component: &CoreHandle, polygon: Vec<Vec2D>) -> SemanticClipRegion {
    let mut region = SemanticClipRegion::from_polygon(&polygon);
    let mut current = Some(component.clone());
    let mut visited = std::collections::HashSet::new();
    let mut applied_shapes = std::collections::HashSet::new();
    while let Some(owner) = current {
        if !visited.insert(owner.clone()) {
            return SemanticClipRegion::default();
        }
        let Some((clip, parent)) = owner.with_mut(|object| {
            let is_artboard = object.as_artboard().is_some();
            let parent = object
                .component_parent_handle()
                .or_else(|| object.as_artboard().and_then(Artboard::host));
            let fallback = if let Some(artboard) = object.as_artboard() {
                artboard
                    .clip()
                    .then(|| (bounds_corners(artboard.bounds()), Some(owner.clone())))
            } else {
                object
                    .as_layout_component()
                    .filter(|layout| layout.base.clip())
                    .map(|layout| {
                        (
                            bounds_corners(layout.local_bounds())
                                .map(|point| layout.shape_world_transform() * point),
                            layout.artboard_handle(),
                        )
                    })
            };
            let clip = fallback.map(|(corners, artboard)| {
                let layout = object.as_layout_component_mut().expect("layout clip owner");
                let path = if is_artboard {
                    layout.local_path()
                } else {
                    layout.world_path()
                };
                let path = path.map(|path| (path.raw_path().clone(), path.fill_rule()));
                (corners, artboard, path)
            });
            (clip, parent)
        }) else {
            return SemanticClipRegion::default();
        };
        let shapes = owner
            .with(|object| {
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
                return SemanticClipRegion::default();
            };
            let Some((artboard, path)) = clip else {
                continue;
            };
            // A visible clip without a path suppresses drawing altogether.
            let Some((mut path, rule)) = path else {
                return SemanticClipRegion::default();
            };
            if let Some(artboard) = artboard {
                let Some(mapped) = artboard.with_downcast_mut::<Artboard, _>(|artboard| {
                    path.morph(|point| artboard.semantic_root_transform(point))
                }) else {
                    return SemanticClipRegion::default();
                };
                path = mapped;
            }
            region.intersect_path(&path, rule);
            if region.is_empty() {
                return region;
            }
        }
        if let Some((mut corners, artboard, mut path)) = clip {
            if let Some(artboard) = artboard {
                let Some((mapped, mapped_path)) =
                    artboard.with_downcast_mut::<Artboard, _>(|artboard| {
                        let corners = corners.map(|point| artboard.semantic_root_transform(point));
                        let path = path.map(|(path, rule)| {
                            (
                                path.morph(|point| artboard.semantic_root_transform(point)),
                                rule,
                            )
                        });
                        (corners, path)
                    })
                else {
                    return SemanticClipRegion::default();
                };
                corners = mapped;
                path = mapped_path;
            }
            if let Some((path, rule)) = path {
                region.intersect_path(&path, rule);
            } else {
                region.intersect_polygon(&corners);
            }
            if region.is_empty() {
                return region;
            }
        }
        current = parent;
    }
    region
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

pub fn semantic_bounds(component: Option<&CoreHandle>) -> Bounds {
    let Some(component) = component else {
        return Bounds::default();
    };
    let geometry = semantic_geometry(component);
    if !geometry.is_empty() {
        let mut result = Bounds::for_expansion();
        for (owner, polygon) in geometry {
            let visible = clip_to_rendered_ancestors(&owner, polygon.to_vec());
            if !visible.is_empty() {
                let bounds = visible.bounds();
                result.expand((bounds.min_x, bounds.min_y));
                result.expand((bounds.max_x, bounds.max_y));
            }
        }
        return result;
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
    let point = Bounds {
        min_x: point.x,
        min_y: point.y,
        max_x: point.x,
        max_y: point.y,
    };
    artboard
        .as_ref()
        .map_or(point, |artboard| root_transform_aabb(artboard, point))
}

/// Test rendered geometry against a root-artboard-space viewport, preserving
/// clip contours and holes rather than intersecting only their bounding boxes.
/// This does not depend on an image or video frame having been decoded.
pub fn rendered_geometry_intersects_viewport(
    component: &CoreHandle,
    viewport: Bounds,
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
        let mut region = clip_to_rendered_ancestors(&owner, polygon);
        region.status()?;
        region.intersect_polygon(&corners);
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
            *node.world_transform(),
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

/// Recalculate semantic bounds after a rendered clip changes without dirtying
/// unrelated text/paint dependencies in the clipped subtree.
pub(crate) fn invalidate_clipped_semantics(children: &[CoreHandle]) {
    let mut pending = children.to_vec();
    let mut visited = std::collections::HashSet::new();
    while let Some(handle) = pending.pop() {
        if !visited.insert(handle.clone()) {
            continue;
        }
        handle.with_mut(|object| {
            if object
                .as_any()
                .is::<crate::mechanical_port::source::semantic::semantic_data::SemanticData>()
            {
                object.component_add_dirt(
                    crate::mechanical_port::source::component_dirt::ComponentDirt::PATH,
                    false,
                );
            }
            append_semantic_children(object, &mut pending);
        });
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
                clip_to_rendered_ancestors(&owner, polygon.to_vec()).status()?;
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
