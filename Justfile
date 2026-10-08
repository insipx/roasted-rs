

flash:
    nix develop .#xtensa -c cargo build -p roasted-zippy --profile firmware --target xtensa-esp32s3-none-elf
    espflash flash \
    --chip esp32s3 \
    --port /dev/ttyACM1 \
    --partition-table crates/zippy/partitions.csv \
    --target-app-partition ota_0 \
    --monitor \
    ./target/xtensa-esp32s3-none-elf/firmware/roasted-zippy
build:
    cargo build --release -p roasted-daemon
