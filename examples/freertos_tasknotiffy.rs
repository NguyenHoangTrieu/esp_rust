use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_sys::*;
use std::ffi::CString;
use std::ptr;
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::{oneshot::*, ADC2};

static mut SHARED_ADC2: Option<AdcDriver::<ADC2>> = None;
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
        FreeRtos::delay_ms(1000);
    }
}

/// Task 2: Nháy GPIO2 liên tục mỗi 500ms
unsafe extern "C" fn task2(_: *mut core::ffi::c_void) {
    let peripherals = Peripherals::new();
    let mut led = PinDriver::output(peripherals.pins.gpio2).unwrap();
    let adc = SHARED_ADC2.as_ref().unwrap();
    let config = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };
    let mut adc_pin = AdcChannelDriver::new(adc, peripherals.pins.gpio25, &config).expect("Error");
    loop {
        led.toggle().unwrap();
        println!("ADC value gpio25: {}", adc.read(&mut adc_pin).unwrap());
        FreeRtos::delay_ms(500);
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    let mut handle1: esp_idf_sys::TaskHandle_t = ptr::null_mut();
    let per = Peripherals::take()?;
    let adc2 = AdcDriver::new(per.adc2).unwrap();
    unsafe {
        SHARED_ADC2 = Some(adc2);
    }
    unsafe {
        // Tạo Task 1
        xTaskCreatePinnedToCore(
            Some(task1),
            CString::new("Task1")?.into_raw(),
            4096,
            ptr::null_mut(),
            5,
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

    // Gửi thông báo cho Task 1 mỗi 3 giây
    loop {
        println!("[Main] Sending notification to Task 1");
        unsafe {
            if !handle1.is_null() {
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
        FreeRtos::delay_ms(3000);
    }
}
