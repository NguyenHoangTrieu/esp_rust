use bstr::ByteSlice; // For byte string operations
use esp32_nimble::{uuid128, BLEDevice, BLEScan}; // BLE support using NimBLE
use esp_idf_svc::hal::{ // ESP-IDF Hardware Abstraction Layer
    prelude::Peripherals, // Access to board peripherals
    task::block_on,        // Used to block on async execution
    timer::{TimerConfig, TimerDriver}, // Timer functionality
};

fn main() -> anyhow::Result<()> {
    // Required for ESP-IDF systems to patch system calls
    esp_idf_svc::sys::link_patches();

    // Initialize the logger to output logs to serial
    esp_idf_svc::log::EspLogger::initialize_default();

    // Get access to board peripherals (timers, pins, etc.)
    let peripherals = Peripherals::take()?;

    // Initialize a timer using TIMER0 with default configuration
    let mut timer = TimerDriver::new(peripherals.timer00, &TimerConfig::new())?;

    // Run async code using block_on
    block_on(async {
        // Get the global BLEDevice singleton instance
        let ble_device = BLEDevice::take();

        // Create a new BLE scanner
        let mut ble_scan = BLEScan::new();

        // Start scanning for BLE devices name contains "ESP32"
        // - active_scan: request scan response
        // - interval and window define scan parameters
        // - 10000 is timeout (ms)
        // - closure returns Some(device) if name contains "ESP32"
        let device = ble_scan
            .active_scan(true)
            .interval(100)
            .window(99)
            .start(ble_device, 10000, |device, data| {
                if let Some(name) = data.name() {
                    if name.contains_str("ESP32") {
                        return Some(*device);
                    }
                }
                None
            })
            .await?;

        // If a matching device was found
        if let Some(device) = device {
            // Create a BLE client to connect to the server
            let mut client = ble_device.new_client();

            // Register connection callback to update BLE connection parameters
            client.on_connect(|client| {
                client.update_conn_params(120, 120, 0, 60).unwrap();
            });

            // Connect to the BLE device using its address
            client.connect(&device.addr()).await?;

            // Discover the desired service using its UUID
            let service = client
                .get_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"))
                .await?;

            // Discover a characteristic to read from using its UUID
            let uuid = uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93");
            let characteristic = service.get_characteristic(uuid).await?;

            // Read the value from the characteristic and print it
            let value = characteristic.read_value().await?;
            ::log::info!(
                "{} value: {}",
                characteristic,
                core::str::from_utf8(&value)? // Convert from UTF-8 bytes to string
            );

            // Discover another characteristic that supports notifications
            let uuid = uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295");
            let characteristic = service.get_characteristic(uuid).await?;

            // Check if characteristic supports notification
            if !characteristic.can_notify() {
                ::log::error!("characteristic can't notify: {}", characteristic);
                return anyhow::Ok(()); // Exit gracefully
            }

            // Subscribe to notifications from the characteristic
            ::log::info!("subscribe to {}", characteristic);
            characteristic
                .on_notify(|data| {
                    // Print out each received notification (UTF-8 string)
                    ::log::info!("{}", core::str::from_utf8(data).unwrap());
                })
                .subscribe_notify(false)
                .await?;

            // Wait for 10 seconds (based on timer ticks)
            timer.delay(timer.tick_hz() * 10).await?;

            // Disconnect the client after done
            client.disconnect()?;
        }

        // Return success
        anyhow::Ok(())
    })
}
