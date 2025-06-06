use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_sys::*;
use std::ffi::CString;
use std::ptr;
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::{oneshot::*, ADC2};
use esp_idf_hal::task::notification::Notification;
use core::num::NonZeroU32;
use esp_idf_hal::prelude::*;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    prelude::*,
    text::Text,
    pixelcolor::*,
};
use esp_idf_hal::i2c::*;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
mod mod_lib {
    pub mod matrix4x4;
}
use mod_lib::matrix4x4::read_keypad;
use core::fmt::Write;

static mut SHARED_ADC2: Option<AdcDriver::<ADC2>> = None;

/// Task 1: Nháy GPIO2 liên tục mỗi 500ms
unsafe extern "C" fn task1(_: *mut core::ffi::c_void) {
    let peripherals = Peripherals::new();
    let mut led = PinDriver::output(peripherals.pins.gpio2).unwrap();
    let mut buzzler = PinDriver::output(peripherals.pins.gpio13).unwrap();
    let adc = SHARED_ADC2.as_ref().unwrap();
    let config = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };
    let mut adc_pin1 = AdcChannelDriver::new(adc, peripherals.pins.gpio25, &config).expect("Error");
    let mut adc_pin2 = AdcChannelDriver::new(adc, peripherals.pins.gpio26, &config).expect("Error");
    loop {
        led.toggle().unwrap();
        let result1 = adc.read(&mut adc_pin1);
        let result2 = adc.read(&mut adc_pin2);
        println!("[Task 2] ADC value gpio25 gpio26: {}, {}", result1.unwrap(), result2.unwrap());
        if result1.unwrap() > 1000 || result2.unwrap() > 800 {
            buzzler.set_low().unwrap();
            println!("[Task 2] IR sensor triggered or ADC value high, buzzler ON");
        } else {
            buzzler.set_high().unwrap();
            
        }
        FreeRtos::delay_ms(500);
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    let mut handle1: esp_idf_sys::TaskHandle_t = ptr::null_mut();
    let mut peripherals = Peripherals::take()?;
    let adc2 = AdcDriver::new(peripherals.adc2).unwrap();
    unsafe {
        SHARED_ADC2 = Some(adc2);
    }
    // Tạo Task:
    unsafe {
        // Tạo Task 1
        xTaskCreatePinnedToCore(
            Some(task1),
            CString::new("Task1")?.into_raw(),
            4096,
            ptr::null_mut(),
            4,
            &mut handle1,
            1,
        );
    }
    //setup GPIO for keypad
    let mut row1 = PinDriver::input(&mut peripherals.pins.gpio2)?;
    let mut row2 = PinDriver::input(&mut peripherals.pins.gpio0)?;
    let mut row3 = PinDriver::input(&mut peripherals.pins.gpio4)?;
    let mut row4 = PinDriver::input(&mut peripherals.pins.gpio16)?;
    row1.set_pull(Pull::Down)?;
    row2.set_pull(Pull::Down)?;
    row3.set_pull(Pull::Down)?;
    row4.set_pull(Pull::Down)?;
    row1.set_interrupt_type(InterruptType::PosEdge)?;
    row2.set_interrupt_type(InterruptType::PosEdge)?;
    row3.set_interrupt_type(InterruptType::PosEdge)?;
    row4.set_interrupt_type(InterruptType::PosEdge)?;
    let mut col1 = PinDriver::output(peripherals.pins.gpio26).unwrap();
    let mut col2 = PinDriver::output(peripherals.pins.gpio25).unwrap();
    let mut col3 = PinDriver::output(peripherals.pins.gpio33).unwrap();
    let mut col4 = PinDriver::output(peripherals.pins.gpio32).unwrap();

    //setup notification for keypad rows
    let notification = Notification::new();
    let notifier1 = notification.notifier();
    let notifier2 = notification.notifier();
    let notifier3 = notification.notifier();
    let notifier4 = notification.notifier();
    unsafe {
        row1.subscribe(move || {
            notifier1.notify(NonZeroU32::new(1).unwrap());
        })?;
        row2.subscribe(move || {
            notifier2.notify(NonZeroU32::new(1).unwrap());
        })?;
        row3.subscribe(move || {
            notifier3.notify(NonZeroU32::new(1).unwrap());
        })?;
        row4.subscribe(move || {
            notifier4.notify(NonZeroU32::new(1).unwrap());
        })?;
    }

    col1.set_high().unwrap();
    col2.set_high().unwrap();
    col3.set_high().unwrap();
    col4.set_high().unwrap();

    // initialize OLED display:
    let peripherals = Peripherals::take()?;
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio5;
    let scl = peripherals.pins.gpio6;

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;

    let interface = I2CDisplayInterface::new(i2c_driver);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    //Main loop:
    loop {
        row1.enable_interrupt()?;
        row2.enable_interrupt()?;
        row3.enable_interrupt()?;
        row4.enable_interrupt()?;
        let bitset = notification.wait(esp_idf_hal::delay::BLOCK);
        match bitset {
            Some(nz) if nz.get() == 1 => {
                if let Some(key) = read_keypad(
                    &mut row1, &mut row2, &mut row3, &mut row4,
                    &mut col1, &mut col2, &mut col3, &mut col4,
                ) {
                    if Some(key) != Some('e') {
                        // Tạo chuỗi để hiển thị ký tự
                        let mut message = String::new();
                        write!(message, "Key: {}", key).unwrap();

                        // Xoá màn hình
                        display.clear(BinaryColor::Off).unwrap();

                        // Vẽ chuỗi lên màn hình
                        Text::new(&message, Point::new(0, 20), style)
                            .draw(&mut display)
                            .unwrap();
                        display.flush().unwrap();
                    }
                }
            }
            Some(_) => {}
            None => {}
        }
    }
}