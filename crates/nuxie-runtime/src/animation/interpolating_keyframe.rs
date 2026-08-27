// Mirrors src/animation/interpolating_keyframe.cpp and its primary header.

fn normalized_interpolator_id(object: &RuntimeObject) -> Option<u64> {
    object
        .uint_property("interpolatorId")
        .filter(|id| *id != u64::from(u32::MAX) && *id != u64::MAX)
}

// C++ InterpolatingKeyFrame::onAddedDirty resolves the authored artboard-local
// interpolator id and rejects missing objects or objects of the wrong type.
fn runtime_key_frame_interpolator(
    file: &RuntimeFile,
    artboard_index: usize,
    key_frame: &RuntimeObject,
) -> Option<RuntimeInterpolator> {
    let local_index = usize::try_from(normalized_interpolator_id(key_frame)?).ok()?;
    let interpolator = file.artboard_local_object(artboard_index, local_index)?;
    RuntimeInterpolator::from_object(interpolator)
}

fn key_frame_interpolator_id_resolves_to_expected_type(
    file: &RuntimeFile,
    artboard_index: usize,
    key_frame: &RuntimeObject,
) -> bool {
    let Some(local_index) =
        normalized_interpolator_id(key_frame).and_then(|id| usize::try_from(id).ok())
    else {
        return false;
    };
    file.artboard_local_object(artboard_index, local_index)
        .and_then(|interpolator| definition_by_type_key(interpolator.type_key))
        .is_some_and(|definition| definition.is_a("KeyFrameInterpolator"))
}

// Rust retains the effective ScriptedInterpolator choice as its execution
// context rather than cloning the polymorphic interpolator object itself.
#[derive(Clone, Copy)]
enum RuntimeScriptedInterpolationContext<'a> {
    Shared(&'a ArtboardInstance),
    Stateful(&'a LinearAnimationInstance, &'a ArtboardInstance),
}

fn effective_scripted_interpolation_context<'a>(
    animation: Option<&'a LinearAnimationInstance>,
    artboard: &'a ArtboardInstance,
) -> RuntimeScriptedInterpolationContext<'a> {
    match animation {
        Some(animation) => RuntimeScriptedInterpolationContext::Stateful(animation, artboard),
        None => RuntimeScriptedInterpolationContext::Shared(artboard),
    }
}

impl RuntimeScriptedInterpolationContext<'_> {
    fn evaluate(
        self,
        key_frame_global_id: u32,
        interpolator_global_id: u32,
        method: ScriptInterpolatorMethod,
        arguments: &[f32],
        fallback: f32,
    ) -> f32 {
        match self {
            Self::Shared(artboard) => artboard.evaluate_shared_scripted_interpolator(
                key_frame_global_id,
                interpolator_global_id,
                method,
                arguments,
                fallback,
            ),
            Self::Stateful(animation, artboard) => animation.evaluate_scripted_interpolator(
                artboard,
                key_frame_global_id,
                interpolator_global_id,
                method,
                arguments,
                fallback,
            ),
        }
    }
}
