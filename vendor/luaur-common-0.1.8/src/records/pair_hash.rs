use crate::functions::hash_combine::hash_combine;

#[allow(non_snake_case)]
#[derive(Debug, Clone, Default)]
pub struct PairHash<T1, T2, H1, H2> {
    pub(crate) h1: H1,
    pub(crate) h2: H2,
    pub(crate) _marker: core::marker::PhantomData<(T1, T2)>,
}

impl<T1, T2, H1, H2> PairHash<T1, T2, H1, H2>
where
    H1: Fn(&T1) -> usize,
    H2: Fn(&T2) -> usize,
{
    #[allow(non_snake_case)]
    #[inline]
    pub fn call(&self, p: &(T1, T2)) -> usize {
        let mut seed = 0;
        hash_combine(&mut seed, (self.h1)(&p.0));
        hash_combine(&mut seed, (self.h2)(&p.1));
        seed
    }
}
