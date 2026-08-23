//! Complete mechanical declaration translation of
//! `renderer/src/webgpu/wagyu-port/include/webgpu/webgpu_enum_class_bitmasks.h`.

#![allow(non_snake_case)]

use std::marker::PhantomData;
use std::ops::{BitAnd, BitOr, BitXor, Not};

/// Rust counterpart of a source `IsWGPUBitmask<T>` specialization.
///
/// Types without this implementation retain the source template's disabled
/// default and cannot enter any of the bitmask operations.
pub(crate) trait IsWGPUBitmask: Copy {
    type Integral: Copy
        + PartialEq
        + From<u8>
        + BitAnd<Output = Self::Integral>
        + BitOr<Output = Self::Integral>
        + BitXor<Output = Self::Integral>
        + Not<Output = Self::Integral>;

    fn fromIntegral(value: Self::Integral) -> Self;
    fn intoIntegral(self) -> Self::Integral;
    fn wrappingSubOne(value: Self::Integral) -> Self::Integral;
}

/// Enabled source `LowerBitmask` specializations lower either a bitmask enum or
/// its intermediate boolean-convertible result to the same integral type.
pub(crate) trait LowerBitmask<T: IsWGPUBitmask>: Copy {
    fn Lower(self) -> T::Integral;
}

impl<T: IsWGPUBitmask> LowerBitmask<T> for T {
    fn Lower(self) -> T::Integral {
        self.intoIntegral()
    }
}

/// Source operators return this integral-holding intermediate, which is
/// explicitly usable as either a boolean or the original bitmask type.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BoolConvertible<T: IsWGPUBitmask> {
    value: T::Integral,
    marker: PhantomData<T>,
}

impl<T: IsWGPUBitmask> Copy for BoolConvertible<T> {}

impl<T: IsWGPUBitmask> Clone for BoolConvertible<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: IsWGPUBitmask> BoolConvertible<T> {
    pub(crate) const fn fromIntegral(value: T::Integral) -> Self {
        Self {
            value,
            marker: PhantomData,
        }
    }

    pub(crate) fn asBool(self) -> bool {
        self.value != T::Integral::from(0)
    }

    pub(crate) fn intoBitmask(self) -> T {
        T::fromIntegral(self.value)
    }

    pub(crate) fn integral(self) -> T::Integral {
        self.value
    }
}

impl<T: IsWGPUBitmask> LowerBitmask<T> for BoolConvertible<T> {
    fn Lower(self) -> T::Integral {
        self.value
    }
}

pub(crate) fn operator_or<T, T1, T2>(left: T1, right: T2) -> BoolConvertible<T>
where
    T: IsWGPUBitmask,
    T1: LowerBitmask<T>,
    T2: LowerBitmask<T>,
{
    BoolConvertible::fromIntegral(left.Lower() | right.Lower())
}

pub(crate) fn operator_and<T, T1, T2>(left: T1, right: T2) -> BoolConvertible<T>
where
    T: IsWGPUBitmask,
    T1: LowerBitmask<T>,
    T2: LowerBitmask<T>,
{
    BoolConvertible::fromIntegral(left.Lower() & right.Lower())
}

pub(crate) fn operator_xor<T, T1, T2>(left: T1, right: T2) -> BoolConvertible<T>
where
    T: IsWGPUBitmask,
    T1: LowerBitmask<T>,
    T2: LowerBitmask<T>,
{
    BoolConvertible::fromIntegral(left.Lower() ^ right.Lower())
}

pub(crate) fn operator_not<T, T1>(value: T1) -> BoolConvertible<T>
where
    T: IsWGPUBitmask,
    T1: LowerBitmask<T>,
{
    BoolConvertible::fromIntegral(!value.Lower())
}

pub(crate) fn HasZeroOrOneBits<T: IsWGPUBitmask>(value: T) -> bool {
    let integral = value.intoIntegral();
    (integral & T::wrappingSubOne(integral)) == T::Integral::from(0)
}

/// Introduces the source operators for one locally declared WebGPU bitmask
/// type. This is the Rust equivalent of `WGPU_IMPORT_BITMASK_OPERATORS` plus
/// the generated `IsWGPUBitmask` specialization.
macro_rules! impl_wgpu_bitmask_operators {
    ($type:ty) => {
        impl std::ops::BitOr for $type {
            type Output = $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>;

            fn bitor(self, right: Self) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_or::<$type, _, _>(self, right)
            }
        }

        impl std::ops::BitAnd for $type {
            type Output = $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>;

            fn bitand(self, right: Self) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_and::<$type, _, _>(self, right)
            }
        }

        impl std::ops::BitXor for $type {
            type Output = $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>;

            fn bitxor(self, right: Self) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_xor::<$type, _, _>(self, right)
            }
        }

        impl std::ops::Not for $type {
            type Output = $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>;

            fn not(self) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_not::<$type, _>(self)
            }
        }

        impl std::ops::BitOr<$crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>> for $type {
            type Output = $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>;

            fn bitor(self, right: Self::Output) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_or::<$type, _, _>(self, right)
            }
        }

        impl std::ops::BitAnd<$crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>> for $type {
            type Output = $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>;

            fn bitand(self, right: Self::Output) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_and::<$type, _, _>(self, right)
            }
        }

        impl std::ops::BitXor<$crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>> for $type {
            type Output = $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>;

            fn bitxor(self, right: Self::Output) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_xor::<$type, _, _>(self, right)
            }
        }

        impl std::ops::BitOr<$type> for $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type> {
            type Output = Self;

            fn bitor(self, right: $type) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_or::<$type, _, _>(self, right)
            }
        }

        impl std::ops::BitAnd<$type> for $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type> {
            type Output = Self;

            fn bitand(self, right: $type) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_and::<$type, _, _>(self, right)
            }
        }

        impl std::ops::BitXor<$type> for $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type> {
            type Output = Self;

            fn bitxor(self, right: $type) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_xor::<$type, _, _>(self, right)
            }
        }

        impl std::ops::BitOr for $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type> {
            type Output = Self;

            fn bitor(self, right: Self) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_or::<$type, _, _>(self, right)
            }
        }

        impl std::ops::BitAnd for $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type> {
            type Output = Self;

            fn bitand(self, right: Self) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_and::<$type, _, _>(self, right)
            }
        }

        impl std::ops::BitXor for $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type> {
            type Output = Self;

            fn bitxor(self, right: Self) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_xor::<$type, _, _>(self, right)
            }
        }

        impl std::ops::Not for $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type> {
            type Output = Self;

            fn not(self) -> Self::Output {
                $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::operator_not::<$type, _>(self)
            }
        }

        impl std::ops::BitOrAssign for $type {
            fn bitor_assign(&mut self, right: Self) {
                *self = (*self | right).intoBitmask();
            }
        }

        impl std::ops::BitAndAssign for $type {
            fn bitand_assign(&mut self, right: Self) {
                *self = (*self & right).intoBitmask();
            }
        }

        impl std::ops::BitXorAssign for $type {
            fn bitxor_assign(&mut self, right: Self) {
                *self = (*self ^ right).intoBitmask();
            }
        }

        impl std::ops::BitOrAssign<$crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>> for $type {
            fn bitor_assign(&mut self, right: $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>) {
                *self = (*self | right).intoBitmask();
            }
        }

        impl std::ops::BitAndAssign<$crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>> for $type {
            fn bitand_assign(&mut self, right: $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>) {
                *self = (*self & right).intoBitmask();
            }
        }

        impl std::ops::BitXorAssign<$crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>> for $type {
            fn bitxor_assign(&mut self, right: $crate::mechanical_port::webgpu::webgpu_enum_class_bitmasks_decl::BoolConvertible<$type>) {
                *self = (*self ^ right).intoBitmask();
            }
        }
    };
}

pub(crate) use impl_wgpu_bitmask_operators;

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Usage(u32);

    impl IsWGPUBitmask for Usage {
        type Integral = u32;

        fn fromIntegral(value: Self::Integral) -> Self {
            Self(value)
        }

        fn intoIntegral(self) -> Self::Integral {
            self.0
        }

        fn wrappingSubOne(value: Self::Integral) -> Self::Integral {
            value.wrapping_sub(1)
        }
    }

    impl_wgpu_bitmask_operators!(Usage);

    #[test]
    fn source_binary_and_unary_operators_return_bool_convertible() {
        let one = Usage(1);
        let two = Usage(2);
        assert_eq!((one | two).integral(), 3);
        assert_eq!((Usage(3) & two).integral(), 2);
        assert_eq!((Usage(3) ^ one).integral(), 2);
        assert_eq!((!Usage(0)).integral(), u32::MAX);
    }

    #[test]
    fn source_bool_and_enum_conversions_remain_explicit() {
        let empty = Usage(0) & Usage(2);
        assert!(!empty.asBool());
        assert_eq!(empty.intoBitmask(), Usage(0));

        let nonempty = Usage(1) | Usage(2);
        assert!(nonempty.asBool());
        assert_eq!(nonempty.intoBitmask(), Usage(3));
    }

    #[test]
    fn source_assignment_operators_update_the_enum_value() {
        let mut value = Usage(1);
        value |= Usage(2);
        assert_eq!(value, Usage(3));
        value &= Usage(2);
        assert_eq!(value, Usage(2));
        value ^= Usage(3);
        assert_eq!(value, Usage(1));

        value |= Usage(2) | Usage(4);
        assert_eq!(value, Usage(7));
        value &= Usage(6) | Usage(4);
        assert_eq!(value, Usage(6));
        value ^= Usage(1) | Usage(2);
        assert_eq!(value, Usage(5));
    }

    #[test]
    fn source_mixed_lower_bitmask_operands_compose() {
        let result = ((Usage(1) | Usage(2)) & Usage(1)) ^ (Usage(4) | Usage(8));
        assert_eq!(result.integral(), 13);
        assert_eq!((!(Usage(1) | Usage(2))).integral(), !3u32);
    }

    #[test]
    fn source_zero_or_one_bit_predicate_includes_zero() {
        assert!(HasZeroOrOneBits(Usage(0)));
        assert!(HasZeroOrOneBits(Usage(1)));
        assert!(HasZeroOrOneBits(Usage(8)));
        assert!(!HasZeroOrOneBits(Usage(3)));
    }

    #[test]
    fn bool_convertible_retains_integral_layout() {
        assert_eq!(size_of::<BoolConvertible<Usage>>(), size_of::<u32>());
        assert_eq!(align_of::<BoolConvertible<Usage>>(), align_of::<u32>());
    }
}
