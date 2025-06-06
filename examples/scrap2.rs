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
use esp_idf_hal::timer::*;
use esp_idf_hal::timer::config;
static mut SHARED_ADC2: Option<AdcDriver::<ADC2>> = None;

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
    col1.set_high().unwrap();
    col2.set_high().unwrap();
    col3.set_high().unwrap();
    FreeRtos::delay_ms(100);
    return Some(data);
}

/// Task 1: Đợi thông báo từ main
unsafe extern "C" fn task1(_: *mut core::ffi::c_void) {
    let mut notified_value: u32 = 0;
    let per = Peripherals::new();
    let mut led = PinDriver::output(per.pins.gpio4).unwrap();
    let adc = SHARED_ADC2.as_ref().unwrap();
    let config = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };
    let mut adc_pin = AdcChannelDriver::new(adc, per.pins.gpio14, &config).expect("Error");
    led.toggle().unwrap();
    loop {
        let result = xTaskGenericNotifyWait(
            0,
            0,
            0,
            &mut notified_value as *mut u32,
            0xffffffff, // Timeout 5s
        );

        if result == 1 {
            println!("ADC value gpio14: {}", adc.read(&mut adc_pin).unwrap());
            println!("[Task 1] Received Notification! Value = {}", notified_value);
        }
    }
}

/// Task 2: Nháy GPIO2 liên tục mỗi 500ms
unsafe extern "C" fn task2(_: *mut core::ffi::c_void) {
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
    //setup GPIO for keypad
    let mut row1 = PinDriver::input(&mut peripherals.pins.gpio21)?;
    let mut row2 = PinDriver::input(&mut peripherals.pins.gpio19)?;
    let mut row3 = PinDriver::input(&mut peripherals.pins.gpio18)?;
    let mut row4 = PinDriver::input(&mut peripherals.pins.gpio5)?;
    row1.set_pull(Pull::Down)?;
    row2.set_pull(Pull::Down)?;
    row3.set_pull(Pull::Down)?;
    row4.set_pull(Pull::Down)?;
    row1.set_interrupt_type(InterruptType::PosEdge)?;
    row2.set_interrupt_type(InterruptType::PosEdge)?;
    row3.set_interrupt_type(InterruptType::PosEdge)?;
    row4.set_interrupt_type(InterruptType::PosEdge)?;
    unsafe {
        SHARED_ADC2 = Some(adc2);
    }
    // Tạo Task 1 và Task 2
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

        // Tạo Task 2 (blink GPIO2)
        xTaskCreatePinnedToCore(
            Some(task2),
            CString::new("Task2")?.into_raw(),
            4096,
            ptr::null_mut(),
            5,
            ptr::null_mut(),
            1,
        );
    }
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
    let mut col1 = PinDriver::output(peripherals.pins.gpio17).unwrap();
    let mut col2 = PinDriver::output(peripherals.pins.gpio16).unwrap();
    let mut col3 = PinDriver::output(peripherals.pins.gpio4).unwrap();

    col1.set_high().unwrap();
    col2.set_high().unwrap();
    col3.set_high().unwrap();
    //setup timer for notification
    let timer_conf = config::Config::new().auto_reload(true);
    let mut timer = TimerDriver::new(peripherals.timer00, &timer_conf)?;
    timer.set_alarm(timer.tick_hz() * 3)?;
    let notifier = notification.notifier();
    unsafe {
        timer.subscribe(move || {
            notifier.notify(NonZeroU32::new(2).unwrap());
        })?;
    }
    timer.enable_interrupt()?;
    timer.enable_alarm(true)?;
    timer.enable(true)?;
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
                    &mut col1, &mut col2, &mut col3,
                ) {
                    if Some(key) != Some('e') {
                        println!("[Main] Phím nhấn: {}", key);
                    }
                }
            }
            Some(nz) if nz.get() == 2 => {
                println!("[Main] Sending notification to Task 1");
                unsafe {
                    let mut previous_value: u32 = 0;
                    xTaskGenericNotify(
                        handle1,
                        0,
                        0x01,
                        eNotifyAction_eSetBits,
                        &mut previous_value,
                    );
                }
            }
            Some(_) => {}
            None => {}
        }
    }
}