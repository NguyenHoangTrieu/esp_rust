use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::*;

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let spi = peripherals.spi2;

    let sclk = peripherals.pins.gpio15;
    let serial_in = peripherals.pins.gpio16; // SDI
    let serial_out = peripherals.pins.gpio17; // SDO
    let cs_1 = peripherals.pins.gpio18;
    let cs_2 = peripherals.pins.gpio19;

    println!("Starting SPI loopback test");

    let driver = SpiDriver::new::<SPI2>(
        spi,
        sclk,
        serial_out,
        Some(serial_in),
        &SpiDriverConfig::new(),
    )?;

    let config_1 = config::Config::new().baudrate(26.MHz().into());
    let mut device_1 = SpiDeviceDriver::new(&driver, Some(cs_1), &config_1)?;

    let config_2 = config::Config::new().baudrate(13.MHz().into());
    let mut device_2 = SpiDeviceDriver::new(&driver, Some(cs_2), &config_2)?;

    let mut read = [0u8; 4];
    let write = [0xde, 0xad, 0xbe, 0xef];

    loop {
        // we are using thread::sleep here to make sure the watchdog isn't triggered
        FreeRtos::delay_ms(500);
        device_1.transfer(&mut read, &write)?;
        println!("Device 1: Wrote {write:x?}, read {read:x?}");

        let write_buf = [0xde, 0xad, 0xbe, 0xef];
        let mut write_in_place_buf = [0xde, 0xad, 0xbe, 0xef];
        let mut read_buf = [0; 8];

        println!("Device 2: To write {write_in_place_buf:x?} ... ");
        // cascade multiple operations with different buffer length into one transaction
        device_2.transaction(&mut [
            Operation::Write(&write_buf),
            Operation::TransferInPlace(&mut write_in_place_buf),
            Operation::Read(&mut read_buf),
        ])?;
        println!("... read {write_in_place_buf:x?}");
    }
}