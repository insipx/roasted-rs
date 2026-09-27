# Zippy

Zippy uses esp-hal and Embassy through esp-rtos.

## Building

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

From the workspace root, specify both the package and target:

```sh
nix develop .#xtensa -c cargo build -p roasted-zippy --target xtensa-esp32s3-none-elf --profile firmware
```
