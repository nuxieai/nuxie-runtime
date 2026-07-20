impl<T0, T1, T2, T3, T4, T5, T6> crate::records::variant::Variant7<T0, T1, T2, T3, T4, T5, T6>
where
    T0: PartialEq,
    T1: PartialEq,
    T2: PartialEq,
    T3: PartialEq,
    T4: PartialEq,
    T5: PartialEq,
    T6: PartialEq,
{
    pub fn variant_operator_ne(&self, other: &Self) -> bool {
        !(*self == *other)
    }
}

impl<T0, T1, T2, T3, T4, T5> crate::records::variant::Variant6<T0, T1, T2, T3, T4, T5>
where
    T0: PartialEq,
    T1: PartialEq,
    T2: PartialEq,
    T3: PartialEq,
    T4: PartialEq,
    T5: PartialEq,
{
    pub fn variant_operator_ne(&self, other: &Self) -> bool {
        !(*self == *other)
    }
}

impl<T0, T1, T2, T3, T4> crate::records::variant::Variant5<T0, T1, T2, T3, T4>
where
    T0: PartialEq,
    T1: PartialEq,
    T2: PartialEq,
    T3: PartialEq,
    T4: PartialEq,
{
    pub fn variant_operator_ne(&self, other: &Self) -> bool {
        !(*self == *other)
    }
}

impl<T0, T1, T2, T3> crate::records::variant::Variant4<T0, T1, T2, T3>
where
    T0: PartialEq,
    T1: PartialEq,
    T2: PartialEq,
    T3: PartialEq,
{
    pub fn variant_operator_ne(&self, other: &Self) -> bool {
        !(*self == *other)
    }
}

impl<T0, T1, T2> crate::records::variant::Variant3<T0, T1, T2>
where
    T0: PartialEq,
    T1: PartialEq,
    T2: PartialEq,
{
    pub fn variant_operator_ne(&self, other: &Self) -> bool {
        !(*self == *other)
    }
}

impl<T0, T1> crate::records::variant::Variant2<T0, T1>
where
    T0: PartialEq,
    T1: PartialEq,
{
    pub fn variant_operator_ne(&self, other: &Self) -> bool {
        !(*self == *other)
    }
}

impl<T0> crate::records::variant::Variant1<T0>
where
    T0: PartialEq,
{
    pub fn variant_operator_ne(&self, other: &Self) -> bool {
        !(*self == *other)
    }
}
