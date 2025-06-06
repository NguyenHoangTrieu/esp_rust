use esp_idf_hal::{delay::FreeRtos, gpio::*};
use esp_idf_hal::prelude::*;
use esp_idf_hal::task::notification::Notification;
use core::num::NonZeroU32;
use std::time::Duration;
use std::thread;

fn read_keypad(
    row1: &mut PinDriver<Gpio21, Input>,
    row2: &mut PinDriver<Gpio19, Input>,
    row3: &mut PinDriver<Gpio18, Input>,
    row4: &mut PinDriver<Gpio5, Input>,
    col1: &mut PinDriver<Gpio17, Output>,
    col2: &mut PinDriver<Gpio16, Output>,
    col3: &mut PinDriver<Gpio4, Output>,
) -> Option<char> {
    let mut data: char  = 'e';
    let mut mark_read: bool = false;
    col1.set_high().unwrap();
    col2.set_low().unwrap();
    col3.set_low().unwrap();
    if row1.is_high() { mark_read = true; data = '1'; }
    if row2.is_high() { mark_read = true; data = '4'; }
    if row3.is_high() { mark_read = true; data = '7'; }
    if row4.is_high() { mark_read = true; data = '*'; }

    if !mark_read {
        col1.set_low().unwrap();
        col2.set_high().unwrap();
        col3.set_low().unwrap();
        if row1.is_high() { mark_read = true; data = '2'; }
        if row2.is_high() { mark_read = true; data = '5'; }
        if row3.is_high() { mark_read = true; data = '8'; }
        if row4.is_high() { mark_read = true; data = '0'; }
    }
    if !mark_read {
        col1.set_low().unwrap();
        col2.set_low().unwrap();
        col3.set_high().unwrap();
        if row1.is_high() { data = '3'; }
        if row2.is_high() { data = '6'; }
        if row3.is_high() { data = '9'; }
        if row4.is_high() { data = '#'; } 
    }
    FreeRtos::delay_ms(100);
    return Some(data);
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    let mut peripherals = Peripherals::take().unwrap();
    let gpio2 = core::mem::replace(&mut peripherals.pins.gpio2, unsafe { core::mem::zeroed() });
    let mut row1 = PinDriver::input(&mut peripherals.pins.gpio21)?;
    let mut row2 = PinDriver::input(&mut peripherals.pins.gpio19)?;
    let mut row3 = PinDriver::input(&mut peripherals.pins.gpio18)?;
    let mut row4 = PinDriver::input(&mut peripherals.pins.gpio5)?;
    row1.set_pull(Pull::Down)?;
    row2.set_pull(Pull::Down)?;
    row3.set_pull(Pull::Down)?;
    row4.set_pull(Pull::Down)?;
    row1.set_interrupt_type(InterruptType::AnyEdge)?;
    row2.set_interrupt_type(InterruptType::AnyEdge)?;
    row3.set_interrupt_type(InterruptType::AnyEdge)?;
    row4.set_interrupt_type(InterruptType::AnyEdge)?;
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
    let _thread0 = std::thread::Builder::new()
        .stack_size(4000)
        .spawn(move || {
            task1_handle(gpio2);
        })?;
    let mut col1 = PinDriver::output(&mut peripherals.pins.gpio17)?;
    let mut col2 = PinDriver::output(&mut peripherals.pins.gpio16)?;
    let mut col3 = PinDriver::output(&mut peripherals.pins.gpio4)?;

    loop {
        row1.enable_interrupt()?;
        row2.enable_interrupt()?;
        row3.enable_interrupt()?;
        row4.enable_interrupt()?;
        col1.set_high()?;
        col2.set_high()?;
        col3.set_high()?;
        let bitset = notification.wait(esp_idf_hal::delay::BLOCK);
        if bitset == Some(NonZeroU32::new(1).unwrap())  {
            if let Some(key) = read_keypad(
                &mut row1, &mut row2, &mut row3, &mut row4,
                &mut col1, &mut col2, &mut col3,
            ) {
                if Some(key) != Some('e') {
                    println!("Phím nhấn: {}", key);
                }
            }
        }
    }
}

fn task1_handle(gpio2: esp_idf_hal::gpio::Gpio2) {
    let mut led = {
        PinDriver::output(gpio2).unwrap()
    };
    loop {
        log::info!("Task 1: Toggle LED");
        led.toggle().unwrap();
        thread::sleep(Duration::from_secs(2));
    }
}