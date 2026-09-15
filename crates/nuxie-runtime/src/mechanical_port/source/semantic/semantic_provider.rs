use crate::mechanical_port::source::{
    artboard::Artboard,
    core::CoreHandle,
    generated::{container_component_base::ContainerComponentBase, node_base::NodeBase},
    math::{aabb::Aabb, vec2d::Vec2D},
    semantic::{
        semantic_inference_registry::{resolve_inferred_semantics, supports_inferred_semantics},
        semantic_snapshot::Bounds,
    },
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
    !semantic_is_fully_clipped(component)
}

fn semantic_is_fully_clipped(component: &CoreHandle) -> bool {
    let geometry = semantic_geometry(component);
    !geometry.is_empty()
        && geometry.into_iter().all(|(owner, polygon)| {
            !polygon_has_area(&clip_to_ancestor_layouts(&owner, polygon.to_vec()))
        })
}

fn clip_to_ancestor_layouts(component: &CoreHandle, mut polygon: Vec<Vec2D>) -> Vec<Vec2D> {
    let mut current = Some(component.clone());
    let mut visited = std::collections::HashSet::new();
    while let Some(owner) = current {
        if !visited.insert(owner.clone()) {
            return Vec::new();
        }
        let Some((clip, parent)) = owner.with(|object| {
            let artboard = object.as_artboard();
            let clip = if let Some(artboard) = artboard {
                artboard
                    .clip()
                    .then(|| (bounds_corners(artboard.bounds()), Some(owner.clone())))
            } else {
                object
                    .as_layout_component()
                    .filter(|layout| layout.base.clip())
                    .map(|layout| {
                        let transform = layout.shape_world_transform();
                        (
                            bounds_corners(layout.local_bounds()).map(|point| transform * point),
                            layout.artboard_handle(),
                        )
                    })
            };
            let parent = object
                .component_parent_handle()
                .or_else(|| artboard.and_then(Artboard::host));
            (clip, parent)
        }) else {
            return Vec::new();
        };
        if let Some((mut clip, artboard)) = clip {
            if let Some(artboard) = artboard {
                let Some(mapped) = artboard.with_downcast_mut::<Artboard, _>(|artboard| {
                    clip.map(|point| artboard.root_transform(point))
                }) else {
                    return Vec::new();
                };
                clip = mapped;
            }
            polygon = intersect_convex_clip(polygon, &clip);
            if polygon.is_empty() {
                return Vec::new();
            }
        }
        current = parent;
    }
    polygon
}

fn bounds_corners(bounds: Aabb) -> [Vec2D; 4] {
    [
        Vec2D::new(bounds.min_x, bounds.min_y),
        Vec2D::new(bounds.max_x, bounds.min_y),
        Vec2D::new(bounds.max_x, bounds.max_y),
        Vec2D::new(bounds.min_x, bounds.max_y),
    ]
}

// Keep the polygon between intersections: reducing a rotated clip to its AABB
// before intersecting the next clip can introduce area that was never visible.
fn intersect_convex_clip(mut polygon: Vec<Vec2D>, clip: &[Vec2D; 4]) -> Vec<Vec2D> {
    let cross =
        |a: Vec2D, b: Vec2D, p: Vec2D| (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
    let area = cross(clip[0], clip[1], clip[2]);
    if !area.is_finite() || area == 0.0 || clip.iter().any(|p| !p.x.is_finite() || !p.y.is_finite())
    {
        return Vec::new();
    }
    let orientation = area.signum();
    for index in 0..4 {
        let input = std::mem::take(&mut polygon);
        let Some(mut previous) = input.last().copied() else {
            break;
        };
        let a = clip[index];
        let b = clip[(index + 1) % 4];
        let mut previous_distance = cross(a, b, previous) * orientation;
        for point in input {
            let distance = cross(a, b, point) * orientation;
            if (distance >= 0.0) != (previous_distance >= 0.0) {
                let t = previous_distance / (previous_distance - distance);
                polygon.push(Vec2D::new(
                    previous.x + t * (point.x - previous.x),
                    previous.y + t * (point.y - previous.y),
                ));
            }
            if distance >= 0.0 {
                polygon.push(point);
            }
            previous = point;
            previous_distance = distance;
        }
    }
    polygon
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
            let visible = clip_to_ancestor_layouts(&owner, polygon.to_vec());
            if polygon_has_area(&visible) {
                for point in visible {
                    result.expand((point.x, point.y));
                }
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
            polygon.map(|point| artboard.root_transform(point))
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

#[cfg(test)]
mod clipping_tests {
    use super::*;

    #[test]
    fn rotated_clip_rejects_its_empty_aabb_corners_in_either_winding() {
        let mut clip = [
            Vec2D::new(0.0, 1.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(2.0, 1.0),
            Vec2D::new(1.0, 2.0),
        ];
        let corner = vec![
            Vec2D::new(0.0, 0.0),
            Vec2D::new(0.25, 0.0),
            Vec2D::new(0.25, 0.25),
            Vec2D::new(0.0, 0.25),
        ];
        assert!(intersect_convex_clip(corner.clone(), &clip).is_empty());
        clip.reverse();
        assert!(intersect_convex_clip(corner, &clip).is_empty());
    }

    #[test]
    fn rotated_clip_preserves_only_the_visible_triangle() {
        let clip = [
            Vec2D::new(0.0, 1.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(2.0, 1.0),
            Vec2D::new(1.0, 2.0),
        ];
        let square = vec![
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(0.0, 1.0),
        ];
        let polygon = intersect_convex_clip(square, &clip);
        let twice_area: f32 = polygon
            .iter()
            .zip(polygon.iter().cycle().skip(1))
            .take(polygon.len())
            .map(|(a, b)| a.x * b.y - a.y * b.x)
            .sum();
        assert!((twice_area.abs() - 1.0).abs() < 0.0001);
        assert!(
            polygon
                .iter()
                .all(|p| p.x + p.y >= 1.0 && p.x <= 1.0 && p.y <= 1.0)
        );
    }
}
