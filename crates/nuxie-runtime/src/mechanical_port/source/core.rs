use crate::mechanical_port::source::{
    component_dirt::ComponentDirt, core::binary_reader::BinaryReader, core_context::CoreContext,
    data_bind::data_bind::DataBind, importers::import_stack::ImportStack, status_code::StatusCode,
};

pub mod binary_data_reader;
pub mod binary_reader;
pub mod binary_stream;
pub mod binary_writer;
pub mod field_types;
pub mod type_conversions;
pub mod vector_binary_stream;
pub mod vector_binary_writer;

pub struct Core {
    first_observer: Option<*mut DataBind>,
}

impl Default for Core {
    fn default() -> Self {
        Self {
            first_observer: None,
        }
    }
}

impl Clone for Core {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Core {
    pub const EMPTY_ID: u32 = u32::MAX;
    pub const INVALID_PROPERTY_KEY: i32 = 0;

    pub fn core_type(&self) -> u16 {
        panic!("abstract Core::core_type");
    }

    pub fn is_type_of(&self, _type_key: u16) -> bool {
        panic!("abstract Core::is_type_of");
    }

    pub fn deserialize(&mut self, _property_key: u16, _reader: &mut BinaryReader<'_>) -> bool {
        panic!("abstract Core::deserialize");
    }

    pub fn clone_core(&self) -> Option<Box<Core>> {
        None
    }

    pub fn validate(&mut self, _context: &mut dyn CoreContext) -> bool {
        true
    }

    pub fn on_added_dirty(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn import(&mut self, _import_stack: &mut ImportStack) -> StatusCode {
        StatusCode::Ok
    }

    pub fn notify_property_changed(&mut self, property_key: u16) {
        let mut observer = self.first_observer;
        while let Some(observer_ptr) = observer {
            let observer_ref = unsafe { &mut *observer_ptr };
            observer = observer_ref.next_observer();
            if observer_ref.property_key() == u32::from(property_key) {
                observer_ref.add_dirt(u32::from(ComponentDirt::BINDINGS_TARGET.0), false);
            }
        }
    }

    pub fn add_property_observer(&mut self, observer: *mut DataBind) {
        let mut current = self.first_observer;
        while let Some(current_ptr) = current {
            assert_ne!(current_ptr, observer, "DataBind already subscribed");
            current = unsafe { &*current_ptr }.next_observer();
        }
        unsafe { &mut *observer }.set_next_observer(self.first_observer);
        self.first_observer = Some(observer);
    }

    pub fn remove_property_observer(&mut self, observer: *mut DataBind) {
        let mut link = &mut self.first_observer as *mut Option<*mut DataBind>;
        unsafe {
            while let Some(current) = *link {
                if current == observer {
                    *link = (&*observer).next_observer();
                    (&mut *observer).set_next_observer(None);
                    return;
                }
                link = (&mut *current).next_observer_ref();
            }
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        let mut observer = self.first_observer;
        while let Some(observer_ptr) = observer {
            let observer_ref = unsafe { &mut *observer_ptr };
            observer = observer_ref.next_observer();
            observer_ref.on_target_destroyed();
        }
        self.first_observer = None;
    }
}
