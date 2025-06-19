cp esp_rust/src/main1/main.rs src/main.rs
cp esp_rust/simulator/buffer.rs src/simulator/buffer.rs
cargo build --release --target xtensa-esp32-espidf
espflash flash target/xtensa-esp32-espidf/release/hello_world --chip esp32 --baud 460800 --port /dev/ttyUSB0
cp esp_rust/src/main2/main.rs src/main.rs
cargo build --release --target xtensa-esp32-espidf
espflash flash target/xtensa-esp32-espidf/release/hello_world --chip esp32 --baud 460800 --port /dev/ttyUSB1
cp esp_rust/src/main3/main.rs src/main.rs
cargo build --release --target xtensa-esp32-espidf
espflash flash target/xtensa-esp32-espidf/release/hello_world --chip esp32 --baud 460800 --port /dev/ttyUSB2
espflash monitor --chip esp32 --port /dev/ttyUSB2 --baud 19200