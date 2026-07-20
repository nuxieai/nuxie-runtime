use crate::records::variant::Variant1;

impl<T0> Variant1<T0> {
    pub const fn cmax(l: &[usize]) -> usize {
        let mut res = 0;
        let mut i = 0;
        while i < l.len() {
            let val = l[i];
            if res < val {
                res = val;
            }
            i += 1;
        }
        res
    }
}

/// Helper for `VariantN` family to access the static `cmax` utility.
/// In C++ this is a static member of the variadic `Variant` template.
/// In Rust, we provide it on the base `Variant1` and can alias it if needed.
#[allow(non_snake_case)]
pub const fn variant_cmax(l: &[usize]) -> usize {
    Variant1::<()>::cmax(l)
}
