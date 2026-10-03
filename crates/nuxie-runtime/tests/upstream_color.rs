//! Regression port from `tests/unit_tests/runtime/color_test.cpp` at upstream
//! 93d8161dacd3031f49b676d3192c038f3870e33d.

use nuxie_runtime::source::data_bind::data_values::data_value_color::DataValueColor;

#[test]
fn data_value_color_channels_keep_their_low_byte() {
    let mut color = DataValueColor::new(0x80102030u32 as i32);
    color.set_red(-1);
    assert_eq!(color.value() as u32, 0x80FF2030);
    color.set_blue(300);
    assert_eq!(color.value() as u32, 0x80FF202C);
    color.set_alpha(0x1FF);
    assert_eq!(color.value() as u32, 0xFFFF202C);
}
