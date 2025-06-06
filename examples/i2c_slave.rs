use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::i2c::{I2c, I2cSlaveConfig, I2cSlaveDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::AnyIOPin;

const SLAVE_ADDR: u8 = 0x22;
const SLAVE_BUFFER_SIZE: usize = 128;

fn i2c_slave_init<'d>(
    i2c: impl Peripheral<P = impl I2c> + 'd,
    sda: AnyIOPin,
    scl: AnyIOPin,
    buflen: usize,
    slave_addr: u8,
) -> anyhow::Result<I2cSlaveDriver<'d>> {
    let config = I2cSlaveConfig::new()
        .rx_buffer_length(buflen)
        .tx_buffer_length(buflen);
    let driver = I2cSlaveDriver::new(i2c, sda, scl, slave_addr, &config)?;
    Ok(driver)
}

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();
    let peripherals = Peripherals::take()?;

    let mut i2c_slave = i2c_slave_init(
        peripherals.i2c1,
        peripherals.pins.gpio18.into(), // SDA
        peripherals.pins.gpio19.into(), // SCL
        SLAVE_BUFFER_SIZE,
        SLAVE_ADDR,
    )?;

    let mut data: [u8; 256] = [0; 256];

    loop {
        let mut reg_addr: [u8; 1] = [0];
        let res = i2c_slave.read(&mut reg_addr, BLOCK);
        if res.is_err() {
            println!("SLAVE: Failed to read reg addr");
            continue;
        }

        let mut rx_data: [u8; 1] = [0];
        match i2c_slave.read(&mut rx_data, 0) {
            Ok(_) => {
                println!(
                    "SLAVE: write {:#04x} -> reg[{:#04x}]",
                    rx_data[0], reg_addr[0]
                );
                data[reg_addr[0] as usize] = rx_data[0];
            }
            Err(_) => {
                let val = data[reg_addr[0] as usize];
                println!(
                    "SLAVE: read reg[{:#04x}] -> {:#04x}",
                    reg_addr[0], val
                );
                i2c_slave.write(&[val], BLOCK)?;
            }
        }
    }
}
