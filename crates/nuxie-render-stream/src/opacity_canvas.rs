//! Sequential canvas execution. Call only when no destination frame is open.
use crate::{opacity, Command, Frame, LoadedResources, RenderStream, ReplayError};
use nuxie_render_api::{BlendMode, Factory, ImageSampler, RenderCanvas, Renderer};
use std::collections::HashMap;

impl RenderStream {
    /// Render to an owned texture, preparing nested opacity groups child-first.
    /// The caller must have no active frame on this factory's render context.
    /// Uses viewport-sized textures; allocation is limited to 256 MiB of RGBA
    /// pixels. The returned canvas retains the output image's backend owner.
    pub fn render_frame_to_canvas(
        &self,
        frame_index: usize,
        factory: &mut dyn Factory,
    ) -> Result<Box<dyn RenderCanvas>, ReplayError> {
        let frame = self.frames.get(frame_index).ok_or(ReplayError::MissingFrame(frame_index))?;
        let plan = opacity::plan(frame)?;
        let (width, height) = self.frame_size.filter(|&(w, h)| w > 0 && h > 0)
            .ok_or(ReplayError::UnsupportedOperation("opacity canvas requires nonzero frameSize"))?;
        let bytes = u64::from(width).checked_mul(u64::from(height))
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_mul(plan.groups.len() as u64 + 1));
        if bytes.is_none_or(|n| n > 256 * 1024 * 1024) {
            return Err(ReplayError::UnsupportedOperation("opacity canvas pixel budget exceeded"));
        }
        // Device-space composition requires undoing the inherited transform.
        // Keep this restriction explicit until singular-transform handling exists.
        for (command, op) in frame.commands.iter().enumerate() {
            if let Command::Transform(matrix) = op {
                if matrix.invert().is_none_or(|inverse| inverse.0.iter().any(|v| !v.is_finite())) {
                    return Err(ReplayError::InvalidOpacityGroup {
                        command, reason: "canvas execution requires finite invertible transforms",
                    });
                }
            }
        }
        let resources = LoadedResources::new(&self.resources, factory)?;
        let mut canvases = HashMap::new();
        for group in &plan.groups {
            let mut canvas = factory.make_render_canvas(width, height).map_err(ReplayError::Canvas)?;
            let mut target = canvas.begin_compositing_frame(0).map_err(ReplayError::Canvas)?;
            let result = (|| {
                // Ancestor clips and modulation are applied once when compositing.
                for index in state_commands(&plan, group) {
                    if matches!(frame.commands[index], Command::Transform(_)) {
                        resources.execute(&frame.commands[index], factory, target.renderer())?;
                    }
                }
                execute(frame, group.begin + 1..group.end, &plan, &canvases,
                    &resources, factory, target.renderer())
            })();
            // Close the shared context frame even when a draw reports an error.
            let finished = target.finish().map_err(ReplayError::Canvas);
            result?;
            finished?;
            canvases.insert(group.begin, (group, canvas));
        }
        let mut canvas = factory.make_render_canvas(width, height).map_err(ReplayError::Canvas)?;
        let mut target = canvas.begin_compositing_frame(self.clear_color.unwrap_or(0)).map_err(ReplayError::Canvas)?;
        let result = execute(frame, 0..frame.commands.len(), &plan, &canvases,
            &resources, factory, target.renderer());
        let finished = target.finish().map_err(ReplayError::Canvas);
        result?;
        finished?;
        Ok(canvas)
    }
}

fn state_commands(plan: &opacity::Plan, group: &opacity::Group) -> Vec<usize> {
    let mut state = group.inherited_state;
    let mut commands = Vec::new();
    while let Some(index) = state {
        commands.push(plan.states[index].command);
        state = plan.states[index].previous;
    }
    commands.reverse();
    commands
}

fn execute(
    frame: &Frame,
    range: std::ops::Range<usize>,
    plan: &opacity::Plan,
    canvases: &HashMap<usize, (&opacity::Group, Box<dyn RenderCanvas>)>,
    resources: &LoadedResources,
    factory: &mut dyn Factory,
    renderer: &mut dyn Renderer,
) -> Result<(), ReplayError> {
    let mut index = range.start;
    while index < range.end {
        if let Command::BeginOpacity(alpha) = frame.commands[index] {
            // The prepared entry retains its plan node: avoid scanning every
            // group for every sibling in documents with many opacity groups.
            let (group, canvas) = canvases.get(&index).expect("child prepared first");
            renderer.save();
            // The prepared texture is already in device coordinates. Existing
            // clips retain their captured coordinate systems across this reset.
            for command in state_commands(plan, group).into_iter().rev() {
                if let Command::Transform(matrix) = frame.commands[command] {
                    renderer.transform(matrix.invert().expect("preflighted transform"));
                }
            }
            let image = canvas.render_image();
            renderer.draw_image(Some(image.as_ref()), ImageSampler::default(), BlendMode::SrcOver, alpha);
            renderer.restore();
            index = group.end + 1;
        } else {
            resources.execute(&frame.commands[index], factory, renderer)?;
            index += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_render_api::{Mat2D, RecordingFactory};

    #[test]
    fn canvas_preflight_rejects_invalid_extents_budget_and_transform_without_output() {
        for (size, transform) in [
            (None, Mat2D::IDENTITY),
            (Some((0, 16)), Mat2D::IDENTITY),
            (Some((u32::MAX, u32::MAX)), Mat2D::IDENTITY),
            (Some((8192, 8192)), Mat2D::IDENTITY),
            (Some((16, 16)), Mat2D([0.; 6])),
            (Some((16, 16)), Mat2D([f32::NAN; 6])),
        ] {
            let stream = RenderStream {
                frame_size: size, clear_color: None, resources: vec![],
                frames: vec![Frame { commands: vec![Command::Transform(transform),
                    Command::BeginOpacity(0.5), Command::EndOpacity] }],
            };
            let mut factory = RecordingFactory::new();
            let before = factory.stream();
            assert!(stream.render_frame_to_canvas(0, &mut factory).is_err());
            assert_eq!(factory.stream(), before);
        }
    }

    #[test]
    fn pixel_budget_counts_root_and_every_nested_or_sibling_surface() {
        // 1024² RGBA consumes 4 MiB per surface. The root plus 63 groups
        // fits exactly; one additional group or one extra column does not.
        for nested in [false, true] {
            for (groups, width, exceeds) in [(62, 1024, false), (63, 1024, false),
                (64, 1024, true), (63, 1025, true)] {
                let mut commands = Vec::new();
                if nested {
                    commands.extend((0..groups).map(|_| Command::BeginOpacity(0.5)));
                    commands.extend((0..groups).map(|_| Command::EndOpacity));
                } else {
                    for _ in 0..groups {
                        commands.extend([Command::BeginOpacity(0.5), Command::EndOpacity]);
                    }
                }
                let stream = RenderStream { frame_size: Some((width, 1024)), clear_color: None,
                    resources: vec![], frames: vec![Frame { commands }] };
                // This factory cannot allocate canvases, so reaching its error
                // proves preflight accepted the exact boundary without a GPU allocation.
                let mut factory = RecordingFactory::new();
                let before = factory.stream();
                let result = stream.render_frame_to_canvas(0, &mut factory);
                if exceeds {
                    assert!(matches!(result, Err(ReplayError::UnsupportedOperation(
                        "opacity canvas pixel budget exceeded"))), "nested={nested}, groups={groups}, width={width}");
                } else {
                    assert!(matches!(result, Err(ReplayError::Canvas(_))),
                        "nested={nested}, groups={groups}, width={width}");
                }
                assert_eq!(factory.stream(), before, "preflight must not paint");
            }
        }
    }

    #[test]
    fn unsupported_canvas_factory_reports_error() {
        let stream = RenderStream { frame_size: Some((16, 16)), clear_color: None,
            resources: vec![], frames: vec![Frame { commands: vec![
                Command::BeginOpacity(0.5), Command::EndOpacity] }] };
        assert!(matches!(stream.render_frame_to_canvas(0, &mut RecordingFactory::new()),
            Err(ReplayError::Canvas(_))));
    }
}
