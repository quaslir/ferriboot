use bzimage::e820::E820Entry;

use uefi::mem::memory_map::{MemoryMapMut, MemoryMapOwned};
use uefi::{boot::MemoryType, mem::memory_map::MemoryMap};
pub fn to_e820_type(typ: MemoryType) -> u32 {
    match typ {
        MemoryType::CONVENTIONAL
        | MemoryType::LOADER_CODE
        | MemoryType::LOADER_DATA
        | MemoryType::BOOT_SERVICES_CODE
        | MemoryType::BOOT_SERVICES_DATA => bzimage::e820::E820_RAM,
        MemoryType::ACPI_RECLAIM => bzimage::e820::E820_ACPI,
        MemoryType::ACPI_NON_VOLATILE => bzimage::e820::E820_NVS,
        MemoryType::UNUSABLE => bzimage::e820::E820_UNUSABLE,
        MemoryType::PERSISTENT_MEMORY => bzimage::e820::E820_PMEM,
        _ => bzimage::e820::E820_RESERVED,
    }
}

pub fn build_e820(map: &mut MemoryMapOwned, out: &mut [E820Entry; 128]) -> uefi::Result<usize> {
    map.sort();
    let mut count = 0;
    for desc in map.entries() {
        let addr = desc.phys_start;
        let size = desc.page_count * 4096;
        let typ = to_e820_type(desc.ty);

        if count > 0
            && out[count - 1].typ == typ
            && out[count - 1].addr + out[count - 1].size == addr
        {
            out[count - 1].size += size;
            continue;
        }

        if count == bzimage::e820::E820_MAX_ENTRIES {
            return Err(uefi::Error::from(uefi::Status::BUFFER_TOO_SMALL));
        }
        out[count] = E820Entry { addr, size, typ };
        count += 1;
    }

    Ok(count)
}
