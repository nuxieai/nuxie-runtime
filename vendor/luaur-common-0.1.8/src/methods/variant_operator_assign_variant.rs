use crate::records::variant::Variant3;

pub fn operator_assign<T0, T1, T2>(_self: &mut Variant3<T0, T1, T2>, _other: &Variant3<T0, T1, T2>)
where
    Variant3<T0, T1, T2>: Clone,
{
    *_self = _other.clone();
}
