#[allow(non_snake_case)]
pub unsafe fn fnVisitV<Visitor, T>(vis: &mut Visitor, src: *mut core::ffi::c_void)
where
    Visitor: FnMut(&T),
{
    let val_ptr = src as *const T;
    vis(&*val_ptr);
}
