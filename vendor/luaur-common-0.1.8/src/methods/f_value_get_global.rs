use crate::records::f_value::FValue;

impl<T: Copy> FValue<T> {
    pub fn get_global(&self) -> T {
        unsafe { *self.value.get() }
    }
}
