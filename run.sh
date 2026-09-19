#!/bin/sh
set -e

cargo build --target x86_64-unknown-uefi

mkdir -p esp/EFI/BOOT
cp target/x86_64-unknown-uefi/debug/bootloader.efi esp/EFI/BOOT/BOOTX64.EFI

qemu-system-x86_64 \
  -drive if=pflash,format=raw,readonly=on,file="$(brew --prefix qemu)/share/qemu/edk2-x86_64-code.fd" \
  -drive format=raw,file=fat:rw:esp \
  -serial stdio \
  -m 512M
