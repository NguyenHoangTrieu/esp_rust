use esp32_nimble::{enums::*, utilities::BleUuid, BLEDevice, BLEScan};
use log::*;
use esp_idf_svc::hal::{
  prelude::Peripherals,
  task::block_on,
  timer::{TimerConfig, TimerDriver},
};
const SERVICE_UUID: BleUuid = BleUuid::Uuid16(0xABCD);

fn main() -> anyhow::Result<()> {
  esp_idf_svc::sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take()?;

  let mut timer = TimerDriver::new(peripherals.timer00, &TimerConfig::new())?;

  block_on(async {
  let ble_device = BLEDevice::take();

  ble_device.set_power(PowerType::Default, PowerLevel::P9)?;

  ble_device
    .security()
    .set_auth(AuthReq::all())
    .set_io_cap(SecurityIOCap::KeyboardOnly);

  let mut ble_scan = BLEScan::new();

  let device = ble_scan
    .active_scan(true)
    .interval(100)
    .window(99)
    .start(ble_device, 10000, |device, data| {
    if data.is_advertising_service(&SERVICE_UUID) {
      return Some(*device);
    }
    None
    })
    .await?;

  let Some(device) = device else {
    ::log::warn!("device not found");
    return anyhow::Ok(());
  };

  info!("Advertised Device: {:?}", device);
  let mut client = ble_device.new_client();
  client.connect(&device.addr()).await?;
  client.on_passkey_request(|| 123456);
  client.secure_connection().await?;
  let service = client.get_service(SERVICE_UUID).await?;
  let non_secure_characteristic = service.get_characteristic(BleUuid::Uuid16(0x1234)).await?;
  let value = non_secure_characteristic.read_value().await?;

  ::log::info!(
    "{:?} value: {}",
    non_secure_characteristic.uuid(),
    core::str::from_utf8(&value)?
  );

  let secure_characteristic = service.get_characteristic(BleUuid::Uuid16(0x1235)).await?;

  let value = secure_characteristic.read_value().await?;

  ::log::info!(
    "{:?} value: {}",
    secure_characteristic.uuid(),
    core::str::from_utf8(&value)?
  );

  let notification_characteristic = service.get_characteristic(BleUuid::Uuid16(0x1236)).await?;
  if !notification_characteristic.can_notify() {
    ::log::error!("characteristic can't notify: {}", notification_characteristic);
    return anyhow::Ok(());
  }
  ::log::info!("subscribe to {}", notification_characteristic);
  notification_characteristic
    .on_notify(|data| {
      ::log::info!("{}", core::str::from_utf8(data).unwrap());
    })
    .subscribe_notify(false)
    .await?;
  timer.delay(timer.tick_hz() * 100).await?;
  client.disconnect()?;
  anyhow::Ok(())
  })
}
