use crate::mechanical_port::source::{
    data_bind::data_bind_path::DataBindPath,
    generated::data_bind::data_bind_path_base::DataBindPathBase,
    importers::{data_bind_path_importer::DataBindPathImporter, import_stack::ImportStack},
};

#[derive(Default)]
pub struct DataBindPathReferencer {
    data_bind_path: Option<Box<DataBindPath>>,
}

impl DataBindPathReferencer {
    pub fn data_bind_path(&self) -> Option<&DataBindPath> {
        self.data_bind_path.as_deref()
    }

    pub fn copy_data_bind_path(&mut self, data_bind_path: Option<&DataBindPath>) {
        if let Some(data_bind_path) = data_bind_path {
            let mut cloned = data_bind_path.clone();
            cloned.set_file(data_bind_path.file());
            self.data_bind_path = Some(Box::new(cloned));
        }
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
        self.data_bind_path = Some(unsafe { Box::from_raw(data_bind_path.as_ptr()) });
    }

    pub fn decode_data_bind_path(&mut self, value: &[u8]) {
        let mut data_bind_path = Box::<DataBindPath>::default();
        data_bind_path.decode_path(value);
        data_bind_path.set_resolved(true);
        self.data_bind_path = Some(data_bind_path);
    }
}
