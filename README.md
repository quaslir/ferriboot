# ferriboot
A minimal UEFI bootloader written in Rust, built from scratch as a learning project. It reads a kernel ELF from the EFI System Partition, loads its segments into memory, and hands control to the kernel along with boot info (framebuffer, memory map).
