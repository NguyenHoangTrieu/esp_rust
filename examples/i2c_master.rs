use std::thread;
use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::i2c::{I2c, I2cConfig, I2cDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::units::Hertz;
const SLAVE_ADDR: u8 = 0x22;

fn i2c_master_init<'d>(
    i2c: impl Peripheral<P = impl I2c> + 'd,
    sda: AnyIOPin,
    scl: AnyIOPin,
    baudrate: Hertz,
) -> anyhow::Result<I2cDriver<'d>> {
    let config = I2cConfig::new().baudrate(baudrate);
    let driver = I2cDriver::new(i2c, sda, scl, &config)?;
    Ok(driver)
}

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();
    let peripherals = Peripherals::take()?;

    let mut i2c_master = i2c_master_init(
        peripherals.i2c0,
        peripherals.pins.gpio21.into(), // SDA
        peripherals.pins.gpio22.into(), // SCL
        100.kHz().into(),
    )?;

    // Test 1: Simple write
    let tx_buf: [u8; 8] = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
    println!("MASTER: sending {:?}", tx_buf);
    i2c_master.write(SLAVE_ADDR, &tx_buf, BLOCK)?;

    // Test 2: Simple read
    let mut rx_buf: [u8; 8] = [0; 8];
    i2c_master.read(SLAVE_ADDR, &mut rx_buf, BLOCK)?;
    println!("MASTER: received {:?}", rx_buf);

    // Test 3: Register read/write
    let reg_addr: u8 = 0x05;
    let new_value: u8 = 0x42;

    // Read from register
    i2c_master.write(SLAVE_ADDR, &[reg_addr], BLOCK)?;
    let mut rx_buf: [u8; 1] = [0; 1];
    i2c_master.read(SLAVE_ADDR, &mut rx_buf, BLOCK)?;
    println!("MASTER: Read reg[{:#04x}] = {:#04x}", reg_addr, rx_buf[0]);

    // Write to register
    println!("MASTER: Write reg[{:#04x}] = {:#04x}", reg_addr, new_value);
    i2c_master.write(SLAVE_ADDR, &[reg_addr, new_value], BLOCK)?;

    // Read again
    i2c_master.write(SLAVE_ADDR, &[reg_addr], BLOCK)?;
    i2c_master.read(SLAVE_ADDR, &mut rx_buf, BLOCK)?;
    println!("MASTER: Re-read reg[{:#04x}] = {:#04x}", reg_addr, rx_buf[0]);

    loop{
        // Keep the program running
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
