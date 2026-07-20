use crate::records::variant::{
    Variant1, Variant2, Variant3, Variant4, Variant5, Variant6, Variant7,
};

impl<T0: 'static> Variant1<T0> {
    pub fn get_type_id<T: 'static>() -> i32 {
        if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T0>() {
            0
        } else {
            -1
        }
    }
}

impl<T0: 'static, T1: 'static> Variant2<T0, T1> {
    pub fn get_type_id<T: 'static>() -> i32 {
        if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T0>() {
            0
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T1>() {
            1
        } else {
            -1
        }
    }
}

impl<T0: 'static, T1: 'static, T2: 'static> Variant3<T0, T1, T2> {
    pub fn get_type_id<T: 'static>() -> i32 {
        if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T0>() {
            0
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T1>() {
            1
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T2>() {
            2
        } else {
            -1
        }
    }
}

impl<T0: 'static, T1: 'static, T2: 'static, T3: 'static> Variant4<T0, T1, T2, T3> {
    pub fn get_type_id<T: 'static>() -> i32 {
        if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T0>() {
            0
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T1>() {
            1
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T2>() {
            2
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T3>() {
            3
        } else {
            -1
        }
    }
}

impl<T0: 'static, T1: 'static, T2: 'static, T3: 'static, T4: 'static> Variant5<T0, T1, T2, T3, T4> {
    pub fn get_type_id<T: 'static>() -> i32 {
        if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T0>() {
            0
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T1>() {
            1
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T2>() {
            2
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T3>() {
            3
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T4>() {
            4
        } else {
            -1
        }
    }
}

impl<T0: 'static, T1: 'static, T2: 'static, T3: 'static, T4: 'static, T5: 'static>
    Variant6<T0, T1, T2, T3, T4, T5>
{
    pub fn get_type_id<T: 'static>() -> i32 {
        if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T0>() {
            0
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T1>() {
            1
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T2>() {
            2
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T3>() {
            3
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T4>() {
            4
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T5>() {
            5
        } else {
            -1
        }
    }
}

impl<T0: 'static, T1: 'static, T2: 'static, T3: 'static, T4: 'static, T5: 'static, T6: 'static>
    Variant7<T0, T1, T2, T3, T4, T5, T6>
{
    pub fn get_type_id<T: 'static>() -> i32 {
        if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T0>() {
            0
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T1>() {
            1
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T2>() {
            2
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T3>() {
            3
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T4>() {
            4
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T5>() {
            5
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<T6>() {
            6
        } else {
            -1
        }
    }
}
