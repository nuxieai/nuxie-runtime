//! Structural preparation for isolated opacity. Ranges are in child-first order.
use crate::{Command, Frame, ReplayError};

const MAX_GROUP_DEPTH: usize = 64;

#[derive(Debug, PartialEq)]
pub(crate) struct Group {
    pub begin: usize,
    pub end: usize,
    pub opacity: f32,
    /// Effective state at entry; outer clips must be applied at composition,
    /// rather than baked into the texture and applied a second time.
    pub inherited_state: Option<usize>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct StateNode {
    pub command: usize,
    pub previous: Option<usize>,
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Plan {
    pub groups: Vec<Group>,
    /// Persistent state chains keep planning linear even with many siblings.
    pub states: Vec<StateNode>,
}

pub(crate) fn plan(frame: &Frame) -> Result<Plan, ReplayError> {
    // Preserve the existing flat-stream state contract.
    if !frame.commands.iter().any(|c| matches!(c, Command::BeginOpacity(_) | Command::EndOpacity)) {
        return Ok(Plan::default());
    }
    let mut stack = Vec::new();
    let mut result = Plan::default();
    let mut saves = Vec::new();
    let mut state = None;
    for (command, op) in frame.commands.iter().enumerate() {
        let invalid = |reason| ReplayError::InvalidOpacityGroup { command, reason };
        match *op {
            Command::BeginOpacity(opacity) => {
                if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
                    return Err(invalid("alpha must be finite and between zero and one"));
                }
                if stack.len() == MAX_GROUP_DEPTH {
                    return Err(invalid("group nesting exceeds 64"));
                }
                stack.push((command, opacity, saves.len(), state));
            }
            Command::EndOpacity => {
                let Some((begin, opacity, entry_saves, inherited_state)) = stack.pop() else {
                    return Err(invalid("end has no matching begin"));
                };
                if saves.len() != entry_saves {
                    return Err(invalid("unrestored state inside group"));
                }
                result.groups.push(Group { begin, end: command, opacity, inherited_state });
                // A group is an implicit state scope, independent of explicit saves.
                state = inherited_state;
            }
            Command::Save => saves.push(state),
            Command::Restore => {
                if saves.is_empty() || stack.last().is_some_and(|&(_, _, entry, _)| saves.len() <= entry) {
                    return Err(invalid("restore crosses a group or frame boundary"));
                }
                state = saves.pop().expect("save checked above");
            }
            Command::Transform(_) | Command::ClipPath(_) | Command::ClipOutRect(_)
            | Command::ClipAxis { .. } | Command::ModulateOpacity(_) => {
                result.states.push(StateNode { command, previous: state });
                state = Some(result.states.len() - 1);
            }
            _ => {}
        }
    }
    if let Some(&(command, _, _, _)) = stack.last() {
        return Err(ReplayError::InvalidOpacityGroup { command, reason: "unterminated group" });
    }
    if !saves.is_empty() {
        return Err(ReplayError::InvalidOpacityGroup {
            command: frame.commands.len(), reason: "unrestored frame state",
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderStream;
    use nuxie_render_api::RecordingFactory;

    #[test]
    fn nested_groups_prepare_children_before_parents() {
        let frame = Frame { commands: vec![Command::Save, Command::BeginOpacity(0.5),
            Command::BeginOpacity(0.25), Command::Save, Command::Restore,
            Command::EndOpacity, Command::EndOpacity, Command::Restore] };
        assert_eq!(plan(&frame).unwrap().groups, vec![
            Group { begin: 2, end: 5, opacity: 0.25, inherited_state: None },
            Group { begin: 1, end: 6, opacity: 0.5, inherited_state: None },
        ]);
    }

    #[test]
    fn state_capture_preserves_clip_order_and_restores_group_and_save_scopes() {
        use nuxie_render_api::{Aabb, Mat2D};
        let frame = Frame { commands: vec![
            Command::Transform(Mat2D([1.0, 0.0, 0.0, 1.0, 4.0, 5.0])), // 0
            Command::Save,
            Command::ClipOutRect(Aabb::new(1.0, 2.0, 3.0, 4.0)), // 2
            Command::BeginOpacity(0.5), // 3 inherits 0,2
            Command::ModulateOpacity(0.75), // 4
            Command::BeginOpacity(0.25), // 5 inherits 0,2,4
            Command::Transform(Mat2D([2.0, 0.0, 0.0, 2.0, 0.0, 0.0])), // 6
            Command::EndOpacity,
            Command::BeginOpacity(0.5), // 8 must not inherit 6
            Command::EndOpacity,
            Command::EndOpacity,
            Command::BeginOpacity(0.5), // 11 must not inherit 4
            Command::EndOpacity,
            Command::Restore,
            Command::BeginOpacity(0.5), // 14 must not inherit 2
            Command::EndOpacity,
        ] };
        let plan = plan(&frame).unwrap();
        let capture = |begin| {
            let mut cursor = plan.groups.iter().find(|g| g.begin == begin).unwrap().inherited_state;
            let mut commands = Vec::new();
            while let Some(index) = cursor {
                commands.push(plan.states[index].command);
                cursor = plan.states[index].previous;
            }
            commands.reverse();
            commands
        };
        assert_eq!(capture(3), vec![0, 2]);
        assert_eq!(capture(5), vec![0, 2, 4]);
        assert_eq!(capture(8), vec![0, 2, 4]);
        assert_eq!(capture(11), vec![0, 2]);
        assert_eq!(capture(14), vec![0]);
    }

    #[test]
    fn sibling_groups_share_state_without_quadratic_copies() {
        let mut commands = vec![Command::ModulateOpacity(1.0); 10_000];
        for _ in 0..10_000 {
            commands.extend([Command::BeginOpacity(0.5), Command::EndOpacity]);
        }
        let plan = plan(&Frame { commands }).unwrap();
        assert_eq!(plan.states.len(), 10_000);
        assert_eq!(plan.groups.len(), 10_000);
        assert!(plan.groups.iter().all(|g| g.inherited_state == Some(9_999)));
    }

    #[test]
    fn renderer_records_checked_groups_and_preserves_semantic_commands() {
        use nuxie_render_api::Renderer;
        let factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let before = factory.stream();
        assert!(!renderer.end_opacity_group());
        for alpha in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            assert!(!renderer.begin_opacity_group(alpha));
        }
        assert_eq!(factory.stream(), before);
        assert!(renderer.begin_opacity_group(0.5));
        assert!(renderer.begin_opacity_group(0.25));
        assert!(renderer.end_opacity_group());
        assert!(renderer.end_opacity_group());
        let stream = RenderStream::parse(&factory.stream()).unwrap();
        let groups = plan(&stream.frames[0]).unwrap().groups;
        assert_eq!(groups.iter().map(|g| g.opacity).collect::<Vec<_>>(), vec![0.25, 0.5]);
        for _ in 0..64 { assert!(renderer.begin_opacity_group(1.0)); }
        let before = factory.stream();
        assert!(!renderer.begin_opacity_group(1.0));
        assert_eq!(factory.stream(), before);
        for _ in 0..64 { assert!(renderer.end_opacity_group()); }
        assert!(!renderer.end_opacity_group());
    }

    #[test]
    fn malformed_groups_never_reach_renderer() {
        for commands in [
            vec![Command::EndOpacity],
            vec![Command::BeginOpacity(0.5)],
            vec![Command::BeginOpacity(f32::NAN), Command::EndOpacity],
            vec![Command::BeginOpacity(f32::INFINITY), Command::EndOpacity],
            vec![Command::BeginOpacity(-0.1), Command::EndOpacity],
            vec![Command::BeginOpacity(1.1), Command::EndOpacity],
            vec![Command::Save, Command::BeginOpacity(0.5), Command::Restore, Command::EndOpacity],
            vec![Command::BeginOpacity(0.5), Command::Save, Command::EndOpacity],
        ] {
            let stream = RenderStream { frame_size: None, clear_color: None,
                resources: vec![], frames: vec![Frame { commands }] };
            let mut factory = RecordingFactory::new();
            let mut renderer = factory.make_renderer();
            let before = factory.stream();
            assert!(matches!(stream.replay_frame(0, &mut factory, &mut renderer),
                Err(ReplayError::InvalidOpacityGroup { .. })));
            assert_eq!(factory.stream(), before);
        }
    }

    #[test]
    fn nesting_limit_and_zero_one_groups() {
        for depth in [64, 65] {
            let mut commands = vec![Command::BeginOpacity(0.0); depth];
            commands.extend(vec![Command::EndOpacity; depth]);
            assert_eq!(plan(&Frame { commands }).is_ok(), depth == 64);
        }
        assert_eq!(plan(&Frame { commands: vec![Command::BeginOpacity(1.0), Command::EndOpacity] })
            .unwrap().groups.len(), 1);
    }

    #[test]
    fn parsed_groups_require_preparation_without_partial_output() {
        let stream = RenderStream::parse("rive-golden-stream-v1\nsave\nbeginOpacity opacity=0.5\nendOpacity\nrestore\nframe\n").unwrap();
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let before = factory.stream();
        assert_eq!(stream.replay_frame(0, &mut factory, &mut renderer),
            Err(ReplayError::UnsupportedOperation("prepared opacity groups")));
        assert_eq!(factory.stream(), before);
    }
}
