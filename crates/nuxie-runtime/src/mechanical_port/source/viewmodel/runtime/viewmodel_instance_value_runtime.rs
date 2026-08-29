use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::mechanical_port::source::{
    core::CoreHandle,
    viewmodel::viewmodel_instance_value::{
        ViewModelInstanceValueDelegate, ViewModelInstanceValueDelegateHandle,
    },
};

pub use crate::mechanical_port::source::data_bind::data_values::data_type::DataType;

struct RuntimeValueChangeTracker {
    changed: Rc<Cell<bool>>,
}

impl ViewModelInstanceValueDelegate for RuntimeValueChangeTracker {
    fn value_changed(&mut self) {
        self.changed.set(true);
    }
}

struct RuntimeValueInner {
    value: CoreHandle,
    data_type: DataType,
    changed: Rc<Cell<bool>>,
    delegate: ViewModelInstanceValueDelegateHandle,
}

impl Drop for RuntimeValueInner {
    fn drop(&mut self) {
        let _ = self.value.with_mut(|value| {
            if let Some(value) = value.as_view_model_instance_value_mut() {
                value.remove_delegate(&self.delegate);
            }
        });
    }
}

#[derive(Clone)]
pub struct ViewModelInstanceValueRuntime {
    inner: Rc<RuntimeValueInner>,
}

impl ViewModelInstanceValueRuntime {
    pub fn new(value: CoreHandle, data_type: DataType) -> Option<Self> {
        let changed = Rc::new(Cell::new(false));
        let delegate: ViewModelInstanceValueDelegateHandle =
            Rc::new(RefCell::new(RuntimeValueChangeTracker {
                changed: Rc::clone(&changed),
            }));
        value
            .with_mut(|value| {
                value
                    .as_view_model_instance_value_mut()
                    .map(|value| value.add_delegate(&delegate))
            })
            .flatten()?;
        Some(Self {
            inner: Rc::new(RuntimeValueInner {
                value,
                data_type,
                changed,
                delegate,
            }),
        })
    }

    pub fn handle(&self) -> CoreHandle {
        self.inner.value.clone()
    }

    pub fn data_type(&self) -> DataType {
        self.inner.data_type
    }

    pub fn clear_changes(&self) {
        self.inner.changed.set(false);
    }

    pub fn has_changed(&self) -> bool {
        self.inner.changed.get()
    }

    pub fn flush_changes(&self) -> bool {
        self.inner.changed.replace(false)
    }

    pub fn name(&self) -> String {
        self.inner
            .value
            .with(|value| {
                value
                    .as_view_model_instance_value()
                    .map(|value| value.name())
            })
            .flatten()
            .unwrap_or_default()
    }

    pub fn relink_data_bind(&self) {}
}
