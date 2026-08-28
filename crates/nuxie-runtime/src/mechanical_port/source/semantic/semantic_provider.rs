use crate::mechanical_port::source::{
    artboard::Artboard,
    core::CoreHandle,
    math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
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

fn bounds_from_aabb(bounds: Aabb) -> Bounds {
    Bounds {
        min_x: bounds.min_x,
        min_y: bounds.min_y,
        max_x: bounds.max_x,
        max_y: bounds.max_y,
    }
}

fn aabb_from_bounds(bounds: Bounds) -> Aabb {
    Aabb::new(bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y)
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

fn node_world_bounds(component: &CoreHandle) -> Option<(Bounds, Option<CoreHandle>)> {
    let (local_bounds, world_transform, artboard) = component.with(|component| {
        let node = component.as_node()?;
        let local_bounds = component.semantic_provider_local_bounds()?;
        Some((
            local_bounds,
            *node.world_transform(),
            node.artboard_handle(),
        ))
    })??;
    if local_bounds.empty() {
        return None;
    }
    let world_bounds = world_transform.map_bounding_box(local_bounds);
    if world_bounds.empty() {
        return None;
    }
    Some((bounds_from_aabb(world_bounds), artboard))
}

fn collect_descendant_bounds(component: &CoreHandle, merged: &mut Aabb, found: &mut bool) {
    let children = component
        .with(|component| {
            component
                .as_container_component()
                .map(|container| container.children().to_vec())
        })
        .flatten()
        .unwrap_or_default();
    for child in children {
        if let Some((bounds, _)) = node_world_bounds(&child) {
            let bounds = aabb_from_bounds(bounds);
            if *found {
                merged.expand(bounds);
            } else {
                *merged = bounds;
                *found = true;
            }
        }
        collect_descendant_bounds(&child, merged, found);
    }
}

pub fn semantic_bounds(component: Option<&CoreHandle>) -> Bounds {
    let Some(component) = component else {
        return Bounds::default();
    };
    if let Some((world_bounds, artboard)) = node_world_bounds(component) {
        return artboard.as_ref().map_or(world_bounds, |artboard| {
            root_transform_aabb(artboard, world_bounds)
        });
    }

    let (is_node, artboard, world_transform) = component
        .with(|component| {
            component.as_node().map(|node| {
                (
                    true,
                    node.artboard_handle(),
                    Mat2D::from(*node.world_transform()),
                )
            })
        })
        .flatten()
        .unwrap_or((false, None, Mat2D::default()));
    if !is_node {
        return Bounds::default();
    }

    let is_container = component
        .with(|component| component.as_container_component().is_some())
        .unwrap_or(false);
    if is_container {
        let mut merged = Aabb::default();
        let mut found = false;
        collect_descendant_bounds(component, &mut merged, &mut found);
        if found {
            let merged = bounds_from_aabb(merged);
            return artboard
                .as_ref()
                .map_or(merged, |artboard| root_transform_aabb(artboard, merged));
        }
    }

    let point = Bounds {
        min_x: world_transform[4],
        min_y: world_transform[5],
        max_x: world_transform[4],
        max_y: world_transform[5],
    };
    artboard
        .as_ref()
        .map_or(point, |artboard| root_transform_aabb(artboard, point))
}
