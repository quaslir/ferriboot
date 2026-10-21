use crate::ParseError;
use {crate::write_u32, crate::write_u64};

pub const E820_ENTRIES_OFFSET: usize = 0x1E8;
pub const E820_TABLE_OFFSET: usize = 0x2D0;
pub const E820_ENTRY_SIZE: usize = 20;
pub const E820_MAX_ENTRIES: usize = 128;
pub const E820_RAM: u32 = 1;
pub const E820_RESERVED: u32 = 2;
pub const E820_ACPI: u32 = 3;
pub const E820_NVS: u32 = 4;
pub const E820_UNUSABLE: u32 = 5;
pub const E820_PMEM: u32 = 7;

#[derive(Debug, Clone, Copy, Default)]
pub struct E820Entry {
    pub addr: u64,
    pub size: u64,
    pub typ: u32,
}

pub fn fill_e820(params: &mut [u8], entries: &[E820Entry]) -> Result<(), ParseError> {
    if entries.len() > E820_MAX_ENTRIES {
        return Err(ParseError::TooManyE820Entries);
    }

    params[E820_ENTRIES_OFFSET] = entries.len() as u8;

    for (i, entry) in entries.iter().enumerate() {
        let offset = E820_TABLE_OFFSET + i * E820_ENTRY_SIZE;
        write_u64(params, offset, entry.addr);
        write_u64(params, offset + 8, entry.size);
        write_u32(params, offset + 16, entry.typ);
    }

    Ok(())
}
