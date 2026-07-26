use nuxie_binary::RuntimeObject;

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionInputCondition {
    input_index: usize,
}

impl RuntimeTransitionInputCondition {
    pub(super) fn from_object(object: &RuntimeObject) -> Option<Self> {
        Some(Self {
            input_index: usize::try_from(object.uint_property("inputId")?).ok()?,
        })
    }

    #[cfg(test)]
    pub(super) fn new(input_index: usize) -> Self {
        Self { input_index }
    }

    pub(super) fn input_index(self) -> usize {
        self.input_index
    }
}
