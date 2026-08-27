pub trait GamepadListener<I, D> {
    fn gamepad_dispatch(
        &mut self,
        invocation: &I,
        out_dispatched_scripted_drawable: Option<&mut Option<*mut D>>,
    ) -> bool;
}
