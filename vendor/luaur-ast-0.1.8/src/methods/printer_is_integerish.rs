use crate::records::printer::Printer;

impl<'a> Printer<'a> {
    pub fn printer_is_integerish(d: f64) -> bool {
        if d <= (i32::MAX as f64) && d >= (i32::MIN as f64) {
            (d as i32 as f64) == d && !(d == 0.0 && d.is_sign_negative())
        } else {
            false
        }
    }
}
