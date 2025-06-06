// src/keypad.rs

use esp_idf_hal::gpio::*;
use esp_idf_hal::delay::FreeRtos;

pub fn read_keypad(
    row1: &mut PinDriver<Gpio21, Input>,
    row2: &mut PinDriver<Gpio19, Input>,
    row3: &mut PinDriver<Gpio18, Input>,
    row4: &mut PinDriver<Gpio5, Input>,
    col1: &mut PinDriver<Gpio17, Output>,
    col2: &mut PinDriver<Gpio16, Output>,
    col3: &mut PinDriver<Gpio4, Output>,
    col4: &mut PinDriver<Gpio2, Output>,
) -> Option<char> {
    let mut data: char = 'e';
    let mut mark_read = false;

    col1.set_high().unwrap();
    col2.set_low().unwrap();
    col3.set_low().unwrap();
    col4.set_low().unwrap();

    if row1.is_high() { mark_read = true; data = '7'; }
    if row2.is_high() { mark_read = true; data = '4'; }
    if row3.is_high() { mark_read = true; data = '1'; }
    if row4.is_high() { mark_read = true; data = 'O'; }

    if !mark_read {
        col1.set_low().unwrap();
        col2.set_high().unwrap();
        col3.set_low().unwrap();
        col4.set_low().unwrap();

        if row1.is_high() { mark_read = true; data = '8'; }
        if row2.is_high() { mark_read = true; data = '5'; }
        if row3.is_high() { mark_read = true; data = '2'; }
        if row4.is_high() { mark_read = true; data = '0'; }
    }

    if !mark_read {
        col1.set_low().unwrap();
        col2.set_low().unwrap();
        col3.set_high().unwrap();
        col4.set_low().unwrap();

        if row1.is_high() { mark_read = true; data = '9'; }
        if row2.is_high() { mark_read = true; data = '6'; }
        if row3.is_high() { mark_read = true; data = '3'; }
        if row4.is_high() { mark_read = true; data = '='; }
    }

    if !mark_read {
        col1.set_low().unwrap();
        col2.set_low().unwrap();
        col3.set_low().unwrap();
        col4.set_high().unwrap();

        if row1.is_high() { data = '/'; }
        if row2.is_high() { data = '*'; }
        if row3.is_high() { data = '-'; }
        if row4.is_high() { data = '+'; }
    }

    col1.set_high().unwrap();
    col2.set_high().unwrap();
    col3.set_high().unwrap();
    col4.set_high().unwrap();

    FreeRtos::delay_ms(100);

    return Some(data);
}
