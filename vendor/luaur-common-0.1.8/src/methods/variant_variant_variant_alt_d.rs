use crate::records::variant::{
    Variant1, Variant2, Variant3, Variant4, Variant5, Variant6, Variant7,
};

impl<T0: Clone> Variant1<T0> {
    pub fn variant_variant_mut(&mut self) -> Self {
        self.clone()
    }
}

impl<T0: Clone, T1: Clone> Variant2<T0, T1> {
    pub fn variant_variant_mut(&mut self) -> Self {
        self.clone()
    }
}

impl<T0: Clone, T1: Clone, T2: Clone> Variant3<T0, T1, T2> {
    pub fn variant_variant_mut(&mut self) -> Self {
        self.clone()
    }
}

impl<T0: Clone, T1: Clone, T2: Clone, T3: Clone> Variant4<T0, T1, T2, T3> {
    pub fn variant_variant_mut(&mut self) -> Self {
        self.clone()
    }
}

impl<T0: Clone, T1: Clone, T2: Clone, T3: Clone, T4: Clone> Variant5<T0, T1, T2, T3, T4> {
    pub fn variant_variant_mut(&mut self) -> Self {
        self.clone()
    }
}

impl<T0: Clone, T1: Clone, T2: Clone, T3: Clone, T4: Clone, T5: Clone>
    Variant6<T0, T1, T2, T3, T4, T5>
{
    pub fn variant_variant_mut(&mut self) -> Self {
        self.clone()
    }
}

impl<T0: Clone, T1: Clone, T2: Clone, T3: Clone, T4: Clone, T5: Clone, T6: Clone>
    Variant7<T0, T1, T2, T3, T4, T5, T6>
{
    pub fn variant_variant_mut(&mut self) -> Self {
        self.clone()
    }
}
