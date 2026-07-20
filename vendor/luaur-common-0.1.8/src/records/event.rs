use crate::enums::event_type::EventType;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Event {
    pub(crate) r#type: EventType,
    pub(crate) token: u16,
    pub(crate) data: EventData,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union EventData {
    pub(crate) microsec: u32,
    pub(crate) dataPos: u32,
}

impl std::fmt::Debug for EventData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventData")
            .field("microsec/dataPos", unsafe { &self.microsec })
            .finish()
    }
}
