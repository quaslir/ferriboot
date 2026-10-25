# ferriboot

A UEFI bootloader for Linux written in Rust. It loads the kernel and initrd
from the EFI System Partition and starts Linux through the x86 boot protocol
(not the EFI stub), with a small boot menu.

`ferriboot` (the CLI) installs the bootloader, registers it in the firmware,
generates the config and keeps it up to date after kernel updates.

## Status

Tested on Arch Linux (real hardware) and Fedora (QEMU/OVMF) and Ubuntu.

Limitations:
- x86_64 UEFI only
- no Secure Boot support

## Building

    rustup target add x86_64-unknown-uefi
    cargo build --release -p bootloader --target x86_64-unknown-uefi
    cargo build --release -p ferriboot-cli --target x86_64-unknown-linux-gnu

## Installing

    sudo ./target/x86_64-unknown-linux-gnu/release/ferriboot install \
        --efi target/x86_64-unknown-uefi/release/bootloader.efi

This copies the bootloader to `EFI/ferriboot/` on the ESP, writes
`EFI/ferriboot/ferriboot.conf`, adds a firmware boot entry, installs
`/usr/local/bin/ferriboot` and a kernel update hook
(pacman, kernel-install or /etc/kernel/postinst.d).

Keep another bootloader installed until you have checked that ferriboot
boots your system.

## Usage

    sudo ferriboot mkconfig            # regenerate the config
    sudo ferriboot mkconfig --dry-run  # show what would change
    sudo ferriboot uninstall           # remove everything

## License

MIT
