use super::ScriptListenerInvocation;
use crate::ArtboardInstance;
use crate::components::Mat2D;
use crate::properties::property_key_for_name;
use nuxie_schema::definition_by_name;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeListenerAlignTarget {
    pub(crate) action_owner: super::RuntimeActionCoreHandle,
}

impl RuntimeListenerAlignTarget {
    #[cfg(test)]
    pub(crate) fn for_test(
        flags: u64,
        target_local_id: Option<usize>,
        preserve_offset: bool,
    ) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("ListenerAlignTarget");
        action_owner.set_uint(super::listener_action_owner::LISTENER_FLAGS_KEY, flags);
        action_owner.set_uint(
            super::listener_action_owner::LISTENER_ALIGN_TARGET_ID_KEY,
            target_local_id
                .and_then(|value| u64::try_from(value).ok())
                .unwrap_or(u64::from(u32::MAX)),
        );
        action_owner.set_bool(
            super::listener_action_owner::LISTENER_ALIGN_PRESERVE_OFFSET_KEY,
            preserve_offset,
        );
        Self { action_owner }
    }

    pub(crate) fn perform(
        &self,
        artboard: &mut ArtboardInstance,
        invocation: &ScriptListenerInvocation,
    ) -> bool {
        let (position, previous_position) = match invocation {
            ScriptListenerInvocation::Pointer {
                x,
                y,
                previous_x,
                previous_y,
                ..
            } => ((*x, *y), (*previous_x, *previous_y)),
            _ => ((0.0, 0.0), (0.0, 0.0)),
        };
        let target_local_id = self
            .action_owner
            .uint(super::listener_action_owner::LISTENER_ALIGN_TARGET_ID_KEY);
        let Ok(target_local_id) = usize::try_from(target_local_id) else {
            return false;
        };
        let Some(target_handle) = artboard.component_handle(target_local_id) else {
            return false;
        };
        let target = artboard.component_at(target_handle);
        if !definition_by_name(target.type_name).is_some_and(|definition| definition.is_a("Node")) {
            return false;
        }
        let target_type = target.type_name;
        // C++ `getParentWorld` inspects the immediate Component parent. It
        // does not use `Node::m_ParentTransformComponent`, which may skip a
        // non-world-transform container and find a more distant ancestor.
        let parent_world = target
            .parent
            .map(|parent| artboard.component_at(parent))
            .filter(|parent| parent.capabilities.world_transform)
            .map(|parent| parent.transform.world_transform)
            .unwrap_or(Mat2D::IDENTITY);
        let Some(inverse) = invert_parent_world_like_cpp(parent_world) else {
            return false;
        };
        let local_position = inverse.transform_point(position.0, position.1);
        let previous_local = inverse.transform_point(previous_position.0, previous_position.1);
        let Some(x_key) = property_key_for_name(target_type, "x") else {
            return false;
        };
        let Some(y_key) = property_key_for_name(target_type, "y") else {
            return false;
        };
        let preserve_offset = self
            .action_owner
            .bool(super::listener_action_owner::LISTENER_ALIGN_PRESERVE_OFFSET_KEY);
        let (x, y) = if preserve_offset {
            (
                artboard
                    .double_property(target_local_id, x_key)
                    .unwrap_or(0.0)
                    + local_position.0
                    - previous_local.0,
                artboard
                    .double_property(target_local_id, y_key)
                    .unwrap_or(0.0)
                    + local_position.1
                    - previous_local.1,
            )
        } else {
            local_position
        };
        let changed_x = artboard.set_double_property(target_local_id, x_key, x);
        let changed_y = artboard.set_double_property(target_local_id, y_key, y);
        changed_x || changed_y
    }
}

/// Exact arithmetic from pinned `Mat2D::invert`, as called by
/// `ListenerAlignTarget::perform`.
///
/// The shared Rust matrix helper uses `mul_add` in its determinant and
/// translation terms. C++ at the pin uses distinct `*` and `-` float
/// operations, so this owner must not silently inherit fused rounding.
fn invert_parent_world_like_cpp(matrix: Mat2D) -> Option<Mat2D> {
    let [aa, ab, ac, ad, atx, aty] = matrix.0;
    let mut determinant = aa * ad - ab * ac;
    if determinant == 0.0 {
        return None;
    }
    determinant = 1.0 / determinant;
    Some(Mat2D([
        ad * determinant,
        -ab * determinant,
        -ac * determinant,
        aa * determinant,
        (ac * aty - ad * atx) * determinant,
        (ab * atx - aa * aty) * determinant,
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::ScriptPointerEventKind;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue, RuntimeFile};
    use nuxie_graph::GraphFile;

    fn property(type_name: &str, name: &str, value: AuthoringValue) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
            value,
        }
    }

    fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn artboard() -> ArtboardInstance {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                "Node",
                vec![
                    property("Node", "parentId", AuthoringValue::Uint(0)),
                    property("Node", "x", AuthoringValue::Double(0.0)),
                    property("Node", "y", AuthoringValue::Double(0.0)),
                ],
            ),
            record(
                "Node",
                vec![
                    property("Node", "parentId", AuthoringValue::Uint(1)),
                    property("Node", "x", AuthoringValue::Double(5.0)),
                    property("Node", "y", AuthoringValue::Double(7.0)),
                ],
            ),
            record(
                "CustomPropertyGroup",
                vec![property(
                    "CustomPropertyGroup",
                    "parentId",
                    AuthoringValue::Uint(1),
                )],
            ),
            record(
                "Node",
                vec![
                    property("Node", "parentId", AuthoringValue::Uint(3)),
                    property("Node", "x", AuthoringValue::Double(5.0)),
                    property("Node", "y", AuthoringValue::Double(7.0)),
                ],
            ),
            record(
                "Event",
                vec![property("Event", "parentId", AuthoringValue::Uint(0))],
            ),
        ])
        .expect("align records import");
        let graph = GraphFile::from_runtime_file(&file).expect("align graph builds");
        ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("align artboard"),
            &graph.artboards,
        )
        .expect("align artboard instantiates")
    }

    fn pointer(x: f32, y: f32, previous_x: f32, previous_y: f32) -> ScriptListenerInvocation {
        ScriptListenerInvocation::Pointer {
            pointer_id: 1,
            x,
            y,
            previous_x,
            previous_y,
            event: ScriptPointerEventKind::Move,
            timestamp_seconds: 0.0,
        }
    }

    fn position(artboard: &ArtboardInstance) -> (f32, f32) {
        let x = property_key_for_name("Node", "x").expect("Node.x");
        let y = property_key_for_name("Node", "y").expect("Node.y");
        (
            artboard.double_property(2, x).expect("target x"),
            artboard.double_property(2, y).expect("target y"),
        )
    }

    #[test]
    fn align_target_inverse_uses_cpp_nonfused_float_rounding() {
        let matrix = Mat2D([9050.804_7, -1436.746, -1482.837, -9626.756, 123.25, -44.5]);
        let inverse = invert_parent_world_like_cpp(matrix).expect("matrix is invertible");

        assert_eq!(
            inverse.0.map(f32::to_bits),
            [
                0x38e2_2db1,
                0xb787_062b,
                0xb78b_5b0f,
                0xb8d4_a58a,
                0xbc65_e5a8,
                0xbb25_b2c3,
            ],
            "pinned Mat2D::invert rounds each multiply before subtraction",
        );
        assert_ne!(
            matrix.determinant().to_bits(),
            (matrix.0[0] * matrix.0[3] - matrix.0[1] * matrix.0[2]).to_bits(),
            "the fixture must distinguish the shared fused helper from pinned C++ arithmetic",
        );
    }

    #[test]
    fn align_target_matches_pointer_preserve_replace_and_invalid_matrix() {
        let mut artboard = artboard();
        artboard
            .component_mut(1)
            .expect("parent node")
            .transform
            .world_transform = Mat2D([2.0, 0.0, 0.0, 4.0, 10.0, 20.0]);

        let replace = RuntimeListenerAlignTarget::for_test(0, Some(2), false);
        assert!(replace.perform(&mut artboard, &pointer(18.0, 32.0, 14.0, 24.0)));
        assert_eq!(position(&artboard), (4.0, 3.0));

        let preserve = RuntimeListenerAlignTarget::for_test(0, Some(2), true);
        assert!(preserve.perform(&mut artboard, &pointer(22.0, 40.0, 18.0, 32.0)));
        assert_eq!(position(&artboard), (6.0, 5.0));

        assert!(replace.perform(&mut artboard, &ScriptListenerInvocation::None));
        assert_eq!(position(&artboard), (-5.0, -5.0));

        let before = position(&artboard);
        artboard
            .component_mut(1)
            .expect("parent node")
            .transform
            .world_transform = Mat2D([0.0, 0.0, 0.0, 0.0, 10.0, 20.0]);
        assert!(!replace.perform(&mut artboard, &pointer(1.0, 2.0, 0.0, 0.0)));
        assert_eq!(position(&artboard), before);

        assert!(
            !RuntimeListenerAlignTarget::for_test(0, None, false)
                .perform(&mut artboard, &pointer(1.0, 2.0, 0.0, 0.0))
        );
        assert!(
            !RuntimeListenerAlignTarget::for_test(0, Some(5), false)
                .perform(&mut artboard, &pointer(1.0, 2.0, 0.0, 0.0))
        );

        let immediate_non_world_parent = RuntimeListenerAlignTarget::for_test(0, Some(4), false);
        assert!(
            !artboard
                .component(3)
                .expect("custom property group")
                .capabilities
                .world_transform
        );
        assert!(
            immediate_non_world_parent.perform(&mut artboard, &pointer(18.0, 32.0, 14.0, 24.0))
        );
        let x = property_key_for_name("Node", "x").expect("Node.x");
        let y = property_key_for_name("Node", "y").expect("Node.y");
        assert_eq!(
            (
                artboard.double_property(4, x).expect("nested target x"),
                artboard.double_property(4, y).expect("nested target y"),
            ),
            (18.0, 32.0),
            "getParentWorld uses identity for an immediate non-world-transform parent"
        );
    }
}
