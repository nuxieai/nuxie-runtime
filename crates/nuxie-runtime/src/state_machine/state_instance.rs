use super::system_state_instance::RuntimeSystemStateInstance;
use super::*;

/// One mutable occurrence of an immutable `RuntimeLayerState` definition.
///
/// This corresponds to pinned C++ `StateInstance`: the definition handle is
/// retained once, and animation/blend mutation lives inside the occurrence
/// instead of parallel fields on the layer.
#[derive(Debug, Clone)]
pub(super) struct RuntimeStateInstance {
    state_index: usize,
    kind: RuntimeStateInstanceKind,
}

#[derive(Debug, Clone)]
enum RuntimeStateInstanceKind {
    System(RuntimeSystemStateInstance),
    Animation {
        animation: LinearAnimationInstance,
        keep_going: bool,
    },
    Blend1D(BlendState1DInstance),
    BlendDirect(BlendStateDirectInstance),
}

impl RuntimeStateInstance {
    pub(super) fn make(
        layer: &RuntimeStateMachineLayer,
        state_index: usize,
        artboard: &ArtboardInstance,
        animation_definitions: &Arc<Vec<RuntimeLinearAnimation>>,
        empty_animation_definition: &Arc<RuntimeLinearAnimation>,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) -> Option<Self> {
        let state = layer.states.get(state_index)?;
        let kind = if let Some(definition) = state.blend_state_1d.as_ref() {
            let mut occurrence = BlendState1DInstance::new(
                definition,
                artboard,
                animation_definitions,
                empty_animation_definition,
                state.resets_blend_values(),
            );
            occurrence.advance(definition, artboard, inputs, bindable_numbers, 0.0);
            RuntimeStateInstanceKind::Blend1D(occurrence)
        } else if let Some(definition) = state.blend_state_direct.as_ref() {
            let mut occurrence = BlendStateDirectInstance::new(
                definition,
                animation_definitions,
                empty_animation_definition,
            );
            occurrence.advance(definition, artboard, inputs, bindable_numbers, 0.0);
            RuntimeStateInstanceKind::BlendDirect(occurrence)
        } else if let Some(handle) = state.animation {
            let mut occurrence = LinearAnimationInstance::new(
                handle,
                Arc::clone(animation_definitions),
                Arc::clone(empty_animation_definition),
                state.speed,
            )?;
            let keep_going = occurrence.advance(0.0);
            RuntimeStateInstanceKind::Animation {
                animation: occurrence,
                keep_going,
            }
        } else {
            RuntimeStateInstanceKind::System(RuntimeSystemStateInstance)
        };
        Some(Self { state_index, kind })
    }

    pub(super) fn state_index(&self) -> usize {
        self.state_index
    }

    pub(super) fn state<'a>(
        &self,
        layer: &'a RuntimeStateMachineLayer,
    ) -> Option<&'a RuntimeLayerState> {
        layer.states.get(self.state_index)
    }

    pub(super) fn is_same_definition(&self, state_index: usize) -> bool {
        self.state_index == state_index
    }

    pub(super) fn keep_going(&self) -> bool {
        match &self.kind {
            RuntimeStateInstanceKind::System(_) => false,
            RuntimeStateInstanceKind::Animation { keep_going, .. } => *keep_going,
            RuntimeStateInstanceKind::Blend1D(_) | RuntimeStateInstanceKind::BlendDirect(_) => true,
        }
    }

    pub(super) fn advance(
        &mut self,
        layer: &RuntimeStateMachineLayer,
        artboard: &mut ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        let Some(state) = layer.states.get(self.state_index) else {
            return false;
        };
        match &mut self.kind {
            RuntimeStateInstanceKind::System(instance) => instance.advance(),
            RuntimeStateInstanceKind::Animation {
                animation,
                keep_going,
            } => {
                *keep_going = artboard.advance_linear_animation_instance_with_events(
                    animation,
                    elapsed_seconds * state.speed,
                    reported_events,
                );
                *keep_going
            }
            RuntimeStateInstanceKind::Blend1D(instance) => {
                let Some(definition) = state.blend_state_1d.as_ref() else {
                    return false;
                };
                instance.advance_with_events(
                    definition,
                    artboard,
                    inputs,
                    bindable_numbers,
                    elapsed_seconds,
                    reported_events,
                )
            }
            RuntimeStateInstanceKind::BlendDirect(instance) => {
                let Some(definition) = state.blend_state_direct.as_ref() else {
                    return false;
                };
                instance.advance_with_events(
                    definition,
                    artboard,
                    inputs,
                    bindable_numbers,
                    elapsed_seconds,
                    reported_events,
                )
            }
        }
    }

    pub(super) fn apply(&mut self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        match &mut self.kind {
            RuntimeStateInstanceKind::System(instance) => instance.apply(),
            RuntimeStateInstanceKind::Animation { animation, .. } => animation.apply(artboard, mix),
            RuntimeStateInstanceKind::Blend1D(instance) => instance.apply(artboard, mix),
            RuntimeStateInstanceKind::BlendDirect(instance) => instance.apply(artboard, mix),
        }
    }

    pub(super) fn plain_animation(&self) -> Option<&LinearAnimationInstance> {
        match &self.kind {
            RuntimeStateInstanceKind::Animation { animation, .. } => Some(animation),
            _ => None,
        }
    }

    pub(super) fn plain_animation_mut(&mut self) -> Option<&mut LinearAnimationInstance> {
        match &mut self.kind {
            RuntimeStateInstanceKind::Animation { animation, .. } => Some(animation),
            _ => None,
        }
    }

    pub(super) fn transition_animation(
        &self,
        blend_animation_index: Option<usize>,
    ) -> Option<&LinearAnimationInstance> {
        match &self.kind {
            RuntimeStateInstanceKind::Animation { animation, .. } => Some(animation),
            RuntimeStateInstanceKind::Blend1D(instance) => {
                instance.animation_instance(blend_animation_index?)
            }
            RuntimeStateInstanceKind::BlendDirect(instance) => {
                instance.animation_instance(blend_animation_index?)
            }
            RuntimeStateInstanceKind::System(_) => None,
        }
    }

    pub(super) fn spilled_time(&self) -> f32 {
        self.plain_animation()
            .map(LinearAnimationInstance::spilled_time)
            .unwrap_or(0.0)
    }

    pub(super) fn clear_spilled_time(&mut self) {
        if let Some(animation) = self.plain_animation_mut() {
            animation.clear_spilled_time();
        }
    }

    pub(super) fn build_key_frame_data_binds(
        &mut self,
        graphs: &[Option<crate::RuntimeDataBindGraph>],
        enrollment: crate::animation::RuntimeKeyFrameDataBindEnrollment,
    ) {
        self.for_each_animation_instance_mut(|instance| {
            let prototype = graphs
                .get(instance.animation_index())
                .and_then(Option::as_ref);
            if let Some(prototype) = prototype {
                instance.build_key_frame_data_binds(prototype, enrollment);
            }
        });
    }

    pub(super) fn collect_key_frame_data_bind_occurrence_ids(
        &mut self,
        enrollment: crate::animation::RuntimeKeyFrameDataBindEnrollment,
        ids: &mut Vec<crate::animation::RuntimeKeyFrameDataBindOccurrenceId>,
    ) {
        self.for_each_animation_instance_mut(|instance| {
            ids.extend(instance.key_frame_data_bind_occurrence_ids(enrollment));
        });
    }

    pub(super) fn ensure_key_frame_data_binds(
        &mut self,
        graphs: &[Option<crate::RuntimeDataBindGraph>],
    ) {
        self.for_each_animation_instance_mut(|instance| {
            let Some(prototype) = graphs
                .get(instance.animation_index())
                .and_then(Option::as_ref)
            else {
                return;
            };
            instance.ensure_key_frame_data_binds(prototype);
        });
    }

    pub(super) fn enroll_unassigned_key_frame_data_binds(&mut self, next_id: &mut u64) {
        self.for_each_animation_instance_mut(|instance| {
            instance.enroll_unassigned_key_frame_data_binds(next_id);
        });
    }

    pub(super) fn prepare_key_frame_data_bind_occurrence(
        &mut self,
        occurrence_id: crate::animation::RuntimeKeyFrameDataBindOccurrenceId,
        graphs: &[Option<crate::RuntimeDataBindGraph>],
    ) -> Option<bool> {
        let mut result = None;
        self.for_each_animation_instance_mut(|instance| {
            if result.is_some() {
                return;
            }
            let Some(prototype) = graphs
                .get(instance.animation_index())
                .and_then(Option::as_ref)
            else {
                return;
            };
            result = instance.prepare_key_frame_data_bind_occurrence(occurrence_id, prototype);
        });
        result
    }

    pub(super) fn advance_key_frame_data_bind_occurrence(
        &mut self,
        occurrence_id: crate::animation::RuntimeKeyFrameDataBindOccurrenceId,
        graphs: &[Option<crate::RuntimeDataBindGraph>],
        elapsed_seconds: f32,
    ) -> Option<bool> {
        let mut result = None;
        self.for_each_animation_instance_mut(|instance| {
            if result.is_some() {
                return;
            }
            let Some(prototype) = graphs
                .get(instance.animation_index())
                .and_then(Option::as_ref)
            else {
                return;
            };
            result = instance.advance_key_frame_data_bind_occurrence(
                occurrence_id,
                prototype,
                elapsed_seconds,
            );
        });
        result
    }

    pub(super) fn remove_key_frame_data_binds(&mut self) {
        self.for_each_animation_instance_mut(LinearAnimationInstance::remove_key_frame_data_binds);
    }

    pub(super) fn for_each_animation_instance_mut(
        &mut self,
        mut callback: impl FnMut(&mut LinearAnimationInstance),
    ) {
        match &mut self.kind {
            RuntimeStateInstanceKind::System(_) => {}
            RuntimeStateInstanceKind::Animation { animation, .. } => callback(animation),
            RuntimeStateInstanceKind::Blend1D(instance) => {
                instance.for_each_animation_instance_mut(callback)
            }
            RuntimeStateInstanceKind::BlendDirect(instance) => {
                instance.for_each_animation_instance_mut(callback)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_binary::FixtureRecord;
    use nuxie_graph::GraphFile;

    struct AnimationStateFixture {
        layer: RuntimeStateMachineLayer,
        occurrence: RuntimeStateInstance,
        artboard: ArtboardInstance,
    }

    impl AnimationStateFixture {
        fn new(
            duration: u64,
            fps: u64,
            animation_speed: f32,
            loop_value: u64,
            state_speed: f32,
        ) -> Self {
            let file = RuntimeFile::from_fixture_records(vec![
                fixture_record("Backboard"),
                fixture_record("Artboard"),
            ])
            .expect("dummy upstream Artboard imports");
            let graph = GraphFile::from_runtime_file(&file).expect("dummy Artboard graph builds");
            let artboard = ArtboardInstance::from_graph(
                &file,
                graph.artboards.first().expect("dummy Artboard exists"),
            )
            .expect("dummy Artboard instance builds");

            let animation_definitions = Arc::new(vec![RuntimeLinearAnimation {
                global_id: 1,
                name: None,
                fps,
                duration,
                speed: animation_speed,
                loop_value,
                work_start: 0,
                work_end: 0,
                enable_work_area: false,
                quantize: false,
                keyed_objects: Arc::new(Vec::new()),
                key_frame_data_bind_templates: Arc::new(Vec::new()),
                has_keyed_callbacks: false,
            }]);
            let empty_animation_definition = Arc::new(RuntimeLinearAnimation::empty());
            let layer = RuntimeStateMachineLayer {
                global_id: 1,
                name: None,
                states: vec![RuntimeLayerState {
                    global_id: None,
                    type_name: Some("AnimationState"),
                    animation: Some(RuntimeLinearAnimationHandle::new(0)),
                    blend_state_1d: None,
                    blend_state_direct: None,
                    speed: state_speed,
                    flags: 0,
                    fire_actions: Vec::new(),
                    listener_actions: Vec::new(),
                    transitions: Vec::new(),
                }],
                entry_state_index: None,
                any_state_index: None,
                exit_state_index: None,
            };
            let occurrence = RuntimeStateInstance::make(
                &layer,
                0,
                &artboard,
                &animation_definitions,
                &empty_animation_definition,
                &[],
                &[],
            )
            .expect("AnimationState creates an AnimationStateInstance occurrence");
            Self {
                layer,
                occurrence,
                artboard,
            }
        }

        fn advance(&mut self, elapsed_seconds: f32) {
            self.occurrence.advance(
                &self.layer,
                &mut self.artboard,
                &[],
                &[],
                elapsed_seconds,
                &mut Vec::new(),
            );
        }

        fn animation_instance(&self) -> &LinearAnimationInstance {
            self.occurrence
                .plain_animation()
                .expect("AnimationState occurrence retains its LinearAnimationInstance")
        }
    }

    fn fixture_record(type_name: &str) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing {type_name}"))
                .type_key
                .int,
            properties: Vec::new(),
        }
    }

    #[test]
    fn upstream_animation_state_instance_01_advances_in_step_with_animation_speed_1() {
        let mut fixture = AnimationStateFixture::new(10, 2, 1.0, 0, 1.0);
        fixture.advance(2.0);
        assert_eq!(fixture.animation_instance().time(), 2.0);
        assert_eq!(fixture.animation_instance().total_time(), 2.0);
    }

    #[test]
    fn upstream_animation_state_instance_02_advances_twice_as_fast_when_speed_is_doubled() {
        let mut fixture = AnimationStateFixture::new(10, 2, 1.0, 0, 2.0);
        fixture.advance(2.0);
        assert_eq!(fixture.animation_instance().time(), 4.0);
        assert_eq!(fixture.animation_instance().total_time(), 4.0);
    }

    #[test]
    fn upstream_animation_state_instance_03_advances_half_as_fast_when_speed_is_halved() {
        let mut fixture = AnimationStateFixture::new(10, 2, 1.0, 0, 0.5);
        fixture.advance(2.0);
        assert_eq!(fixture.animation_instance().time(), 1.0);
        assert_eq!(fixture.animation_instance().total_time(), 1.0);
    }

    #[test]
    fn upstream_animation_state_instance_04_advances_backwards_when_speed_is_negative() {
        let mut fixture = AnimationStateFixture::new(10, 2, 1.0, 1, -1.0);
        fixture.advance(2.0);
        assert_eq!(fixture.animation_instance().time(), 3.0);
        assert_eq!(fixture.animation_instance().total_time(), 2.0);
    }

    #[test]
    fn upstream_animation_state_instance_05_starts_a_positive_animation_at_the_beginning() {
        let fixture = AnimationStateFixture::new(10, 2, 1.0, 0, 1.0);
        assert_eq!(fixture.animation_instance().time(), 0.0);
    }

    #[test]
    fn upstream_animation_state_instance_06_starts_a_negative_animation_at_the_end() {
        let fixture = AnimationStateFixture::new(10, 2, -1.0, 0, 1.0);
        assert_eq!(fixture.animation_instance().time(), 5.0);
    }

    #[test]
    fn upstream_animation_state_instance_07_negative_state_starts_positive_animation_at_end() {
        let fixture = AnimationStateFixture::new(10, 2, 1.0, 0, -1.0);
        assert_eq!(fixture.animation_instance().time(), 5.0);
    }

    #[test]
    fn upstream_animation_state_instance_08_negative_state_starts_negative_animation_at_beginning()
    {
        let fixture = AnimationStateFixture::new(10, 2, -1.0, 0, -1.0);
        assert_eq!(fixture.animation_instance().time(), 0.0);
    }

    #[test]
    fn upstream_animation_state_instance_09_spilled_time_nx_one_shot() {
        let mut fixture = AnimationStateFixture::new(4, 2, 2.0, 0, 1.0);
        fixture.advance(3.0);
        assert_eq!(fixture.animation_instance().time(), 2.0);
        assert_eq!(fixture.animation_instance().total_time(), 6.0);
        assert_eq!(fixture.animation_instance().spilled_time(), 2.0);
    }

    #[test]
    fn upstream_animation_state_instance_10_spilled_time_inverse_nx_one_shot() {
        let mut fixture = AnimationStateFixture::new(4, 2, 0.5, 0, 1.0);
        fixture.advance(5.0);
        assert_eq!(fixture.animation_instance().time(), 2.0);
        assert_eq!(fixture.animation_instance().total_time(), 2.5);
        assert_eq!(fixture.animation_instance().spilled_time(), 1.0);
    }

    #[test]
    fn upstream_animation_state_instance_11_spilled_time_nx_loop() {
        let mut fixture = AnimationStateFixture::new(4, 2, 2.0, 1, 1.0);
        fixture.advance(5.5);
        assert_eq!(fixture.animation_instance().time(), 1.0);
        assert_eq!(fixture.animation_instance().total_time(), 11.0);
        assert_eq!(fixture.animation_instance().spilled_time(), 0.5);
    }

    #[test]
    fn upstream_animation_state_instance_12_spilled_time_inverse_nx_loop() {
        let mut fixture = AnimationStateFixture::new(4, 2, 0.5, 1, 1.0);
        fixture.advance(10.0);
        assert_eq!(fixture.animation_instance().time(), 1.0);
        assert_eq!(fixture.animation_instance().total_time(), 5.0);
        assert_eq!(fixture.animation_instance().spilled_time(), 2.0);
    }

    #[test]
    fn upstream_animation_state_instance_13_spilled_time_negative_nx_one_shot() {
        let mut fixture = AnimationStateFixture::new(4, 2, -2.0, 0, 1.0);
        fixture.advance(3.0);
        assert_eq!(fixture.animation_instance().time(), 0.0);
        assert_eq!(fixture.animation_instance().total_time(), 6.0);
        assert_eq!(fixture.animation_instance().spilled_time(), 2.0);
    }

    #[test]
    fn upstream_animation_state_instance_14_spilled_time_negative_nx_loop() {
        let mut fixture = AnimationStateFixture::new(4, 2, -2.0, 1, 1.0);
        fixture.advance(5.5);
        assert_eq!(fixture.animation_instance().time(), 1.0);
        assert_eq!(fixture.animation_instance().total_time(), 11.0);
        assert_eq!(fixture.animation_instance().spilled_time(), 0.5);
    }
}
