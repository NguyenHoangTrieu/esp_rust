use esp_idf_hal::gpio::*;
use esp_idf_hal::prelude::*;
use esp_idf_hal::delay::*;
use esp_idf_hal::*;
use core::sync::atomic::{AtomicBool, Ordering};
mod simulator{
    pub mod e32_module;
    pub mod buffer;
}
use simulator::e32_module::*;
use simulator::buffer::*;

const BUFF_SIZE: usize = 256;
const MAX_WAIT_TIMES: u8 = 3;
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
    let timer_conf = timer::config::Config::new().auto_reload(false);
    let mut timer0 = timer::TimerDriver::new(peripherals.timer00, &timer_conf)?;
    timer0.set_alarm(timer0.tick_hz() / 1000000 * BYTE_TIME_19200)?;
    unsafe {
        timer0.subscribe(move || {
            TIMER0_EXPIRED.store(true, Ordering::Relaxed);
        })?;
    }
    timer0.enable_interrupt()?;
    timer0.enable_alarm(false)?;
    timer0.enable(false)?;
    // UART0: PC ↔ ESP32
    let uart0 = uart::UartDriver::new(
        peripherals.uart0,
        pins.gpio1,  // TX0
        pins.gpio3,  // RX0
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &uart::config::Config::default().baudrate(Hertz(19200)),
    )?;
    let mut timer1 = timer::TimerDriver::new(peripherals.timer01, &timer_conf)?;
    timer1.set_alarm(timer1.tick_hz() / 1000000 * BYTE_TIME_57600)?;
    unsafe {
        timer1.subscribe(move || {
            TIMER1_EXPIRED.store(true, Ordering::Relaxed);
        })?;
    }
    timer1.enable_interrupt()?;
    timer1.enable_alarm(false)?;
    timer1.enable(false)?;
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
    let mut low_wait = 0;
    let mut up_wait = 0;
    let mut low_last_size = 0;
    let mut up_last_size = 0;
    let mut buf = [0_u8; 1];
    let mut buf1 = [0_u8; 1];
    let mut params_buf = [0_u8; CONF_SIZE];
    uart0.write(b"ESP32 E32 Module Bridge\n")?;
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
                println!("State: Normal");
                // Read from UART0 (PC) → upper buffer
                while let Ok(_b) = uart0.read(&mut buf, BLOCK) {
                    println!("Read from UART0: {}", buf[0]);
                    upper_buffer.enqueue(buf[0]);
                }

                // Read from UART1 (STM32) → lower buffer
                while let Ok(_b) = uart1.read(&mut buf1, BLOCK) {
                    println!("Read from UART1: {}", buf1[0]);
                    lower_buffer.enqueue(buf1[0]);
                }

                // Handle lower_buffer → UART1
                if lower_buffer.available() > 0 {
                    let current_size = lower_buffer.available();
                    if current_size != low_last_size {
                        TIMER1_EXPIRED.store(false, Ordering::Relaxed);
                        timer1.enable_alarm(true)?;
                        timer1.enable(true)?;
                        low_wait = 0;
                        low_last_size = current_size;
                    } else if TIMER1_EXPIRED.load(Ordering::Relaxed) {
                        TIMER1_EXPIRED.store(false, Ordering::Relaxed);
                        timer1.enable(true)?;
                        timer1.enable_alarm(true)?;
                        low_wait += 1;
                        if low_wait >= MAX_WAIT_TIMES {
                            TIMER1_EXPIRED.store(false, Ordering::Relaxed);
                            timer1.enable(false)?;
                            timer1.enable_alarm(false)?;
                            aux.set_low()?;
                            let data = lower_buffer.deallqueue();
                            uart0.write(&data)?;
                            aux.set_high()?;
                        }
                    }
                }

                // Handle upper_buffer → UART0
                if upper_buffer.available() > 0 {
                    let current_size = upper_buffer.available();
                    if current_size != up_last_size {
                        TIMER0_EXPIRED.store(false, Ordering::Relaxed);
                        timer0.enable(true)?;
                        timer0.enable_alarm(true)?;
                        up_wait = 0;
                        up_last_size = current_size;
                    } else if TIMER0_EXPIRED.load(Ordering::Relaxed){
                        TIMER0_EXPIRED.store(false, Ordering::Relaxed);
                        timer0.enable(true)?;
                        timer0.enable_alarm(true)?;
                        up_wait += 1;
                        if up_wait >= MAX_WAIT_TIMES {
                            TIMER0_EXPIRED.store(false, Ordering::Relaxed);
                            timer0.enable(false)?;
                            timer0.enable_alarm(false)?;
                            aux.set_low()?;
                            aux.set_low()?;
                            let data = upper_buffer.deallqueue();
                            uart1.write(&data)?;
                            aux.set_high()?;
                        }
                    }
                }
            }
            E32State::Sleep => {
                println!("State: Sleep");
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
                println!("State: WakeUp/PowerSaving");
                // TODO: WAKE_UP / POWER_SAVING mode
            }
        }
    }
}
