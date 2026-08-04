use core::marker::PhantomData;

pub struct ItemInterfaceMap2<K, V>(pub(crate) PhantomData<(K, V)>);
