# ferriboot

A minimal UEFI bootloader written in Rust, built from scratch as a learning
project. It reads a kernel ELF from the EFI System Partition, loads its
segments into memory, and hands control to the kernel along with boot info
(framebuffer, memory map).

## Status
- [x] UEFI hello world
- [ ] Firmware info (UEFI version, screen resolution, memory map)
- [ ] Read kernel file from ESP
- [ ] ELF parser
- [ ] Load segments and jump to kernel
- [ ] ExitBootServices + BootInfo

## Build & run
Requires Rust (with `x86_64-unknown-uefi` target) and QEMU.

    ./run.sh
