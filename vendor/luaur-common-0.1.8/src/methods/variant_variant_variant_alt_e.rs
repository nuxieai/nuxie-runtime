use crate::records::variant::{
    Variant1, Variant2, Variant3, Variant4, Variant5, Variant6, Variant7,
};

impl<T0> Variant1<T0> {
    pub fn variant_destructor(&mut self) {}
}

impl<T0, T1> Variant2<T0, T1> {
    pub fn variant_destructor(&mut self) {}
}

impl<T0, T1, T2> Variant3<T0, T1, T2> {
    pub fn variant_destructor(&mut self) {}
}

impl<T0, T1, T2, T3> Variant4<T0, T1, T2, T3> {
    pub fn variant_destructor(&mut self) {}
}

impl<T0, T1, T2, T3, T4> Variant5<T0, T1, T2, T3, T4> {
    pub fn variant_destructor(&mut self) {}
}

impl<T0, T1, T2, T3, T4, T5> Variant6<T0, T1, T2, T3, T4, T5> {
    pub fn variant_destructor(&mut self) {}
}

impl<T0, T1, T2, T3, T4, T5, T6> Variant7<T0, T1, T2, T3, T4, T5, T6> {
    pub fn variant_destructor(&mut self) {}
}
