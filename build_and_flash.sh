cd esp_rust
git pull
cd ..
cp esp_rust/Cargo.toml Cargo.toml
cp sdkconfig.defaults sdkconfig.defaults
cp esp_rust/src/main1/main.rs src/main.rs
cargo build --release --target xtensa-esp32-espidf
espflash flash target/xtensa-esp32-espidf/release/hello_world --chip esp32 --baud 460800 --port /dev/ttyUSB0
cp esp_rust/src/main2/main.rs src/main.rs
cargo build --release --target xtensa-esp32-espidf
espflash flash target/xtensa-esp32-espidf/release/hello_world --chip esp32 --baud 460800 --port /dev/ttyUSB1
cp esp_rust/src/main3/main.rs src/main.rs
cargo build --release --target xtensa-esp32-espidf
espflash flash target/xtensa-esp32-espidf/release/hello_world --chip esp32 --baud 460800 --port /dev/ttyUSB2