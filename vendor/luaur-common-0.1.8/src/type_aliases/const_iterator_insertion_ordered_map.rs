#[allow(non_camel_case_types)]
pub type const_iterator<'a, T> = core::slice::Iter<'a, T>;

#[allow(non_camel_case_types)]
pub type ConstIterator<'a, T> = const_iterator<'a, T>;
