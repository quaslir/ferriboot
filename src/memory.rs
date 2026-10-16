use uefi::Error;

use uefi::{
    boot::{self, MemoryType},
    mem::memory_map::{MemoryMap, MemoryMapOwned},
    println,
};

pub fn print_memory_info() -> Result<(), Error> {
    let map = boot::memory_map(MemoryType::LOADER_DATA)?;
    let mut i: u64 = 0;
    let mut bytes_count: u64 = 0;
    for desc in map.entries() {
        if desc.ty == MemoryType::CONVENTIONAL {
            bytes_count += desc.page_count * 4096;
        }
        i += 1;
    }

    uefi::println!("Iterations: {}", i);
    uefi::println!("Bytes: {}", bytes_count);
    Ok(())
}
