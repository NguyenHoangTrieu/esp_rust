use esp_idf_hal::gpio::*;
use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::*;
use esp_idf_hal::delay::*;
use embedded_hal::serial::{Read, Write};

mod e32_module;
mod buffer;

use simulator::e32_module::*;
use simulator::buffer::*;

const BUFF_SIZE: usize = 256;
const MAX_WAIT_TIMES: u8 = 3;
const BYTE_TIME_115200: u64 = 70;  // us
const BYTE_TIME_9600: u64 = 833;   // us

#[derive(PartialEq, Copy, Clone)]
enum E32State {
    Normal,
    WakeUp,
    PowerSaving,
    Sleep,
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    // Init peripherals
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // UART0: STM32 ↔ ESP32
    let mut uart0 = UartDriver::new(
        peripherals.uart0,
        pins.gpio1,  // TX0
        pins.gpio3,  // RX0
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &Config::default().baudrate(Hertz(115_200)),
    )?;

    // UART1: ESP32 ↔ Laptop
    let mut uart1 = UartDriver::new(
        peripherals.uart1,
        pins.gpio12, // TX1
        pins.gpio13, // RX1
        Option::<AnyIOPin>::None,   
        Option::<AnyIOPin>::None,
        &Config::default().baudrate(Hertz(9_600)),
    )?;

    let mut m0 = PinDriver::input(pins.gpio4)?; // M0
    let mut m1 = PinDriver::input(pins.gpio16)?; // M1
    let mut aux = PinDriver::output(pins.gpio2)?; // AUX
    aux.set_high()?; // AUX HIGH ban đầu

    let mut lower_buffer = Buffer::new(BUFF_SIZE);
    let mut upper_buffer = Buffer::new(BUFF_SIZE);
    let mut e32 = E32Module::new();

    let mut state = E32State::Sleep;
    let mut delay = Delay::new_default();

    let mut current_baudrate = 115_200;

    let mut low_buff_time = delay.get_systimer_count();
    let mut up_buff_time = delay.get_systimer_count();
    let mut low_wait = 0;
    let mut up_wait = 0;
    let mut low_last_size = 0;
    let mut up_last_size = 0;

    loop {
        // Check state change
        let new_state = match (m0.is_high(), m1.is_high()) {
            (Ok(false), Ok(false)) => E32State::Normal,
            (Ok(true), Ok(false)) => E32State::WakeUp,
            (Ok(false), Ok(true)) => E32State::PowerSaving,
            (Ok(true), Ok(true)) => E32State::Sleep,
            _ => state,
        };

        if new_state != state {
            state = new_state;
            if state == E32State::Sleep {
                uart0.set_baudrate(Hertz(CONF_BAUD as u32))?;
                delay.delay_ms(2u32);
            } else {
                uart0.set_baudrate(Hertz(115_200))?;
                delay.delay_ms(1u32);
            }
        }

        match state {
            E32State::Normal => {
                // Read from UART0 (STM32) → lower buffer
                while let Ok(b) = uart0.read() {
                    lower_buffer.enqueue(b);
                }

                // Read from UART1 (Laptop) → upper buffer
                while let Ok(b) = uart1.read() {
                    upper_buffer.enqueue(b);
                }

                // Handle lower_buffer → UART1
                if lower_buffer.available() > 0 {
                    let now = delay.get_systimer_count();
                    let current_size = lower_buffer.available();
                    if current_size != low_last_size {
                        low_buff_time = now;
                        low_wait = 0;
                        low_last_size = current_size;
                    } else if now - low_buff_time >= BYTE_TIME_115200 {
                        low_buff_time = now;
                        low_wait += 1;
                        if low_wait >= MAX_WAIT_TIMES {
                            aux.set_low()?;
                            let data = lower_buffer.deallqueue();
                            uart1.write_bytes(&data)?;
                            aux.set_high()?;
                        }
                    }
                }

                // Handle upper_buffer → UART0
                if upper_buffer.available() > 0 {
                    let now = delay.get_systimer_count();
                    let current_size = upper_buffer.available();
                    if current_size != up_last_size {
                        up_buff_time = now;
                        up_wait = 0;
                        up_last_size = current_size;
                    } else if now - up_buff_time >= BYTE_TIME_9600 {
                        up_buff_time = now;
                        up_wait += 1;
                        if up_wait >= MAX_WAIT_TIMES {
                            aux.set_low()?;
                            let data = upper_buffer.deallqueue();
                            for b in data {
                                uart0.write(&b)?;
                            }
                            aux.set_high()?;
                        }
                    }
                }
            }
            E32State::Sleep => {
                if let Ok(b) = uart0.read() {
                    aux.set_low()?;
                    let mut buffer = [0u8; CONF_SIZE];
                    buffer[0] = b;
                    for i in 1..CONF_SIZE {
                        buffer[i] = uart0.read().unwrap_or(0);
                    }
                    e32.set_params(&buffer, CONF_SIZE);
                    uart0.write_bytes(&buffer)?;
                    aux.set_high()?;
                }
            }
            _ => {
                // TODO: WAKE_UP / POWER_SAVING mode
            }
        }
    }
}
