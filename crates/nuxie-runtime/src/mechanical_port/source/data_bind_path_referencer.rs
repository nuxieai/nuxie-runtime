use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::data_bind_path::DataBindPath,
    generated::data_bind::data_bind_path_base::DataBindPathBase,
    importers::{data_bind_path_importer::DataBindPathImporter, import_stack::ImportStack},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
enum DataBindPathOccurrence {
    Authored(CoreHandle),
    Runtime(Rc<RefCell<DataBindPath>>),
}

#[derive(Default)]
pub struct DataBindPathReferencer {
    data_bind_path: Option<DataBindPathOccurrence>,
}

impl DataBindPathReferencer {
    pub fn with_data_bind_path<R>(&self, use_path: impl FnOnce(&DataBindPath) -> R) -> Option<R> {
        match self.data_bind_path.as_ref()? {
            DataBindPathOccurrence::Authored(path) => path.with_downcast(use_path),
            DataBindPathOccurrence::Runtime(path) => Some(use_path(&path.borrow())),
        }
    }

    pub fn with_data_bind_path_mut<R>(
        &self,
        use_path: impl FnOnce(&mut DataBindPath) -> R,
    ) -> Option<R> {
        match self.data_bind_path.as_ref()? {
            DataBindPathOccurrence::Authored(path) => path.with_downcast_mut(use_path),
            DataBindPathOccurrence::Runtime(path) => Some(use_path(&mut path.borrow_mut())),
        }
    }

    pub fn copy_data_bind_path(&mut self, source: &DataBindPathReferencer) {
        let Some(cloned) = source.with_data_bind_path(|data_bind_path| {
            let mut cloned = data_bind_path.clone();
            cloned.set_file(data_bind_path.file());
            DataBindPathOccurrence::Runtime(Rc::new(RefCell::new(cloned)))
        }) else {
            return;
        };
        self.data_bind_path = Some(cloned);
    }

    pub fn import_data_bind_path(&mut self, import_stack: &mut ImportStack) {
        let Some(importer) =
            import_stack.latest::<DataBindPathImporter>(DataBindPathBase::TYPE_KEY)
        else {
            return;
        };
        let Some(data_bind_path) = importer.claim() else {
            return;
        };
        assert!(self.data_bind_path.is_none());
        self.data_bind_path = Some(DataBindPathOccurrence::Authored(data_bind_path));
    }

    pub fn decode_data_bind_path(&mut self, value: &[u8]) {
        let mut data_bind_path = DataBindPath::default();
        data_bind_path.decode_path(value);
        data_bind_path.set_resolved(true);
        self.data_bind_path = Some(DataBindPathOccurrence::Runtime(Rc::new(RefCell::new(
            data_bind_path,
        ))));
    }
}

impl Drop for DataBindPathReferencer {
    fn drop(&mut self) {
        if let Some(DataBindPathOccurrence::Authored(path)) = self.data_bind_path.take() {
            path.remove_occurrence();
        }
    }
}
