use crate::mechanical_port::source::dirtyable::Dirtyable;

pub trait ViewModelValueDependent: Dirtyable {
    fn relink_data_bind(&mut self);
}
