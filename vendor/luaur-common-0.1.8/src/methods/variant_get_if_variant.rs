use crate::records::variant::{
    Variant1, Variant2, Variant3, Variant4, Variant5, Variant6, Variant7,
};

pub fn variant_get_if() {
    // This file should not contain shared logic; the real translation is provided
    // via the per-arity `get_if` methods generated below.
    // The C++ method:
    //   template<typename T>
    //   const T* get_if() const
    // maps to returning `Option<&T>` / `Option<&mut T>` in the Rust port.
    // However, per the schedule, this specific one-shot item is the `const T*` version,
    // so we return `Option<&T>`.
}

/// Returns `Some(&T)` if the active alternative has type `T`, else `None`.
impl<T0: 'static> Variant1<T0> {
    pub fn get_if<T: 'static>(&self) -> Option<&T> {
        let tid = Self::get_type_id::<T>();
        if tid < 0 {
            return None;
        }

        if self.index() as i32 == tid {
            // SAFETY:
            // - In this port, VariantN stores the active alternative as a real value inside the enum.
            // - `get_type_id::<T>()` guarantees the active variant matches `T`.
            // - We can safely take a reference to the correct payload.
            if tid == 0 {
                self.get_if_0().and_then(|v| {
                    let r: &T = unsafe { &*(v as *const T0 as *const T) };
                    Some(r)
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Returns `Some(&T)` if the active alternative has type `T`, else `None`.
impl<T0: 'static, T1: 'static> Variant2<T0, T1> {
    pub fn get_if<T: 'static>(&self) -> Option<&T> {
        let tid = Self::get_type_id::<T>();
        if tid < 0 {
            return None;
        }

        if self.index() as i32 == tid {
            match tid {
                0 => self.get_if_0().and_then(|v0| {
                    let r: &T = unsafe { &*(v0 as *const T0 as *const T) };
                    Some(r)
                }),
                1 => self.get_if_1().and_then(|v1| {
                    let r: &T = unsafe { &*(v1 as *const T1 as *const T) };
                    Some(r)
                }),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Returns `Some(&T)` if the active alternative has type `T`, else `None`.
impl<T0: 'static, T1: 'static, T2: 'static> Variant3<T0, T1, T2> {
    pub fn get_if<T: 'static>(&self) -> Option<&T> {
        let tid = Self::get_type_id::<T>();
        if tid < 0 {
            return None;
        }

        if self.index() as i32 == tid {
            match tid {
                0 => self.get_if_0().and_then(|v0| {
                    let r: &T = unsafe { &*(v0 as *const T0 as *const T) };
                    Some(r)
                }),
                1 => self.get_if_1().and_then(|v1| {
                    let r: &T = unsafe { &*(v1 as *const T1 as *const T) };
                    Some(r)
                }),
                2 => self.get_if_2().and_then(|v2| {
                    let r: &T = unsafe { &*(v2 as *const T2 as *const T) };
                    Some(r)
                }),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Returns `Some(&T)` if the active alternative has type `T`, else `None`.
impl<T0: 'static, T1: 'static, T2: 'static, T3: 'static> Variant4<T0, T1, T2, T3> {
    pub fn get_if<T: 'static>(&self) -> Option<&T> {
        let tid = Self::get_type_id::<T>();
        if tid < 0 {
            return None;
        }

        if self.index() as i32 == tid {
            match tid {
                0 => self.get_if_0().and_then(|v0| {
                    let r: &T = unsafe { &*(v0 as *const T0 as *const T) };
                    Some(r)
                }),
                1 => self.get_if_1().and_then(|v1| {
                    let r: &T = unsafe { &*(v1 as *const T1 as *const T) };
                    Some(r)
                }),
                2 => self.get_if_2().and_then(|v2| {
                    let r: &T = unsafe { &*(v2 as *const T2 as *const T) };
                    Some(r)
                }),
                3 => self.get_if_3().and_then(|v3| {
                    let r: &T = unsafe { &*(v3 as *const T3 as *const T) };
                    Some(r)
                }),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Returns `Some(&T)` if the active alternative has type `T`, else `None`.
impl<T0: 'static, T1: 'static, T2: 'static, T3: 'static, T4: 'static> Variant5<T0, T1, T2, T3, T4> {
    pub fn get_if<T: 'static>(&self) -> Option<&T> {
        let tid = Self::get_type_id::<T>();
        if tid < 0 {
            return None;
        }

        if self.index() as i32 == tid {
            match tid {
                0 => self.get_if_0().and_then(|v0| {
                    let r: &T = unsafe { &*(v0 as *const T0 as *const T) };
                    Some(r)
                }),
                1 => self.get_if_1().and_then(|v1| {
                    let r: &T = unsafe { &*(v1 as *const T1 as *const T) };
                    Some(r)
                }),
                2 => self.get_if_2().and_then(|v2| {
                    let r: &T = unsafe { &*(v2 as *const T2 as *const T) };
                    Some(r)
                }),
                3 => self.get_if_3().and_then(|v3| {
                    let r: &T = unsafe { &*(v3 as *const T3 as *const T) };
                    Some(r)
                }),
                4 => self.get_if_4().and_then(|v4| {
                    let r: &T = unsafe { &*(v4 as *const T4 as *const T) };
                    Some(r)
                }),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Returns `Some(&T)` if the active alternative has type `T`, else `None`.
impl<T0: 'static, T1: 'static, T2: 'static, T3: 'static, T4: 'static, T5: 'static>
    Variant6<T0, T1, T2, T3, T4, T5>
{
    pub fn get_if<T: 'static>(&self) -> Option<&T> {
        let tid = Self::get_type_id::<T>();
        if tid < 0 {
            return None;
        }

        if self.index() as i32 == tid {
            match tid {
                0 => self.get_if_0().and_then(|v0| {
                    let r: &T = unsafe { &*(v0 as *const T0 as *const T) };
                    Some(r)
                }),
                1 => self.get_if_1().and_then(|v1| {
                    let r: &T = unsafe { &*(v1 as *const T1 as *const T) };
                    Some(r)
                }),
                2 => self.get_if_2().and_then(|v2| {
                    let r: &T = unsafe { &*(v2 as *const T2 as *const T) };
                    Some(r)
                }),
                3 => self.get_if_3().and_then(|v3| {
                    let r: &T = unsafe { &*(v3 as *const T3 as *const T) };
                    Some(r)
                }),
                4 => self.get_if_4().and_then(|v4| {
                    let r: &T = unsafe { &*(v4 as *const T4 as *const T) };
                    Some(r)
                }),
                5 => self.get_if_5().and_then(|v5| {
                    let r: &T = unsafe { &*(v5 as *const T5 as *const T) };
                    Some(r)
                }),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Returns `Some(&T)` if the active alternative has type `T`, else `None`.
impl<T0: 'static, T1: 'static, T2: 'static, T3: 'static, T4: 'static, T5: 'static, T6: 'static>
    Variant7<T0, T1, T2, T3, T4, T5, T6>
{
    pub fn get_if<T: 'static>(&self) -> Option<&T> {
        let tid = Self::get_type_id::<T>();
        if tid < 0 {
            return None;
        }

        if self.index() as i32 == tid {
            match tid {
                0 => self.get_if_0().and_then(|v0| {
                    let r: &T = unsafe { &*(v0 as *const T0 as *const T) };
                    Some(r)
                }),
                1 => self.get_if_1().and_then(|v1| {
                    let r: &T = unsafe { &*(v1 as *const T1 as *const T) };
                    Some(r)
                }),
                2 => self.get_if_2().and_then(|v2| {
                    let r: &T = unsafe { &*(v2 as *const T2 as *const T) };
                    Some(r)
                }),
                3 => self.get_if_3().and_then(|v3| {
                    let r: &T = unsafe { &*(v3 as *const T3 as *const T) };
                    Some(r)
                }),
                4 => self.get_if_4().and_then(|v4| {
                    let r: &T = unsafe { &*(v4 as *const T4 as *const T) };
                    Some(r)
                }),
                5 => self.get_if_5().and_then(|v5| {
                    let r: &T = unsafe { &*(v5 as *const T5 as *const T) };
                    Some(r)
                }),
                6 => self.get_if_6().and_then(|v6| {
                    let r: &T = unsafe { &*(v6 as *const T6 as *const T) };
                    Some(r)
                }),
                _ => None,
            }
        } else {
            None
        }
    }
}
