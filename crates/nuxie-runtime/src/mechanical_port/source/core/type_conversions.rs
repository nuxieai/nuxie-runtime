use std::mem::size_of;

pub fn fits_in<T>(value: i64) -> bool
where
    T: TryFrom<i64>,
{
    T::try_from(value).is_ok()
}

pub fn cast_to<T>(value: i64) -> T
where
    T: TryFrom<i64>,
{
    assert!(size_of::<T>() <= 4);
    match T::try_from(value) {
        Ok(value) => value,
        Err(_) => panic!("integer value does not fit destination type"),
    }
}

pub trait UnsignedIntegral: Copy {
    fn overflowing_mul(self, other: Self) -> (Self, bool);
}

macro_rules! impl_unsigned_integral {
    ($($ty:ty),* $(,)?) => {
        $(
            impl UnsignedIntegral for $ty {
                fn overflowing_mul(self, other: Self) -> (Self, bool) {
                    <$ty>::overflowing_mul(self, other)
                }
            }
        )*
    };
}

impl_unsigned_integral!(u8, u16, u32, u64, u128, usize);

pub fn checked_mul<T>(a: T, b: T, out: &mut T) -> bool
where
    T: UnsignedIntegral,
{
    let (product, overflow) = a.overflowing_mul(b);
    *out = product;
    !overflow
}

pub fn mul_overflows<T>(a: T, b: T) -> bool
where
    T: UnsignedIntegral,
{
    let mut product = a;
    !checked_mul(a, b, &mut product)
}
