use crate::mechanical_port::source::{
    core::{CoreHandle, CoreObject},
    generated::animation::keyframe_interpolator_base::KeyFrameInterpolatorBase,
    importers::{
        artboard_importer::ArtboardImporter, backboard_importer::BackboardImporter,
        import_stack::ImportStack,
    },
    status_code::StatusCode,
};
pub trait KeyFrameInterpolatorBehavior {
    fn transform_value(&mut self, from: f32, to: f32, factor: f32) -> f32;
    fn transform(&self, factor: f32) -> f32;
    fn initialize(&mut self) {}
    fn is_scripted(&self) -> bool {
        false
    }
}
#[derive(Default)]
pub struct KeyFrameInterpolator {
    pub base: KeyFrameInterpolatorBase,
    scripted: bool,
}
impl KeyFrameInterpolator {
    pub fn claims_artboard_slot(_import_stack: &mut ImportStack) -> bool {
        true
    }

    pub fn is_scripted(&self) -> bool {
        self.scripted
    }
    pub fn set_scripted(&mut self, value: bool) {
        self.scripted = value;
    }
    pub fn host_from(component: CoreHandle) -> Option<CoreHandle> {
        Self::is_host(&component).then_some(component)
    }

    /// host_from for a borrowed component, cloning only an actual host.
    pub(crate) fn host_of(component: &CoreHandle) -> Option<CoreHandle> {
        Self::is_host(component).then(|| component.clone())
    }

    fn is_host(component: &CoreHandle) -> bool {
        // Upstream switches on coreType: Artboard inherits LayoutComponent but is not a host.
        component.core_type()
            == Some(
                crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY,
            )
    }
    pub fn import(owner: &mut dyn CoreObject, stack: &mut ImportStack) -> StatusCode {
        if stack
            .latest::<ArtboardImporter>(
                crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
            )
            .is_none()
        {
            if let Some(importer) = stack.latest::<BackboardImporter>(
                crate::mechanical_port::source::generated::backboard_base::BackboardBase::TYPE_KEY,
            ) {
                importer.add_interpolator(owner);
            } else {
                return StatusCode::MissingObject;
            }
        }
        owner.core_mut().import(stack)
    }
    pub fn initialize(&mut self) {}
}
impl std::ops::Deref for KeyFrameInterpolator {
    type Target = KeyFrameInterpolatorBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for KeyFrameInterpolator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

/// A scripted interpolator's method on its script, through the handle:
/// `None` when `interpolator` is not scripted. The instance is read under a
/// shared borrow that ends before the script runs, so the script may reach
/// any object, this interpolator included.
fn scripted_call(
    interpolator: &CoreHandle,
    method: crate::scripting::ScriptInterpolatorMethod,
    args: &[f32],
    fallback: f32,
) -> Option<f32> {
    use crate::mechanical_port::source::scripted::{
        scripted_interpolator::ScriptedInterpolator, scripted_object::ScriptedObject,
    };
    let instance = interpolator.with_downcast::<ScriptedInterpolator, _>(|interpolator| {
        interpolator.scripted.interpolator_instance()
    })?;
    Some(
        instance
            .and_then(|instance| ScriptedObject::call_interpolator_instance(instance, method, args))
            .unwrap_or(fallback),
    )
}

/// `ScriptedInterpolator::transform` through the handle; `None` when
/// `interpolator` is not a ScriptedInterpolator.
pub fn scripted_transform_occurrence(interpolator: &CoreHandle, factor: f32) -> Option<f32> {
    scripted_call(
        interpolator,
        crate::scripting::ScriptInterpolatorMethod::Transform,
        &[factor],
        factor,
    )
}

/// `ScriptedInterpolator::transformValue` through the handle; `None` when
/// `interpolator` is not a ScriptedInterpolator.
pub fn scripted_transform_value_occurrence(
    interpolator: &CoreHandle,
    from: f32,
    to: f32,
    factor: f32,
) -> Option<f32> {
    scripted_call(
        interpolator,
        crate::scripting::ScriptInterpolatorMethod::TransformValue,
        &[from, to, factor],
        from + (to - from) * factor,
    )
}

/// `KeyFrameInterpolator::transform` through the handle: a scripted one runs
/// its script with no borrow of it (`scripted_transform_occurrence`); the
/// others transform under their own borrow as before.
pub fn transform_occurrence(interpolator: &CoreHandle, factor: f32) -> Option<f32> {
    scripted_transform_occurrence(interpolator, factor).or_else(|| {
        interpolator
            .with_mut(|interpolator| interpolator.keyframe_interpolator_transform(factor))
            .flatten()
    })
}

/// `KeyFrameInterpolator::transformValue` through the handle, as
/// [`transform_occurrence`].
pub fn transform_value_occurrence(
    interpolator: &CoreHandle,
    from: f32,
    to: f32,
    factor: f32,
) -> Option<f32> {
    scripted_transform_value_occurrence(interpolator, from, to, factor).or_else(|| {
        interpolator
            .with_mut(|interpolator| {
                interpolator.keyframe_interpolator_transform_value(from, to, factor)
            })
            .flatten()
    })
}
