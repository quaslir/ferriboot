#![no_std]

use crate::ParseError::{BadBootFlag, BadMagic, No64BitEntry, TooShort, UnsupportedVersion};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetupHeader {
    pub setup_sects: u8,
    pub boot_flag: u16,
    pub header: u32,
    pub version: u16,
    pub loadflags: u8,
    pub kernel_alignment: u32,
    pub relocatable_kernel: u8,
    pub xloadflags: u16,
    pub cmdline_size: u32,
    pub pref_address: u64,
    pub init_size: u32,
}
impl SetupHeader {
    pub fn kernel_offset(&self) -> usize {
        return (self.setup_sects as usize + 1) * 512;
    }
}

const SETUP_SECTS_OFFSET: usize = 0x1F1;
const BOOT_FLAG_OFFSET: usize = 0x1FE;
const HEADER_OFFSET: usize = 0x202;
const VERSION_OFFSET: usize = 0x206;
const LOAD_FLAGS_OFFSET: usize = 0x211;
const KERNEL_ALIGNMENT_OFFSET: usize = 0x230;
const RELOCATABLE_KERNEL_OFFSET: usize = 0x234;
const XLOADFLAGS_OFFSET: usize = 0x236;
const CMDLINE_SIZE_OFFSET: usize = 0x238;
const PREF_ADDRESS_OFFSET: usize = 0x258;
const INIT_SIZE_OFFSET: usize = 0x260;

const EXPECTED_BOOT_FLAG: u16 = 0xAA55;
const EXPECTED_MAGIC: u32 = 0x53726448;
const SUPPORTED_VERSION: u16 = 0x020C;
const XLF_KERNEL_64: u16 = 1 << 0;
fn read_u16(buf: &[u8], offset: usize) -> u16 {
    let value = buf[offset..offset + 2].try_into().unwrap();
    u16::from_le_bytes(value)
}

fn read_u32(buf: &[u8], offset: usize) -> u32 {
    let value = buf[offset..offset + 4].try_into().unwrap();
    u32::from_le_bytes(value)
}

fn read_u64(buf: &[u8], offset: usize) -> u64 {
    let value = buf[offset..offset + 8].try_into().unwrap();
    u64::from_le_bytes(value)
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    TooShort,
    BadBootFlag,
    BadMagic,
    UnsupportedVersion,
    No64BitEntry,
}

pub fn parse_setup_header(buffer: &[u8]) -> Result<SetupHeader, ParseError> {
    if buffer.len() < INIT_SIZE_OFFSET + 4 {
        return Err(TooShort);
    }
    let mut setup_sects = buffer[SETUP_SECTS_OFFSET];

    let boot_flag = read_u16(buffer, BOOT_FLAG_OFFSET);
    if boot_flag != EXPECTED_BOOT_FLAG {
        return Err(BadBootFlag);
    }
    let header = read_u32(buffer, HEADER_OFFSET);
    if header != EXPECTED_MAGIC {
        return Err(BadMagic);
    }
    if setup_sects == 0 {
        setup_sects = 4;
    }
    if (setup_sects as usize + 1) * 512 >= buffer.len() {
        return Err(TooShort);
    }
    let version = read_u16(buffer, VERSION_OFFSET);
    if version < SUPPORTED_VERSION {
        return Err(UnsupportedVersion);
    }
    let loadflags = buffer[LOAD_FLAGS_OFFSET];
    let kernel_alignment = read_u32(buffer, KERNEL_ALIGNMENT_OFFSET);
    let relocatable_kernel = buffer[RELOCATABLE_KERNEL_OFFSET];
    let xloadflags = read_u16(buffer, XLOADFLAGS_OFFSET);
    if (xloadflags & XLF_KERNEL_64) == 0 {
        return Err(No64BitEntry);
    }
    let cmdline_size = read_u32(buffer, CMDLINE_SIZE_OFFSET);
    let pref_address = read_u64(buffer, PREF_ADDRESS_OFFSET);
    let init_size = read_u32(buffer, INIT_SIZE_OFFSET);
    Ok(SetupHeader {
        setup_sects,
        boot_flag,
        header,
        version,
        loadflags,
        kernel_alignment,
        relocatable_kernel,
        xloadflags,
        cmdline_size,
        pref_address,
        init_size,
    })
}
