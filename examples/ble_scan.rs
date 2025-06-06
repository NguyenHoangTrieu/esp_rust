use esp32_nimble::{BLEDevice, BLEScan};
use esp_idf_svc::hal::task::block_on;

fn main() -> anyhow::Result<()> {
  esp_idf_svc::sys::link_patches();

  esp_idf_svc::log::EspLogger::initialize_default();
  log::set_max_level(log::LevelFilter::Debug);

  block_on(async {
    let ble_device = BLEDevice::take();
    let mut ble_scan = BLEScan::new();
    ble_scan.active_scan(true).interval(100).window(99);
    loop{
    ble_scan
      .start(ble_device, 5000, |device, data| {
        println!("Advertised Device: ({:?}, {:?})", device, data);
        None::<()>
      })
      .await?;
    
    println!("Scan end");
    // Delay before the next scan
    esp_idf_hal::delay::FreeRtos::delay_ms(3000);
    }
  })
}