use crate::records::variant::{
    Variant1, Variant2, Variant3, Variant4, Variant5, Variant6, Variant7,
};

pub fn get_if<T, V>(var: &V) -> Option<&T>
where
    V: VariantAccess<T>,
{
    var.get_if_t()
}

trait VariantAccess<T> {
    fn get_if_t(&self) -> Option<&T>;
}

impl<T0> VariantAccess<T0> for Variant1<T0> {
    fn get_if_t(&self) -> Option<&T0> {
        self.get_if_0()
    }
}

impl<T0, T1> VariantAccess<T0> for Variant2<T0, T1> {
    fn get_if_t(&self) -> Option<&T0> {
        self.get_if_0()
    }
}

impl<T0, T1, T2> VariantAccess<T0> for Variant3<T0, T1, T2> {
    fn get_if_t(&self) -> Option<&T0> {
        self.get_if_0()
    }
}

impl<T0, T1, T2, T3> VariantAccess<T0> for Variant4<T0, T1, T2, T3> {
    fn get_if_t(&self) -> Option<&T0> {
        self.get_if_0()
    }
}

impl<T0, T1, T2, T3, T4> VariantAccess<T0> for Variant5<T0, T1, T2, T3, T4> {
    fn get_if_t(&self) -> Option<&T0> {
        self.get_if_0()
    }
}

impl<T0, T1, T2, T3, T4, T5> VariantAccess<T0> for Variant6<T0, T1, T2, T3, T4, T5> {
    fn get_if_t(&self) -> Option<&T0> {
        self.get_if_0()
    }
}

impl<T0, T1, T2, T3, T4, T5, T6> VariantAccess<T0> for Variant7<T0, T1, T2, T3, T4, T5, T6> {
    fn get_if_t(&self) -> Option<&T0> {
        self.get_if_0()
    }
}
