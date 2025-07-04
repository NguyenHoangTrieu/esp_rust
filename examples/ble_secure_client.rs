use esp32_nimble::{enums::*, utilities::BleUuid, BLEDevice, BLEScan};
use log::*;
use esp_idf_svc::hal::{ // ESP-IDF Hardware Abstraction Layer
    prelude::Peripherals, // Access to board peripherals
    task::block_on,        // Used to block on async execution
    timer::{TimerConfig, TimerDriver}, // Timer functionality
};
// Define the service UUID the central is looking for
const SERVICE_UUID: BleUuid = BleUuid::Uuid16(0xABCD);

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    // Initialize a timer using TIMER0 with default configuration
    let mut timer = TimerDriver::new(peripherals.timer00, &TimerConfig::new())?;

    // Run the BLE workflow asynchronously
    block_on(async {
    // Take ownership of the BLE device (singleton)
    let ble_device = BLEDevice::take();

    // Set BLE power level to +9 dBm (maximum)
    ble_device.set_power(PowerType::Default, PowerLevel::P9)?;

    // Configure BLE security settings:
    // - Require all authentication features (e.g., bonding, MITM protection)
    // - Set IO capabilities to KeyboardOnly (enables Passkey Entry method)
    ble_device
      .security()
      .set_auth(AuthReq::all())
      .set_io_cap(SecurityIOCap::KeyboardOnly);

    // Create a new BLE scanner
    let mut ble_scan = BLEScan::new();

    // Start an active scan for 10 seconds (10000 ms)
    // Scan interval: 100 units, window: 99 units (high duty scanning)
    // If a device advertises the desired service UUID, capture it
    let device = ble_scan
      .active_scan(true)
      .interval(100)
      .window(99)
      .start(ble_device, 10000, |device, data| {
        if data.is_advertising_service(&SERVICE_UUID) {
          return Some(*device); // Found matching device
        }
        // if let Some(name_bytes) = data.name() {
        //     if let Ok(name) = core::str::from_utf8(name_bytes) {
        //         if name.contains("Iphone") {
        //             return Some(*device);
        //         }
        //     }
        // }
        None
      })
      .await?;

    // If no device was found, log a warning and exit
    let Some(device) = device else {
      ::log::warn!("device not found");
      return anyhow::Ok(());
    };

    // Print info about the found advertised device
    info!("Advertised Device: {:?}", device);

    // Create a new BLE GATT client
    let mut client = ble_device.new_client();

    // Connect to the target device using its BLE address
    client.connect(&device.addr()).await?;

    // Set the passkey (for Passkey Entry pairing) to 123456
    client.on_passkey_request(|| 123456);

    // Start the secure pairing/bonding process
    client.secure_connection().await?;

    // Get the primary service by UUID from the remote device
    let service = client.get_service(SERVICE_UUID).await?;

    // === Read non-secure characteristic ===
    // Get the characteristic with UUID 0x1234 (assumed to not require encryption)
    let non_secure_characteristic = service.get_characteristic(BleUuid::Uuid16(0x1234)).await?;

    // Read its value
    let value = non_secure_characteristic.read_value().await?;

    // Log the read value as a UTF-8 string
    ::log::info!(
      "{:?} value: {}",
      non_secure_characteristic.uuid(),
      core::str::from_utf8(&value)?
    );

    // === Read secure characteristic ===
    // Get the characteristic with UUID 0x1235 (assumed to require encryption)
    let secure_characteristic = service.get_characteristic(BleUuid::Uuid16(0x1235)).await?;

    // Read its value
    let value = secure_characteristic.read_value().await?;

    // Log the read value
    ::log::info!(
      "{:?} value: {}",
      secure_characteristic.uuid(),
      core::str::from_utf8(&value)?
    );

    let notification_characteristic = service.get_characteristic(BleUuid::Uuid16(0x1236)).await?;
    // Check if the characteristic supports notifications
    if !notification_characteristic.can_notify() {
      ::log::error!("characteristic can't notify: {}", notification_characteristic);
      return anyhow::Ok(()); // Exit gracefully
    }
    // Log the read value
    ::log::info!("subscribe to {}", notification_characteristic);
    notification_characteristic
        .on_notify(|data| {
            // Print out each received notification (UTF-8 string)
            ::log::info!("{}", core::str::from_utf8(data).unwrap());
        })
        .subscribe_notify(false)
        .await?;
    timer.delay(timer.tick_hz() * 100).await?;

    // Disconnect from the device gracefully
    client.disconnect()?;

    // Return success
    anyhow::Ok(())
  })
}
