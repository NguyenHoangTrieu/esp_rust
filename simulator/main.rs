use esp_idf_hal::gpio::*;
use esp_idf_hal::prelude::*;
use esp_idf_hal::delay::*;
use esp_idf_hal::*;
use esp_idf_svc::timer::*;
use std::time::Duration;
use core::sync::atomic::{AtomicBool, Ordering};
mod simulator{
    pub mod e32_module;
    pub mod buffer;
}
use simulator::e32_module::*;
use simulator::buffer::*;

const BUFF_SIZE: usize = 256;
const MAX_WAIT_TIMES: u64 = 3;
const BYTE_TIME_57600: u64 = 70*2;  // us
const BYTE_TIME_19200: u64 = 416;   // us
static TIMER0_EXPIRED: AtomicBool = AtomicBool::new(false);
static TIMER1_EXPIRED: AtomicBool = AtomicBool::new(false);


fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    // Init peripherals
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    // init timer for UART
    let timer_service = EspTimerService::new()?;
    let timer0 = timer_service.timer(move || {
        TIMER0_EXPIRED.store(true, Ordering::Relaxed);
    })?;
    let timer1 = timer_service.timer(move || {
        TIMER1_EXPIRED.store(true, Ordering::Relaxed);
    })?;
    // UART0: PC ↔ ESP32
    let uart0 = uart::UartDriver::new(
        peripherals.uart0,
        pins.gpio1,  // TX0
        pins.gpio3,  // RX0
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &uart::config::Config::default().baudrate(Hertz(19200)),
    )?;

    // UART1: ESP32 ↔ STM32
    let uart1 = uart::UartDriver::new(
        peripherals.uart1,
        pins.gpio12, // TX1
        pins.gpio13, // RX1
        Option::<AnyIOPin>::None,   
        Option::<AnyIOPin>::None,
        &uart::config::Config::default().baudrate(Hertz(57600)),
    )?;

    let m0 = PinDriver::input(pins.gpio4)?; // M0
    let m1 = PinDriver::input(pins.gpio16)?; // M1
    let mut aux = PinDriver::output(pins.gpio2)?; // AUX
    aux.set_high()?; // AUX HIGH ban đầu

    let mut lower_buffer = Buffer::new(BUFF_SIZE);
    let mut upper_buffer = Buffer::new(BUFF_SIZE);
    let mut e32 = E32Module::new();

    let mut state;
    let mut buf:[u8; BUFF_SIZE] = [0; BUFF_SIZE];
    let mut buf1: [u8; BUFF_SIZE] = [0; BUFF_SIZE];
    let mut params_buf = [0_u8; CONF_SIZE];
    uart0.write(b"ESP32 E32 Module Bridge\n")?;
    let mut b = 0;
    let mut b1 = 0;
    loop {
        // Check state change
        let new_state = match (m0.is_high(), m1.is_high()) {
            (false, false) => E32State::Normal,
            (true, false) => E32State::WakeUp,
            (false, true) => E32State::PowerSaving,
            (true, true) => E32State::Sleep,
        };
        state = new_state;
        match state {
            E32State::Normal => {
                // Read from UART0 (PC) → upper buffer

                match uart0.read(&mut buf, 100){
                    Ok (n)=> {b = n},
                    Err(_) => {}
                }
                println!("UART0 read: {}", b);
                if b > 0 {
                    for x in buf.iter_mut() {
                        upper_buffer.enqueue(*x);
                        *x = 0;
                    }
                    b = 0;
                    //let _ = buf.iter().map(|x| upper_buffer.enqueue(*x));
                }
                // Read from UART1 (STM32) → lower buffer
                match uart1.read(&mut buf1, 100){
                    Ok (n)=> b1 = n,
                    Err(_) => {}

                }
                println!("UART1 read: {}", b1);
                if b1 > 0 {
                    for x in buf1.iter_mut() {
                        lower_buffer.enqueue(*x);
                        *x = 0;
                    }
                    b1 = 0;
                    //let _ = buf1.iter().map(|x| lower_buffer.enqueue(*x));
                }
                // Handle lower_buffer → UART0
                if lower_buffer.available() > 0 {
                    timer1.after(Duration::from_micros(BYTE_TIME_57600 * MAX_WAIT_TIMES))?;
                    if TIMER1_EXPIRED.load(Ordering::Relaxed) {
                        TIMER1_EXPIRED.store(false, Ordering::Relaxed);
                        let n = lower_buffer.available();
                        println!("lower_buffer available: {}", n);
                        let data = lower_buffer.deallqueue();
                        uart0.write(&data[..n])?;
                    }
                }
                // Handle upper_buffer → UART1
                if upper_buffer.available() > 0 {
                    timer0.after(Duration::from_micros(BYTE_TIME_19200 * MAX_WAIT_TIMES))?;
                    if TIMER0_EXPIRED.load(Ordering::Relaxed) {
                        TIMER0_EXPIRED.store(false, Ordering::Relaxed);
                        let n = upper_buffer.available();
                        println!("upper_buffer available: {}", n);
                        let data = upper_buffer.deallqueue();
                        uart1.write(&data[..n])?;
                    }
                }
            }
            E32State::Sleep => {
                if let Ok(_b) = uart1.read(&mut params_buf, BLOCK) {
                    aux.set_low()?;
                    let result = e32.input_command(&params_buf.as_ref(), params_buf.len());
                    uart1.write(result.as_bytes())?;
                    let new_baud_rate = match e32.uart_bps {
                        UartBps::Bps1200 => Hertz(1200),
                        UartBps::Bps2400 => Hertz(2400),
                        UartBps::Bps4800 => Hertz(4800),
                        UartBps::Bps9600 => Hertz(9600),
                        UartBps::Bps19200 => Hertz(19200),
                        UartBps::Bps38400 => Hertz(38400),
                        UartBps::Bps57600 => Hertz(57600),
                        UartBps::Bps115200 => Hertz(115200),

                    };
                    let new_air_data_rate = match e32.air_data_rate {
                        AirDataRate::Rate300 => Hertz(300),
                        AirDataRate::Rate1200 => Hertz(1200),
                        AirDataRate::Rate2400 => Hertz(2400),
                        AirDataRate::Rate4800 => Hertz(4800),
                        AirDataRate::Rate9600 => Hertz(9600),
                        AirDataRate::Rate19200 => Hertz(19200),
                    };
                    uart0.change_baudrate(new_air_data_rate)?;
                    uart1.change_baudrate(new_baud_rate)?;
                    aux.set_high()?;
                }
            }
            _ => {
                // TODO: WAKE_UP / POWER_SAVING mode
            }
        }
    }
}
