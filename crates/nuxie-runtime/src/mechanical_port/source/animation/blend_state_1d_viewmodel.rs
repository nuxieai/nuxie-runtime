use std::ptr::NonNull;

use crate::mechanical_port::source::{
    animation::{
        animation_reset::AnimationResetTarget,
        animation_reset_factory::{ResetArtboard, ResetLinearAnimation},
        blend_animation_1d::BlendAnimation1D,
        blend_state_1d::BlendState1D,
        blend_state_1d_instance::{
            BlendState1DDefinition, BlendState1DInstance, BlendState1DValueSource,
        },
        blend_state_instance::{BlendAnimationDefinition, BlendStateDefinition},
        state_machine::StateMachine,
    },
    data_bind::bindable_property::BindableProperty,
    generated::{
        animation::blend_state_1d_viewmodel_base::BlendState1DViewModelBase,
        data_bind::bindable_property_base::BindablePropertyBase,
    },
    importers::{
        bindable_property_importer::BindablePropertyImporter, import_stack::ImportStack,
        state_machine_importer::StateMachineImporter,
    },
    status_code::StatusCode,
};

pub struct BlendState1DViewModel {
    pub base: BlendState1DViewModelBase,
    bindable_property: Option<NonNull<BindableProperty>>,
}

impl Default for BlendState1DViewModel {
    fn default() -> Self {
        Self {
            base: BlendState1DViewModelBase::default(),
            bindable_property: None,
        }
    }
}

impl BlendState1DViewModel {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        if import_stack
            .latest::<StateMachineImporter>(StateMachine::TYPE_KEY)
            .is_none()
        {
            return StatusCode::MissingObject;
        }
        let Some(bindable_importer) =
            import_stack.latest::<BindablePropertyImporter>(BindablePropertyBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        self.bindable_property = bindable_importer.bindable_property();
        self.base.base.import(import_stack)
    }

    pub fn bindable_property(&self) -> Option<NonNull<BindableProperty>> {
        self.bindable_property
    }

    pub fn make_instance<R>(
        &self,
        instance: &mut R,
    ) -> BlendState1DInstance<'_, Self, BlendAnimation1D>
    where
        R: ResetArtboard + AnimationResetTarget,
        <BlendAnimation1D as BlendAnimationDefinition>::Animation: ResetLinearAnimation,
    {
        BlendState1DInstance::new(self, instance)
    }
}

impl Drop for BlendState1DViewModel {
    fn drop(&mut self) {
        if let Some(property) = self.bindable_property.take() {
            unsafe { drop(Box::from_raw(property.as_ptr())) };
        }
    }
}

impl BlendStateDefinition<BlendAnimation1D> for BlendState1DViewModel {
    fn animations(&self) -> Vec<&BlendAnimation1D> {
        <BlendState1D as BlendStateDefinition<BlendAnimation1D>>::animations(&self.base.base)
    }

    fn flags(&self) -> u8 {
        <BlendState1D as BlendStateDefinition<BlendAnimation1D>>::flags(&self.base.base)
    }
}

impl BlendState1DDefinition<BlendAnimation1D> for BlendState1DViewModel {
    fn value_source(&self) -> BlendState1DValueSource {
        self.bindable_property
            .map(BlendState1DValueSource::ViewModel)
            .unwrap_or(BlendState1DValueSource::Default)
    }
}
