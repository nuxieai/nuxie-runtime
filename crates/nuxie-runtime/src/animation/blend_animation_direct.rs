// Mirrors src/animation/blend_animation_direct.cpp and
// include/rive/animation/blend_animation_direct.hpp.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendAnimationDirect {
    pub(crate) animation: RuntimeLinearAnimationHandle,
    pub(crate) source: RuntimeDirectBlendSource,
}

impl RuntimeBlendAnimation for RuntimeBlendAnimationDirect {
    fn retained_animation(&self) -> RuntimeLinearAnimationHandle {
        self.animation
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RuntimeDirectBlendSource {
    Input { input_index: usize },
    MixValue { value: f32 },
    BindableProperty { global_id: Option<u32> },
}

impl Default for RuntimeBlendAnimationDirect {
    fn default() -> Self {
        Self {
            animation: RuntimeLinearAnimationHandle::empty(),
            source: RuntimeDirectBlendSource::Input {
                input_index: usize::MAX,
            },
        }
    }
}

impl Drop for RuntimeBlendAnimationDirect {
    fn drop(&mut self) {
        if self.bindable_property().is_some() {
            self.set_bindable_property(None);
        }
    }
}

impl RuntimeBlendAnimationDirect {
    #[allow(dead_code)]
    pub(crate) fn on_added_dirty(&self) -> bool {
        true
    }

    #[allow(dead_code)]
    pub(crate) fn on_added_clean(&self) -> bool {
        true
    }

    pub(crate) fn from_imported(
        file: &RuntimeFile,
        object: &RuntimeObject,
        animation: RuntimeLinearAnimationHandle,
    ) -> Self {
        let blend_source = object.uint_property("blendSource").unwrap_or(0);
        let source = if blend_source == 0 {
            RuntimeDirectBlendSource::Input {
                input_index: object
                    .uint_property("inputId")
                    .and_then(|input_id| usize::try_from(input_id).ok())
                    .unwrap_or(usize::MAX),
            }
        } else if blend_source == 2 {
            RuntimeDirectBlendSource::BindableProperty {
                global_id: file
                    .latest_bindable_property_for_object(object)
                    .map(|property| property.id as u32),
            }
        } else if blend_source == 1 {
            RuntimeDirectBlendSource::MixValue {
                value: object.double_property("mixValue").unwrap_or(100.0),
            }
        } else {
            RuntimeDirectBlendSource::Input {
                input_index: object
                    .uint_property("inputId")
                    .and_then(|input_id| usize::try_from(input_id).ok())
                    .unwrap_or(usize::MAX),
            }
        };
        Self { animation, source }
    }

    pub(crate) fn set_bindable_property(&mut self, value: Option<u32>) {
        if let RuntimeDirectBlendSource::BindableProperty { global_id } = &mut self.source {
            *global_id = value;
        }
    }

    pub(crate) fn bindable_property(&self) -> Option<u32> {
        match self.source {
            RuntimeDirectBlendSource::BindableProperty { global_id } => global_id,
            _ => None,
        }
    }
}
