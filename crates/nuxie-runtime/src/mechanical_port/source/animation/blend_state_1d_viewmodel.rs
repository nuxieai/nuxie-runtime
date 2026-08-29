use crate::mechanical_port::source::{
    animation::{
        animation_reset::AnimationResetTarget,
        animation_reset_factory::ResetArtboard,
        blend_animation_1d::BlendAnimation1D,
        blend_state_1d::BlendState1D,
        blend_state_1d_instance::{
            BlendState1DDefinition, BlendState1DInstance, BlendState1DValueSource,
        },
        blend_state_instance::BlendStateDefinition,
        state_machine::StateMachine,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
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
    bindable_property: Option<CoreHandle>,
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
            .latest::<StateMachineImporter>(crate::mechanical_port::source::generated::animation::state_machine_base::StateMachineBase::TYPE_KEY)
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

    pub fn bindable_property(&self) -> Option<CoreHandle> {
        self.bindable_property.clone()
    }

    pub fn make_instance<R>(
        &self,
        instance: &mut R,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> BlendState1DInstance<Self, BlendAnimation1D>
    where
        R: ResetArtboard + AnimationResetTarget,
    {
        let state = self
            .base
            .base
            .base
            .base
            .base
            .base
            .handle()
            .expect("an imported BlendState1DViewModel has arena identity before instancing");
        BlendState1DInstance::new(state, instance, artboard)
    }
}

impl BlendStateDefinition<BlendAnimation1D> for BlendState1DViewModel {
    fn animations(&self) -> Vec<CoreHandle> {
        <BlendState1D as BlendStateDefinition<BlendAnimation1D>>::animations(&self.base.base)
    }

    fn flags(&self) -> u8 {
        <BlendState1D as BlendStateDefinition<BlendAnimation1D>>::flags(&self.base.base)
    }
}

impl BlendState1DDefinition<BlendAnimation1D> for BlendState1DViewModel {
    fn value_source(&self) -> BlendState1DValueSource {
        self.bindable_property
            .clone()
            .map(BlendState1DValueSource::ViewModel)
            .unwrap_or(BlendState1DValueSource::Default)
    }
}

impl std::ops::Deref for BlendState1DViewModel {
    type Target = BlendState1DViewModelBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for BlendState1DViewModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
