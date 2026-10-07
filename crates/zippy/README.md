# Zippy

Zippy uses esp-hal and Embassy through esp-rtos.

## Building

The nix shell defined in the project as `.#xtensa` provides the tools required
to build for the xtensa-esp32s3 target and emulate esp32s3 with QEMU. It is
assumed the commands are run from this directory. If run from the workspace
root, prefix a comand with `nix develop .#xtensa -c`. For instance,
`nix develop .#xtensa -c cargo run --profile firmware`

```sh
# without direnv, enter the nix-shell using `nix develop`.
# direnv will auto load the shell after `direnv allow`
# The nix shell sets up the `xtensa-esp32s3-none-elf` triple needed
nix develop .#xtensa
cd crates/zippy
cargo build --profile firmware
# cargo run detects the connected board and flashes the firmware.
cargo run --profile firmware
```

## Building Flash Image for ESP32-S3 QEMU

> ![NOTE] Does not work yet. QEMU image does not boot. need to investigate

First, compile roasted-zippy for the xtensa target

```bash
cargo build -p roasted-zippy \
   --target xtensa-esp32s3-none-elf \
   --profile firmware
```

Use the `espflash` tool to setup the compiled ELF binary as a flash image

```bash
espflash save-image \
  --chip esp32s3 \
  --flash-size 16mb \
  --merge \
  target/xtensa-esp32s3-none-elf/firmware/roasted-zippy \
  ./zippy-flash.bin
```

the `xtensa` shell provides espressifs QEMU fork compatible with esp32-s3. run
it with the flash image

```bash
qemu-system-xtensa \
    -machine esp32s3 \
    -nographic \
    -drive file=target/zippy-flash.bin,if=mtd,format=raw
```
