use esp32_nimble::{
  enums::*, utilities::BleUuid, BLEAdvertisementData, BLEDevice, NimbleProperties,
};

fn main() -> anyhow::Result<()> {
  esp_idf_svc::sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

   // Acquire a singleton instance of the BLE device
  let device = BLEDevice::take();
  let ble_advertising = device.get_advertising();

  // Configure BLE security settings
  device
    .security()
    .set_auth(AuthReq::all())             // Require all security features: MITM, bonding, encryption
    .set_passkey(123456)                  // Set fixed passkey for pairing
    .set_io_cap(SecurityIOCap::DisplayOnly) // Set device to have only a display (no input)
    .resolve_rpa();                       // Enable resolving of Resolvable Private Addresses (RPA)

  // Create a GATT server
  let server = device.get_server();

  // Callback: When a client connects
  server.on_connect(|server, desc| {
    ::log::info!("Client connected: {:?}", desc);

    // If max connections not reached, continue advertising
    if server.connected_count() < (esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _) {
      ::log::info!("Multi-connect support: start advertising");
      ble_advertising.lock().start().unwrap();
    }
  });

  // Callback: When a client disconnects
  server.on_disconnect(|_desc, reason| {
    ::log::info!("Client disconnected ({:?})", reason);
  });

  // Callback: Authentication result logging
  server.on_authentication_complete(|_, desc, result| {
    ::log::info!("AuthenticationComplete({:?}): {:?}", result, desc);
  });

  // Create a GATT service with UUID 0xABCD
  let service = server.create_service(BleUuid::Uuid16(0xABCD));

  // Add a non-secure characteristic (anyone can read it)
  let non_secure_characteristic = service
    .lock()
    .create_characteristic(BleUuid::Uuid16(0x1234), NimbleProperties::READ);
  non_secure_characteristic
    .lock()
    .set_value("non_secure_characteristic".as_bytes());

  // Add a secure characteristic (requires encryption and authentication)
  let secure_characteristic = service.lock().create_characteristic(
    BleUuid::Uuid16(0x1235),
    NimbleProperties::READ | NimbleProperties::READ_ENC | NimbleProperties::READ_AUTHEN,
  );
  secure_characteristic
    .lock()
    .set_value("secure_characteristic".as_bytes());

  // Set up BLE advertising with a name and advertised service UUID
  ble_advertising.lock().set_data(
    BLEAdvertisementData::new()
      .name("ESP32-GATT-Server") // Name shown during scanning
      .add_service_uuid(BleUuid::Uuid16(0xABCD)), // Advertise the service
  )?;

  // Start BLE advertising
  ble_advertising.lock().start()?;

  // Log the list of bonded client addresses
  ::log::info!("bonded_addresses: {:?}", device.bonded_addresses());

  // Keep the program running (simulate a running BLE server)
  loop {
    esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
  }
}