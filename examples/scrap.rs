use esp_idf_sys as _; // khởi tạo esp-idf runtime
use esp_idf_svc::log::EspLogger;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::timer::*;
use esp_idf_hal::task::notification::Notification;
use std::thread;
use std::time::Duration;
use std::num::NonZeroU32;
//use std::sync::atomic::{AtomicBool, Ordering};
//use std::sync::Arc;

fn main() -> anyhow::Result<()>  {
    EspLogger::initialize_default();
    esp_idf_hal::sys::link_patches();
    let mut per = Peripherals::take()?;
    let gpio2 = core::mem::replace(&mut per.pins.gpio2, unsafe { core::mem::zeroed() });
    let gpio4 = core::mem::replace(&mut per.pins.gpio4, unsafe { core::mem::zeroed() });
    let _thread0 = std::thread::Builder::new()
    // thread with loop function:
    .stack_size(4000) 
    .spawn(move || {
        task1_handle(gpio2);
    })?;
    // thread with none loop function (thread only run once):
    let thread1 = std::thread::Builder::new()
        .stack_size(4000)
        .spawn(move || task2_handle(gpio4))?;
    thread1.join().unwrap();
    //set timer:
    // The default clock-divider is -> 80
    let timer_conf = config::Config::new().auto_reload(true);
    let mut timer = TimerDriver::new(per.timer00, &timer_conf)?;
    timer.set_alarm(timer.tick_hz() / 2)?; // tick_hz = 1_000_000, so this will set the alarm to 0.5 seconds
    let notification = Notification::new();
    let notifier = notification.notifier();
    unsafe {
        timer.subscribe(move || {
            let bitset = 0b10001010101;
            notifier.notify_and_yield(NonZeroU32::new(bitset).unwrap());
        })?;
    }
    timer.enable_interrupt()?;
    timer.enable_alarm(true)?;
    timer.enable(true)?;
    // main loop (do not using thread::sleep because it will block the main thread and affect timer ISR)
    loop {
        let bitset = notification.wait(esp_idf_hal::delay::BLOCK);
        if let Some(bitset) = bitset {
            log::info!("got event with bits {bitset:#b} from ISR");
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

fn task2_handle(gpio4 : esp_idf_hal::gpio::Gpio4) {
        let mut led = {
            PinDriver::output(gpio4).unwrap()
        };
        log::info!("Task 2 blink Led 4");
        for _i in 0..10 {
            led.toggle().unwrap();
            thread::sleep(Duration::from_millis(500));
        }
        
}