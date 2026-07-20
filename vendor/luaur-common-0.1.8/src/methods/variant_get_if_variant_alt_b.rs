impl<T0: 'static, T1: 'static, T2: 'static, T3: 'static, T4: 'static, T5: 'static, T6: 'static>
    crate::records::variant::Variant7<T0, T1, T2, T3, T4, T5, T6>
{
    pub fn get_if_mut<T: 'static>(&mut self) -> ::core::option::Option<&mut T> {
        let tid = Self::get_type_id::<T>();
        match tid {
            0 => self
                .get_if_0_mut()
                .map(|x| unsafe { &mut *(x as *mut T0 as *mut T) }),
            1 => self
                .get_if_1_mut()
                .map(|x| unsafe { &mut *(x as *mut T1 as *mut T) }),
            2 => self
                .get_if_2_mut()
                .map(|x| unsafe { &mut *(x as *mut T2 as *mut T) }),
            3 => self
                .get_if_3_mut()
                .map(|x| unsafe { &mut *(x as *mut T3 as *mut T) }),
            4 => self
                .get_if_4_mut()
                .map(|x| unsafe { &mut *(x as *mut T4 as *mut T) }),
            5 => self
                .get_if_5_mut()
                .map(|x| unsafe { &mut *(x as *mut T5 as *mut T) }),
            6 => self
                .get_if_6_mut()
                .map(|x| unsafe { &mut *(x as *mut T6 as *mut T) }),
            _ => ::core::option::Option::None,
        }
    }
}
