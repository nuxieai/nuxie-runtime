use crate::records::pair_hash::PairHash;

// Method `call` already defined on PairHash in the record file.
// This operator() method is fully provided by the record's inherent impl.
pub fn _pair_hash_operator_call_body<T1, T2, H1, H2>(h1: &H1, h2: &H2, p: &(T1, T2)) -> usize
where
    H1: Fn(&T1) -> usize,
    H2: Fn(&T2) -> usize,
{
    let mut seed: usize = 0;
    crate::functions::hash_combine::hash_combine(&mut seed, h1(&p.0));
    crate::functions::hash_combine::hash_combine(&mut seed, h2(&p.1));
    seed
}
