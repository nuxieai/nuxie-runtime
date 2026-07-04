use rive_binary::RuntimeFile;
use rive_graph::{ArtboardGraph, PathGeometryNode};

use crate::components::TransformComponents;
use crate::draw::{RuntimePathMeasure, runtime_path_geometry_commands};
use crate::properties::property_key_for_name;
use crate::{ArtboardInstance, Mat2D};

#[derive(Debug, Clone)]
pub(crate) struct RuntimeFollowPathConstraint {
    local_id: usize,
    target_local: usize,
    target_kind: RuntimeFollowPathTargetKind,
    paths: Vec<RuntimeFollowPathPath>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeFollowPathTargetKind {
    Shape,
    Path,
    Other,
}

#[derive(Debug, Clone)]
struct RuntimeFollowPathPath {
    local_id: usize,
    geometry: PathGeometryNode,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeIkConstraint {
    local_id: usize,
    target_local: usize,
    chain: Vec<RuntimeIkChainLink>,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeIkChainLink {
    bone_local: usize,
}

#[derive(Debug, Clone, Copy)]
struct IkChainState {
    bone_index: usize,
    parent_world_inverse: Mat2D,
    transform_components: TransformComponents,
    angle: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransformSpace {
    World,
    Local,
}

impl TransformSpace {
    fn from_value(value: u64) -> Self {
        match value {
            1 => Self::Local,
            _ => Self::World,
        }
    }
}

pub(crate) fn build_runtime_follow_path_constraints(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeFollowPathConstraint> {
    graph
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("FollowPathConstraint"))
        .filter_map(|object| {
            let constraint = file.object(object.global_id as usize)?;
            let target_local = usize::try_from(constraint.uint_property("targetId")?).ok()?;
            let target_type = graph
                .local_objects
                .iter()
                .find(|object| object.local_id == target_local)
                .and_then(|object| object.type_name);

            let target_kind = if target_type == Some("Shape") {
                RuntimeFollowPathTargetKind::Shape
            } else if graph.paths.iter().any(|path| path.local_id == target_local) {
                RuntimeFollowPathTargetKind::Path
            } else {
                RuntimeFollowPathTargetKind::Other
            };

            let paths = match target_kind {
                RuntimeFollowPathTargetKind::Shape => graph
                    .path_composers
                    .iter()
                    .find(|composer| composer.shape_local == target_local)
                    .map(|composer| {
                        composer
                            .paths
                            .iter()
                            .filter_map(|path_ref| {
                                graph
                                    .paths
                                    .iter()
                                    .find(|path| path.local_id == path_ref.local_id)
                                    .cloned()
                                    .map(|geometry| RuntimeFollowPathPath {
                                        local_id: path_ref.local_id,
                                        geometry,
                                    })
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                RuntimeFollowPathTargetKind::Path => graph
                    .paths
                    .iter()
                    .find(|path| path.local_id == target_local)
                    .cloned()
                    .map(|geometry| {
                        vec![RuntimeFollowPathPath {
                            local_id: target_local,
                            geometry,
                        }]
                    })
                    .unwrap_or_default(),
                RuntimeFollowPathTargetKind::Other => Vec::new(),
            };

            Some(RuntimeFollowPathConstraint {
                local_id: object.local_id,
                target_local,
                target_kind,
                paths,
            })
        })
        .collect()
}

pub(crate) fn build_runtime_ik_constraints(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
) -> Vec<RuntimeIkConstraint> {
    graph
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("IKConstraint"))
        .filter_map(|object| {
            let constraint = file.object(object.global_id as usize)?;
            let tip_local = usize::try_from(constraint.uint_property("parentId")?).ok()?;
            if !local_object_type(graph, tip_local).is_some_and(is_bone_type) {
                return None;
            }

            let target_local = usize::try_from(constraint.uint_property("targetId")?).ok()?;
            let mut reverse_chain = vec![tip_local];
            let mut current_local = tip_local;
            let mut remaining = constraint.uint_property("parentBoneCount").unwrap_or(0);
            while remaining > 0 {
                let Some(parent_local) = local_object_parent(file, graph, current_local) else {
                    break;
                };
                if !local_object_type(graph, parent_local).is_some_and(is_bone_type) {
                    break;
                }
                remaining -= 1;
                reverse_chain.push(parent_local);
                current_local = parent_local;
            }
            reverse_chain.reverse();

            Some(RuntimeIkConstraint {
                local_id: object.local_id,
                target_local,
                chain: reverse_chain
                    .into_iter()
                    .map(|bone_local| RuntimeIkChainLink { bone_local })
                    .collect(),
            })
        })
        .collect()
}

/// Runtime constraint application for the C++ `src/constraints/` path.
pub(crate) fn apply_constraints(artboard: &mut ArtboardInstance, component_index: usize) -> bool {
    let constraint_locals = artboard.components[component_index]
        .constraint_locals
        .clone();
    constraint_locals
        .into_iter()
        .fold(false, |changed, constraint_local| {
            changed | apply_constraint(artboard, component_index, constraint_local)
        })
}

fn apply_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    let Some(slot) = artboard.slot(constraint_local) else {
        return false;
    };
    match slot.type_name {
        Some("DistanceConstraint") => {
            apply_distance_constraint(artboard, component_index, constraint_local)
        }
        Some("TranslationConstraint") => {
            apply_translation_constraint(artboard, component_index, constraint_local)
        }
        Some("RotationConstraint") => {
            apply_rotation_constraint(artboard, component_index, constraint_local)
        }
        Some("ScaleConstraint") => {
            apply_scale_constraint(artboard, component_index, constraint_local)
        }
        Some("TransformConstraint") => {
            apply_transform_constraint(artboard, component_index, constraint_local)
        }
        Some("FollowPathConstraint") => {
            apply_follow_path_constraint(artboard, component_index, constraint_local)
        }
        Some("IKConstraint") => apply_ik_constraint(artboard, component_index, constraint_local),
        _ => false,
    }
}

fn apply_distance_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/distance_constraint.cpp`.
    let Some(slot) = artboard.slot(constraint_local) else {
        return false;
    };
    if slot.type_name != Some("DistanceConstraint") {
        return false;
    }

    let Some(target_local) = targeted_constraint_target_local(artboard, constraint_local) else {
        return false;
    };
    if artboard
        .component(target_local)
        .is_some_and(|target| target.is_collapsed())
    {
        return false;
    }

    let Some(target_index) = artboard.component_by_local.get(&target_local).copied() else {
        return false;
    };
    let target_transform = artboard.components[target_index].transform.world_transform;
    let target_x = target_transform.0[4];
    let target_y = target_transform.0[5];

    let world = artboard.components[component_index]
        .transform
        .world_transform;
    let our_x = world.0[4];
    let our_y = world.0[5];
    let to_target_x = our_x - target_x;
    let to_target_y = our_y - target_y;
    let current_distance = to_target_x.hypot(to_target_y);
    let distance = constraint_double(
        artboard,
        constraint_local,
        "DistanceConstraint",
        "distance",
        100.0,
    );

    match constraint_uint(
        artboard,
        constraint_local,
        "DistanceConstraint",
        "modeValue",
        0,
    ) {
        0 if current_distance < distance => return false,
        1 if current_distance > distance => return false,
        _ => {}
    }
    if current_distance < 0.001 {
        return false;
    }

    let scale = distance / current_distance;
    let constrained_x = target_x + to_target_x * scale;
    let constrained_y = target_y + to_target_y * scale;
    let strength = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
    let new_x = our_x + (constrained_x - our_x) * strength;
    let new_y = our_y + (constrained_y - our_y) * strength;

    let world = &mut artboard.components[component_index]
        .transform
        .world_transform
        .0;
    if world[4] == new_x && world[5] == new_y {
        return false;
    }
    world[4] = new_x;
    world[5] = new_y;
    true
}

fn apply_rotation_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/rotation_constraint.cpp`.
    let target_index = targeted_constraint_target_local(artboard, constraint_local)
        .and_then(|target_local| artboard.component_by_local.get(&target_local).copied());
    if target_index.is_some_and(|index| artboard.components[index].is_collapsed()) {
        return false;
    }

    let transform_a = artboard.components[component_index]
        .transform
        .world_transform;
    let components_a = transform_a.decompose();
    let mut components_b = components_a;

    if let Some(target_index) = target_index {
        let mut transform_b = artboard.components[target_index].transform.world_transform;
        if transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "sourceSpaceValue",
        ) == TransformSpace::Local
        {
            let Some(inverse) = invert(parent_world_transform(artboard, target_index)) else {
                return false;
            };
            transform_b = inverse.multiply(transform_b);
        }

        components_b = transform_b.decompose();
        let dest_space = transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "destSpaceValue",
        );
        if !constraint_bool(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "doesCopy",
            true,
        ) {
            components_b.rotation = if dest_space == TransformSpace::Local {
                0.0
            } else {
                components_a.rotation
            };
        } else {
            components_b.rotation *= constraint_double(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "copyFactor",
                1.0,
            );
            if constraint_bool(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "offset",
                false,
            ) {
                let authored =
                    artboard.authored_transform(artboard.components[component_index].local_id);
                components_b.rotation += authored.rotation;
            }
        }

        if dest_space == TransformSpace::Local {
            transform_b = parent_world_transform(artboard, component_index)
                .multiply(Mat2D::compose(components_b));
            components_b = transform_b.decompose();
        }
    }

    let clamp_local = transform_space(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "minMaxSpaceValue",
    ) == TransformSpace::Local;
    if clamp_local {
        let transform_b = Mat2D::compose(components_b);
        let Some(inverse) = invert(parent_world_transform(artboard, component_index)) else {
            return false;
        };
        components_b = inverse.multiply(transform_b).decompose();
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "max",
        false,
    ) && components_b.rotation
        > constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        )
    {
        components_b.rotation = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "min",
        false,
    ) && components_b.rotation
        < constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        )
    {
        components_b.rotation = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        );
    }
    if clamp_local {
        let transform_b = parent_world_transform(artboard, component_index)
            .multiply(Mat2D::compose(components_b));
        components_b = transform_b.decompose();
    }

    components_b.rotation = interpolated_rotation(
        components_a.rotation,
        components_b.rotation,
        constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0),
    );
    components_b.x = components_a.x;
    components_b.y = components_a.y;
    components_b.scale_x = components_a.scale_x;
    components_b.scale_y = components_a.scale_y;
    components_b.skew = components_a.skew;

    write_world_transform(artboard, component_index, Mat2D::compose(components_b))
}

fn apply_scale_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/scale_constraint.cpp`.
    let target_index = targeted_constraint_target_local(artboard, constraint_local)
        .and_then(|target_local| artboard.component_by_local.get(&target_local).copied());
    if target_index.is_some_and(|index| artboard.components[index].is_collapsed()) {
        return false;
    }

    let transform_a = artboard.components[component_index]
        .transform
        .world_transform;
    let components_a = transform_a.decompose();
    let mut components_b = components_a;

    if let Some(target_index) = target_index {
        let mut transform_b = artboard.components[target_index].transform.world_transform;
        if transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "sourceSpaceValue",
        ) == TransformSpace::Local
        {
            let Some(inverse) = invert(parent_world_transform(artboard, target_index)) else {
                return false;
            };
            transform_b = inverse.multiply(transform_b);
        }

        components_b = transform_b.decompose();
        let dest_space = transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "destSpaceValue",
        );
        let authored = artboard.authored_transform(artboard.components[component_index].local_id);
        if !constraint_bool(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "doesCopy",
            true,
        ) {
            components_b.scale_x = if dest_space == TransformSpace::Local {
                1.0
            } else {
                components_a.scale_x
            };
        } else {
            components_b.scale_x *= constraint_double(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "copyFactor",
                1.0,
            );
            if constraint_bool(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "offset",
                false,
            ) {
                components_b.scale_x *= authored.scale_x;
            }
        }

        if !constraint_bool(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "doesCopyY",
            true,
        ) {
            components_b.scale_y = if dest_space == TransformSpace::Local {
                1.0
            } else {
                components_a.scale_y
            };
        } else {
            components_b.scale_y *= constraint_double(
                artboard,
                constraint_local,
                "TransformComponentConstraintY",
                "copyFactorY",
                1.0,
            );
            if constraint_bool(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "offset",
                false,
            ) {
                components_b.scale_y *= authored.scale_y;
            }
        }

        if dest_space == TransformSpace::Local {
            let transform_b = parent_world_transform(artboard, component_index)
                .multiply(Mat2D::compose(components_b));
            components_b = transform_b.decompose();
        }
    }

    let clamp_local = transform_space(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "minMaxSpaceValue",
    ) == TransformSpace::Local;
    if clamp_local {
        let transform_b = Mat2D::compose(components_b);
        let Some(inverse) = invert(parent_world_transform(artboard, component_index)) else {
            return false;
        };
        components_b = inverse.multiply(transform_b).decompose();
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "max",
        false,
    ) && components_b.scale_x
        > constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        )
    {
        components_b.scale_x = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "min",
        false,
    ) && components_b.scale_x
        < constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        )
    {
        components_b.scale_x = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraintY",
        "maxY",
        false,
    ) && components_b.scale_y
        > constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "maxValueY",
            0.0,
        )
    {
        components_b.scale_y = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "maxValueY",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraintY",
        "minY",
        false,
    ) && components_b.scale_y
        < constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "minValueY",
            0.0,
        )
    {
        components_b.scale_y = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "minValueY",
            0.0,
        );
    }
    if clamp_local {
        let transform_b = parent_world_transform(artboard, component_index)
            .multiply(Mat2D::compose(components_b));
        components_b = transform_b.decompose();
    }

    let t = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
    let ti = 1.0 - t;
    components_b.rotation = components_a.rotation;
    components_b.x = components_a.x;
    components_b.y = components_a.y;
    components_b.scale_x = components_a.scale_x * ti + components_b.scale_x * t;
    components_b.scale_y = components_a.scale_y * ti + components_b.scale_y * t;
    components_b.skew = components_a.skew;

    write_world_transform(artboard, component_index, Mat2D::compose(components_b))
}

fn apply_transform_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/transform_constraint.cpp`.
    let Some(target_index) = targeted_constraint_target_local(artboard, constraint_local)
        .and_then(|target_local| artboard.component_by_local.get(&target_local).copied())
    else {
        return false;
    };
    if artboard.components[target_index].is_collapsed() {
        return false;
    }

    let transform_a = artboard.components[component_index]
        .transform
        .world_transform;
    let mut transform_b =
        target_transform_for_transform_constraint(artboard, target_index, constraint_local);
    if transform_space(
        artboard,
        constraint_local,
        "TransformSpaceConstraint",
        "sourceSpaceValue",
    ) == TransformSpace::Local
    {
        let Some(inverse) = invert(parent_world_transform(artboard, target_index)) else {
            return false;
        };
        transform_b = inverse.multiply(transform_b);
    }
    if transform_space(
        artboard,
        constraint_local,
        "TransformSpaceConstraint",
        "destSpaceValue",
    ) == TransformSpace::Local
    {
        transform_b = parent_world_transform(artboard, component_index).multiply(transform_b);
    }

    constrain_world(
        artboard,
        component_index,
        transform_a,
        transform_b,
        constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0),
    )
}

fn apply_follow_path_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/follow_path_constraint.cpp`.
    let Some(runtime) = follow_path_constraint(artboard, constraint_local).cloned() else {
        return false;
    };
    let Some(target_index) = artboard
        .component_by_local
        .get(&runtime.target_local)
        .copied()
    else {
        return false;
    };
    if artboard.components[target_index].is_collapsed() {
        return false;
    }

    let transform_b = target_transform_for_follow_path_constraint(
        artboard,
        &runtime,
        target_index,
        component_index,
    );
    let components = follow_path_constrain_components(
        artboard,
        constraint_local,
        target_index,
        artboard.components[component_index]
            .transform
            .world_transform,
        transform_b,
        parent_world_transform(artboard, component_index),
    );
    write_world_transform(artboard, component_index, Mat2D::compose(components))
}

fn target_transform_for_follow_path_constraint(
    artboard: &ArtboardInstance,
    runtime: &RuntimeFollowPathConstraint,
    target_index: usize,
    component_index: usize,
) -> Mat2D {
    match runtime.target_kind {
        RuntimeFollowPathTargetKind::Shape | RuntimeFollowPathTargetKind::Path => {
            let distance = constraint_double(
                artboard,
                runtime.local_id,
                "FollowPathConstraint",
                "distance",
                0.0,
            );
            let mut commands = Vec::new();
            for path in &runtime.paths {
                let Some(path_world) = artboard
                    .component(path.local_id)
                    .map(|component| component.transform.world_transform)
                else {
                    continue;
                };
                commands.extend(runtime_path_geometry_commands(
                    artboard,
                    &path.geometry,
                    path_world,
                ));
            }

            let sample = RuntimePathMeasure::from_commands(&commands).at_percentage(distance);
            let mut transform_b = artboard.components[target_index].transform.world_transform;

            if constraint_bool(
                artboard,
                runtime.local_id,
                "FollowPathConstraint",
                "orient",
                true,
            ) {
                let components_b = transform_b.decompose();
                let tangent_rotation = sample.tan.1.atan2(sample.tan.0);
                let two_pi = std::f32::consts::PI * 2.0;
                let angle_b = components_b.rotation % two_pi;
                let mut diff = tangent_rotation - angle_b;
                if diff > std::f32::consts::PI {
                    diff -= two_pi;
                } else if diff < -std::f32::consts::PI {
                    diff += two_pi;
                }
                transform_b = Mat2D::from_rotation(
                    angle_b
                        + diff
                            * constraint_double(
                                artboard,
                                runtime.local_id,
                                "Constraint",
                                "strength",
                                1.0,
                            ),
                );
            }

            let offset_position = if constraint_bool(
                artboard,
                runtime.local_id,
                "FollowPathConstraint",
                "offset",
                false,
            ) {
                let local_transform = artboard.components[component_index]
                    .transform
                    .local_transform
                    .0;
                (local_transform[4], local_transform[5])
            } else {
                (0.0, 0.0)
            };
            transform_b.0[4] = sample.pos.0 + offset_position.0;
            transform_b.0[5] = sample.pos.1 + offset_position.1;
            transform_b
        }
        RuntimeFollowPathTargetKind::Other => {
            artboard.components[target_index].transform.world_transform
        }
    }
}

fn follow_path_constrain_components(
    artboard: &ArtboardInstance,
    constraint_local: usize,
    target_index: usize,
    component_transform: Mat2D,
    mut transform_b: Mat2D,
    component_parent_world: Mat2D,
) -> TransformComponents {
    if transform_space(
        artboard,
        constraint_local,
        "TransformSpaceConstraint",
        "sourceSpaceValue",
    ) == TransformSpace::Local
    {
        let target_parent_world = parent_world_transform(artboard, target_index);
        let Some(inverse) = invert(target_parent_world) else {
            return TransformComponents::default();
        };
        transform_b = inverse.multiply(transform_b);
    }
    if transform_space(
        artboard,
        constraint_local,
        "TransformSpaceConstraint",
        "destSpaceValue",
    ) == TransformSpace::Local
    {
        transform_b = component_parent_world.multiply(transform_b);
    }

    let components_a = component_transform.decompose();
    let mut components_b = transform_b.decompose();
    let t = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
    let ti = 1.0 - t;

    if !constraint_bool(
        artboard,
        constraint_local,
        "FollowPathConstraint",
        "orient",
        true,
    ) {
        components_b.rotation = components_a.rotation % (std::f32::consts::PI * 2.0);
    }
    components_b.x = components_a.x * ti + components_b.x * t;
    components_b.y = components_a.y * ti + components_b.y * t;
    components_b.scale_x = components_a.scale_x;
    components_b.scale_y = components_a.scale_y;
    components_b.skew = components_a.skew;
    components_b
}

fn follow_path_constraint(
    artboard: &ArtboardInstance,
    constraint_local: usize,
) -> Option<&RuntimeFollowPathConstraint> {
    artboard
        .follow_path_constraints
        .iter()
        .find(|constraint| constraint.local_id == constraint_local)
}

fn apply_ik_constraint(
    artboard: &mut ArtboardInstance,
    _component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/ik_constraint.cpp`.
    let Some(runtime) = ik_constraint(artboard, constraint_local).cloned() else {
        return false;
    };
    let Some(target_index) = artboard
        .component_by_local
        .get(&runtime.target_local)
        .copied()
    else {
        return false;
    };
    if artboard.components[target_index].is_collapsed() {
        return false;
    }

    let invert_direction = constraint_bool(
        artboard,
        constraint_local,
        "IKConstraint",
        "invertDirection",
        false,
    );
    let world_target_translation =
        world_translation(artboard.components[target_index].transform.world_transform);
    let mut chain = Vec::new();
    let mut changed = false;
    for link in &runtime.chain {
        let Some(bone_index) = artboard.component_by_local.get(&link.bone_local).copied() else {
            continue;
        };
        let parent_world = parent_world_transform(artboard, bone_index);
        let parent_world_inverse = parent_world.invert_or_identity();
        let bone_transform = parent_world_inverse
            .multiply(artboard.components[bone_index].transform.world_transform);
        changed |= write_local_transform(artboard, bone_index, bone_transform);
        chain.push(IkChainState {
            bone_index,
            parent_world_inverse,
            transform_components: bone_transform.decompose(),
            angle: 0.0,
        });
    }

    match chain.len() {
        0 => return changed,
        1 => {
            changed |= solve_ik1(artboard, &mut chain, 0, world_target_translation);
        }
        2 => {
            changed |= solve_ik2(
                artboard,
                &mut chain,
                0,
                1,
                world_target_translation,
                invert_direction,
            );
        }
        count => {
            let tip_index = count - 1;
            for index in 0..tip_index {
                changed |= solve_ik2(
                    artboard,
                    &mut chain,
                    index,
                    tip_index,
                    world_target_translation,
                    invert_direction,
                );
                for child_index in (index + 1)..tip_index {
                    let bone_index = chain[child_index].bone_index;
                    chain[child_index].parent_world_inverse =
                        parent_world_transform(artboard, bone_index).invert_or_identity();
                }
            }
        }
    }

    let strength = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
    if strength != 1.0 {
        for index in 0..chain.len() {
            let from_angle =
                chain[index].transform_components.rotation % (std::f32::consts::PI * 2.0);
            let to_angle = chain[index].angle % (std::f32::consts::PI * 2.0);
            let mut diff = to_angle - from_angle;
            if diff > std::f32::consts::PI {
                diff -= std::f32::consts::PI * 2.0;
            } else if diff < -std::f32::consts::PI {
                diff += std::f32::consts::PI * 2.0;
            }
            changed |= constrain_ik_rotation(artboard, &chain[index], from_angle + diff * strength);
        }
    }

    changed
}

fn solve_ik1(
    artboard: &mut ArtboardInstance,
    chain: &mut [IkChainState],
    index: usize,
    world_target_translation: (f32, f32),
) -> bool {
    let bone_index = chain[index].bone_index;
    let p_a = world_translation(artboard.components[bone_index].transform.world_transform);
    let to_target = (
        world_target_translation.0 - p_a.0,
        world_target_translation.1 - p_a.1,
    );
    let to_target_local = chain[index]
        .parent_world_inverse
        .transform_direction(to_target.0, to_target.1);
    let rotation = point_atan2(to_target_local);
    chain[index].angle = rotation;
    constrain_ik_rotation(artboard, &chain[index], rotation)
}

fn solve_ik2(
    artboard: &mut ArtboardInstance,
    chain: &mut [IkChainState],
    fk1_index: usize,
    fk2_index: usize,
    world_target_translation: (f32, f32),
    invert_direction: bool,
) -> bool {
    let first_child_index = fk1_index + 1;
    let b1_index = chain[fk1_index].bone_index;
    let b2_index = chain[fk2_index].bone_index;
    let first_child_bone_index = chain[first_child_index].bone_index;
    let iworld = chain[fk1_index].parent_world_inverse;

    let mut p_a = world_translation(artboard.components[b1_index].transform.world_transform);
    let mut p_c = world_translation(
        artboard.components[first_child_bone_index]
            .transform
            .world_transform,
    );
    let mut p_b = tip_world_translation(artboard, b2_index);
    let mut p_bt = world_target_translation;

    p_a = iworld.transform_point(p_a.0, p_a.1);
    p_c = iworld.transform_point(p_c.0, p_c.1);
    p_b = iworld.transform_point(p_b.0, p_b.1);
    p_bt = iworld.transform_point(p_bt.0, p_bt.1);

    let av = point_sub(p_b, p_c);
    let bv = point_sub(p_c, p_a);
    let cv = point_sub(p_bt, p_a);
    let a = point_length(av);
    let b = point_length(bv);
    let c = point_length(cv);

    let angle_a = ((-a * a + b * b + c * c) / (2.0 * b * c))
        .clamp(-1.0, 1.0)
        .acos();
    let angle_c = ((a * a + b * b - c * c) / (2.0 * a * b))
        .clamp(-1.0, 1.0)
        .acos();

    let (r1, r2) = if artboard.components[b2_index].parent_local
        != Some(artboard.components[b1_index].local_id)
    {
        let second_child_index = fk1_index + 2;
        let second_child_world_inverse = chain[second_child_index].parent_world_inverse;
        let p_c_world = world_translation(
            artboard.components[first_child_bone_index]
                .transform
                .world_transform,
        );
        let p_b_world = tip_world_translation(artboard, b2_index);
        let av_local = second_child_world_inverse
            .transform_direction(p_b_world.0 - p_c_world.0, p_b_world.1 - p_c_world.1);
        let angle_correction = -point_atan2(av_local);
        if invert_direction {
            (
                point_atan2(cv) - angle_a,
                -angle_c + std::f32::consts::PI + angle_correction,
            )
        } else {
            (
                angle_a + point_atan2(cv),
                angle_c - std::f32::consts::PI + angle_correction,
            )
        }
    } else if invert_direction {
        (point_atan2(cv) - angle_a, -angle_c + std::f32::consts::PI)
    } else {
        (angle_a + point_atan2(cv), angle_c - std::f32::consts::PI)
    };

    let mut changed = false;
    changed |= constrain_ik_rotation(artboard, &chain[fk1_index], r1);
    changed |= constrain_ik_rotation(artboard, &chain[first_child_index], r2);
    if first_child_index != fk2_index {
        let bone_index = chain[fk2_index].bone_index;
        let parent_world = parent_world_transform(artboard, bone_index);
        let local = artboard.components[bone_index].transform.local_transform;
        changed |= write_world_transform(artboard, bone_index, parent_world.multiply(local));
    }

    chain[fk1_index].angle = r1;
    chain[first_child_index].angle = r2;
    changed
}

fn constrain_ik_rotation(
    artboard: &mut ArtboardInstance,
    state: &IkChainState,
    rotation: f32,
) -> bool {
    let bone_index = state.bone_index;
    let mut components = state.transform_components;
    components.rotation = rotation;
    let local_transform = Mat2D::compose(components);
    let parent_world = parent_world_transform(artboard, bone_index);
    write_local_world_transform(
        artboard,
        bone_index,
        local_transform,
        parent_world.multiply(local_transform),
    )
}

fn ik_constraint(
    artboard: &ArtboardInstance,
    constraint_local: usize,
) -> Option<&RuntimeIkConstraint> {
    artboard
        .ik_constraints
        .iter()
        .find(|constraint| constraint.local_id == constraint_local)
}

fn target_transform_for_transform_constraint(
    artboard: &ArtboardInstance,
    target_index: usize,
    constraint_local: usize,
) -> Mat2D {
    let (left, top, width, height) = constraint_bounds(artboard, target_index);
    let origin_x = constraint_double(
        artboard,
        constraint_local,
        "TransformConstraint",
        "originX",
        0.0,
    );
    let origin_y = constraint_double(
        artboard,
        constraint_local,
        "TransformConstraint",
        "originY",
        0.0,
    );
    artboard.components[target_index]
        .transform
        .world_transform
        .multiply(Mat2D([
            1.0,
            0.0,
            0.0,
            1.0,
            left + width * origin_x,
            top + height * origin_y,
        ]))
}

fn constraint_bounds(
    _artboard: &ArtboardInstance,
    _component_index: usize,
) -> (f32, f32, f32, f32) {
    // C++ `TransformComponent::constraintBounds()` defaults to an empty AABB.
    // Text/LayoutComponent overrides stay behind their M6 gates for now.
    (0.0, 0.0, 0.0, 0.0)
}

fn constrain_world(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    from: Mat2D,
    to: Mat2D,
    strength: f32,
) -> bool {
    let components_from = from.decompose();
    let mut components_to = to.decompose();
    let t = strength;
    let ti = 1.0 - t;

    components_to.rotation =
        interpolated_rotation(components_from.rotation, components_to.rotation, t);
    components_to.x = components_from.x * ti + components_to.x * t;
    components_to.y = components_from.y * ti + components_to.y * t;
    components_to.scale_x = components_from.scale_x * ti + components_to.scale_x * t;
    components_to.scale_y = components_from.scale_y * ti + components_to.scale_y * t;
    components_to.skew = components_from.skew * ti + components_to.skew * t;

    write_world_transform(artboard, component_index, Mat2D::compose(components_to))
}

fn apply_translation_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/translation_constraint.cpp`.
    let target_index = targeted_constraint_target_local(artboard, constraint_local)
        .and_then(|target_local| artboard.component_by_local.get(&target_local).copied());
    if target_index.is_some_and(|index| artboard.components[index].is_collapsed()) {
        return false;
    }

    let world = artboard.components[component_index]
        .transform
        .world_transform;
    let translation_a = (world.0[4], world.0[5]);
    let mut translation_b = translation_a;

    if let Some(target_index) = target_index {
        let mut transform_b = artboard.components[target_index].transform.world_transform;
        if transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "sourceSpaceValue",
        ) == TransformSpace::Local
        {
            let Some(inverse) = invert(parent_world_transform(artboard, target_index)) else {
                return false;
            };
            transform_b = inverse.multiply(transform_b);
        }
        translation_b = (transform_b.0[4], transform_b.0[5]);

        let dest_space = transform_space(
            artboard,
            constraint_local,
            "TransformSpaceConstraint",
            "destSpaceValue",
        );
        let authored = artboard.authored_transform(artboard.components[component_index].local_id);
        if !constraint_bool(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "doesCopy",
            true,
        ) {
            translation_b.0 = if dest_space == TransformSpace::Local {
                0.0
            } else {
                translation_a.0
            };
        } else {
            translation_b.0 *= constraint_double(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "copyFactor",
                1.0,
            );
            if constraint_bool(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "offset",
                false,
            ) {
                translation_b.0 += authored.x;
            }
        }

        if !constraint_bool(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "doesCopyY",
            true,
        ) {
            translation_b.1 = if dest_space == TransformSpace::Local {
                0.0
            } else {
                translation_a.1
            };
        } else {
            translation_b.1 *= constraint_double(
                artboard,
                constraint_local,
                "TransformComponentConstraintY",
                "copyFactorY",
                1.0,
            );
            if constraint_bool(
                artboard,
                constraint_local,
                "TransformComponentConstraint",
                "offset",
                false,
            ) {
                translation_b.1 += authored.y;
            }
        }

        if dest_space == TransformSpace::Local {
            translation_b = parent_world_transform(artboard, component_index)
                .transform_point(translation_b.0, translation_b.1);
        }
    }

    let clamp_local = transform_space(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "minMaxSpaceValue",
    ) == TransformSpace::Local;
    if clamp_local {
        let Some(inverse) = invert(parent_world_transform(artboard, component_index)) else {
            return false;
        };
        translation_b = inverse.transform_point(translation_b.0, translation_b.1);
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "max",
        false,
    ) && translation_b.0
        > constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        )
    {
        translation_b.0 = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "maxValue",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraint",
        "min",
        false,
    ) && translation_b.0
        < constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        )
    {
        translation_b.0 = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraint",
            "minValue",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraintY",
        "maxY",
        false,
    ) && translation_b.1
        > constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "maxValueY",
            0.0,
        )
    {
        translation_b.1 = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "maxValueY",
            0.0,
        );
    }
    if constraint_bool(
        artboard,
        constraint_local,
        "TransformComponentConstraintY",
        "minY",
        false,
    ) && translation_b.1
        < constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "minValueY",
            0.0,
        )
    {
        translation_b.1 = constraint_double(
            artboard,
            constraint_local,
            "TransformComponentConstraintY",
            "minValueY",
            0.0,
        );
    }
    if clamp_local {
        translation_b = parent_world_transform(artboard, component_index)
            .transform_point(translation_b.0, translation_b.1);
    }

    let t = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
    let ti = 1.0 - t;
    let new_x = translation_a.0 * ti + translation_b.0 * t;
    let new_y = translation_a.1 * ti + translation_b.1 * t;

    let mut transform = artboard.components[component_index]
        .transform
        .world_transform;
    transform.0[4] = new_x;
    transform.0[5] = new_y;
    write_world_transform(artboard, component_index, transform)
}

fn write_world_transform(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    transform: Mat2D,
) -> bool {
    let world = &mut artboard.components[component_index]
        .transform
        .world_transform
        .0;
    if *world == transform.0 {
        return false;
    }
    *world = transform.0;
    true
}

fn write_local_transform(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    transform: Mat2D,
) -> bool {
    let local = &mut artboard.components[component_index]
        .transform
        .local_transform
        .0;
    if *local == transform.0 {
        return false;
    }
    *local = transform.0;
    true
}

fn write_local_world_transform(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    local_transform: Mat2D,
    world_transform: Mat2D,
) -> bool {
    let local_changed = write_local_transform(artboard, component_index, local_transform);
    let world_changed = write_world_transform(artboard, component_index, world_transform);
    local_changed || world_changed
}

fn world_translation(transform: Mat2D) -> (f32, f32) {
    (transform.0[4], transform.0[5])
}

fn tip_world_translation(artboard: &ArtboardInstance, bone_index: usize) -> (f32, f32) {
    let bone = &artboard.components[bone_index];
    let length = artboard.bone_length(bone.local_id).unwrap_or(0.0);
    bone.transform.world_transform.transform_point(length, 0.0)
}

fn point_sub(left: (f32, f32), right: (f32, f32)) -> (f32, f32) {
    (left.0 - right.0, left.1 - right.1)
}

fn point_length(point: (f32, f32)) -> f32 {
    point.0.hypot(point.1)
}

fn point_atan2(point: (f32, f32)) -> f32 {
    point.1.atan2(point.0)
}

fn targeted_constraint_target_local(
    artboard: &ArtboardInstance,
    constraint_local: usize,
) -> Option<usize> {
    let target_key = property_key_for_name("TargetedConstraint", "targetId")?;
    let target_local =
        usize::try_from(artboard.uint_property(constraint_local, target_key)?).ok()?;
    artboard.slot(target_local).map(|slot| slot.local_id)
}

fn parent_world_transform(artboard: &ArtboardInstance, component_index: usize) -> Mat2D {
    let Some(parent_local) = artboard.components[component_index].parent_local else {
        return Mat2D::IDENTITY;
    };
    artboard
        .component(parent_local)
        .filter(|parent| parent.capabilities.world_transform)
        .map(|parent| parent.transform.world_transform)
        .unwrap_or(Mat2D::IDENTITY)
}

fn invert(transform: Mat2D) -> Option<Mat2D> {
    (transform.determinant() != 0.0).then(|| transform.invert_or_identity())
}

fn transform_space(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
) -> TransformSpace {
    TransformSpace::from_value(constraint_uint(
        artboard,
        local_id,
        type_name,
        property_name,
        0,
    ))
}

fn interpolated_rotation(from: f32, to: f32, strength: f32) -> f32 {
    let two_pi = std::f32::consts::PI * 2.0;
    let angle_a = from % two_pi;
    let angle_b = to % two_pi;
    let mut diff = angle_b - angle_a;
    if diff > std::f32::consts::PI {
        diff -= two_pi;
    } else if diff < -std::f32::consts::PI {
        diff += two_pi;
    }
    from + diff * strength
}

fn constraint_double(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: f32,
) -> f32 {
    property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.double_property(local_id, key))
        .unwrap_or(default)
}

fn constraint_bool(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: bool,
) -> bool {
    property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.bool_property(local_id, key))
        .unwrap_or(default)
}

fn constraint_uint(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: u64,
) -> u64 {
    property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.uint_property(local_id, key))
        .unwrap_or(default)
}

fn local_object_type(graph: &ArtboardGraph, local_id: usize) -> Option<&'static str> {
    graph
        .local_objects
        .iter()
        .find(|object| object.local_id == local_id)
        .and_then(|object| object.type_name)
}

fn local_object_parent(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    local_id: usize,
) -> Option<usize> {
    let global_id = graph
        .local_objects
        .iter()
        .find(|object| object.local_id == local_id)?
        .global_id;
    usize::try_from(file.object(global_id as usize)?.uint_property("parentId")?).ok()
}

fn is_bone_type(type_name: &'static str) -> bool {
    type_name == "Bone" || type_name == "RootBone"
}
