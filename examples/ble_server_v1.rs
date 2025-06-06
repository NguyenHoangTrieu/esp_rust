// Import BLE helper types and macros from the esp32_nimble crate
use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties}; 
use std::format;

fn main() -> anyhow::Result<()> {
    // Link necessary patches for ESP-IDF BLE runtime
    esp_idf_svc::sys::link_patches();

    // Take ownership of the BLE device (singleton)
    let ble_device = BLEDevice::take();
    // Get a handle to the advertising subsystem
    let ble_advertising = ble_device.get_advertising();

    // Get a handle to the GATT server
    let server = ble_device.get_server();

    // Define behavior when a client connects
    server.on_connect(|server, desc| {
        println!("Client connected: {:?}", desc);

        // Update connection parameters: min/max interval, latency, supervision timeout
        server
            .update_conn_params(desc.conn_handle(), 24, 48, 0, 60)
            .unwrap();

        // Restart advertising if multiple connections are supported and available
        if server.connected_count() < (esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _) {
            println!("Multi-connect support: start advertising");
            ble_advertising.lock().start().unwrap();
        }
    });

    // Define behavior when a client disconnects
    server.on_disconnect(|_desc, reason| {
        println!("Client disconnected ({:?})", reason);
    });

    // Create a new BLE service with a 128-bit UUID
    let service = server.create_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"));

    // ---------- STATIC CHARACTERISTIC ----------
    // Create a read-only characteristic under the service
    let static_characteristic = service.lock().create_characteristic(
        uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93"),
        NimbleProperties::READ,
    );
    // Set a static string value to be returned on read
    static_characteristic
        .lock()
        .set_value("Hello, world!".as_bytes());

    // ---------- NOTIFYING CHARACTERISTIC ----------
    // Create a characteristic that can be read and notifies changes
    let notifying_characteristic = service.lock().create_characteristic(
        uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295"),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );
    // Set initial value
    notifying_characteristic.lock().set_value(b"Initial value.");

    // ---------- WRITABLE CHARACTERISTIC ----------
    // Create a characteristic that supports both reading and writing
    let writable_characteristic = service.lock().create_characteristic(
        uuid128!("3c9a3f00-8ed3-4bdf-8a39-a01bebede295"),
        NimbleProperties::READ | NimbleProperties::WRITE,
    );
    writable_characteristic
        .lock()
        .on_read(move |_, _| {
            // Log when characteristic is read
            println!("Read from writable characteristic.");
        })
        .on_write(|args| {
            // Log when new data is written to the characteristic
            println!(
                "Wrote to writable characteristic: {:?} -> {:?}",
                args.current_data(),
                args.recv_data()
            );
        });

    // Configure advertising data: set device name and advertised service UUID
    ble_advertising.lock().set_data(
        BLEAdvertisementData::new()
            .name("ESP32-GATT-Server")
            .add_service_uuid(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa")),
    )?;

    // Start advertising
    ble_advertising.lock().start()?;

    // Print local attribute table to serial console
    server.ble_gatts_show_local();

    // ---------- MAIN LOOP ----------
    // Send a notification every second with an incrementing counter
    let mut counter = 0;
    loop {
        // Wait 1 second
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);

        // Update value of notifying characteristic and send notification
        notifying_characteristic
            .lock()
            .set_value(format!("Counter: {counter}").as_bytes())
            .notify();

        // Increment counter
        counter += 1;
    }
}
