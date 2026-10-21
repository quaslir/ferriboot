#!/bin/sh
set -e
cargo build --release --target x86_64-unknown-uefi -p bootloader
sudo cp target/x86_64-unknown-uefi/release/bootloader.efi /boot/EFI/ferriboot/ferriboot.efi
echo "ferriboot installed"
