#![no_std]

use core::u8;

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
    pub initrd_addr_max: u32,
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
const INITRD_ADDR_MAX_OFFSET: usize = 0x22C;

const EXPECTED_BOOT_FLAG: u16 = 0xAA55;
const EXPECTED_MAGIC: u32 = 0x53726448;
const SUPPORTED_VERSION: u16 = 0x020C;
const XLF_KERNEL_64: u16 = 1 << 0;

// FRAMEBUFFER CONSTANTS

const ORIG_VIDEO_IS_VGA_OFFSET: u8 = 0x0F;
const LFB_WIDTH_OFFSET: u16 = 0x12;
const LFB_HEIGHT_OFFSET: u16 = 0x14;
const LFB_DEPTH_OFFSET: u16 = 0x16;
const LFB_BASE_OFFSET: u32 = 0x18;
const LFB_LINELENGTH_OFFSET: u16 = 0x24;
const RED_SIZE_OFFSET: u8 = 0x26;
const RED_POS_OFFSET: u8 = 0x27;
const GREEN_SIZE_OFFSET: u8 = 0x28;
const GREEN_POS_OFFSET: u8 = 0x29;
const BLUE_SIZE_OFFSET: u8 = 0x2A;
const BLUE_POS_OFFSET: u8 = 0x2B;
const RSVD_SIZE_OFFSET: u8 = 0x2C;
const RSVD_POS_OFFSET: u8 = 0x2D;
const CAPABILITIES_OFFSET: u32 = 0x36;
const EXT_LFB_BASE_OFFSET: u32 = 0x3A;

const VIDEO_TYPE_EFI: u8 = 0x70;
const VIDEO_CAPABILITY_64BIT_BASE: u32 = 1 << 1;
const LFB_DEPTH_32: u32 = 32;
const CHANNEL_SIZE: u32 = 8;

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

fn write_u16(buf: &mut [u8], offset: usize, value: u16) {
    buf[offset..offset + 2].copy_from_slice(&u16::to_le_bytes(value));
}
fn write_u32(buf: &mut [u8], offset: usize, value: u32) {
    buf[offset..offset + 4].copy_from_slice(&u32::to_le_bytes(value));
}
fn write_u64(buf: &mut [u8], offset: usize, value: u64) {
    buf[offset..offset + 8].copy_from_slice(&u64::to_le_bytes(value));
}
pub fn fill_boot_params(
    params: &mut [u8],
    kernel: &[u8],
    initrd_addr: u64,
    initrd_size: usize,
    cmdline_addr: u64,
    rsdp_addr: u64,
) -> Result<(), ParseError> {
    if params.len() < 4096 {
        return Err(TooShort);
    }

    let header = parse_setup_header(kernel)?;
    let mut copy_limit: usize = 0x202;
    copy_limit += kernel[0x201] as usize;
    params[0x1F1..copy_limit].copy_from_slice(&kernel[0x1F1..copy_limit]);
    params[0x210] = 0xFF; // Unknown bootloader (custom value)
    params[0x211] |= 1 << 0;
    write_u32(params, 0x218, initrd_addr as u32);
    write_u32(params, 0x21C, initrd_size as u32);
    write_u32(params, 0x0C0, (initrd_addr >> 32) as u32);
    write_u32(params, 0x0C4, (initrd_size >> 32) as u32);
    write_u32(params, 0x228, cmdline_addr as u32);
    write_u32(params, 0x0C8, (cmdline_addr >> 32) as u32);
    write_u64(params, 0x270, rsdp_addr);
    Ok(())
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
    let initrd_addr_max = read_u32(buffer, INITRD_ADDR_MAX_OFFSET);
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
        initrd_addr_max,
    })
}

pub fn fill_screen_info(
    params: &mut [u8],
    base: u64,
    size: u64,
    w: u16,
    h: u16,
    stride: u16,
    is_bgr: bool,
) -> Result<(), ParseError> {
    params[ORIG_VIDEO_IS_VGA_OFFSET as usize] = VIDEO_TYPE_EFI;
    write_u16(params, LFB_WIDTH_OFFSET as usize, w);
    write_u16(params, LFB_HEIGHT_OFFSET as usize, h);
    write_u16(params, , value);
}

#[cfg(test)]

mod tests {
    use crate::{
        ParseError::{self, BadBootFlag},
        fill_boot_params, parse_setup_header, read_u16, read_u32, read_u64,
    };

    #[test]

    pub fn correct_image() {
        let buffer = include_bytes!("../tests/vmlinuz");
        let result = parse_setup_header(buffer);
        assert!(result.is_ok());
        let header = result.unwrap();

        assert_eq!(header.boot_flag, crate::EXPECTED_BOOT_FLAG);
        assert_eq!(header.header, crate::EXPECTED_MAGIC);
        assert!(header.version >= crate::SUPPORTED_VERSION);
        assert_eq!(header.version, 0x020F);
        assert_ne!(header.xloadflags & crate::XLF_KERNEL_64, 0);
        assert_ne!(header.setup_sects, 0);
        assert!(header.kernel_offset() < buffer.len());
        assert_ne!(header.init_size, 0);
        assert_ne!(header.kernel_alignment, 0);
        assert_eq!(header.initrd_addr_max, 0x7FFFFFFF);
    }
    #[test]
    pub fn too_short() {
        let buffer: [u8; 32] = [0; 32];
        let result = parse_setup_header(&buffer);
        assert!(matches!(result, Err(ParseError::TooShort)));
    }
    #[test]
    pub fn bad_boot_flag() {
        let mut buffer = include_bytes!("../tests/vmlinuz").to_vec();
        buffer[crate::BOOT_FLAG_OFFSET] = 0;
        let result = parse_setup_header(&buffer);
        assert!(matches!(result, Err(ParseError::BadBootFlag)));
    }

    #[test]

    pub fn bad_magic() {
        let mut buffer = include_bytes!("../tests/vmlinuz").to_vec();
        buffer[crate::HEADER_OFFSET] = 0;
        let result = parse_setup_header(&buffer);
        assert!(matches!(result, Err(ParseError::BadMagic)));
    }

    #[test]

    pub fn too_short2() {
        let mut buffer = include_bytes!("../tests/vmlinuz").to_vec();
        buffer.truncate(0x400);
        let result = parse_setup_header(&buffer);
        assert!(matches!(result, Err(ParseError::TooShort)));
    }

    #[test]
    pub fn unsupported_version() {
        let mut buffer = include_bytes!("../tests/vmlinuz").to_vec();
        buffer[crate::VERSION_OFFSET] = 0;
        let result = parse_setup_header(&buffer);
        assert!(matches!(result, Err(ParseError::UnsupportedVersion)));
    }
    #[test]
    pub fn no64bit_entry() {
        let mut buffer = include_bytes!("../tests/vmlinuz").to_vec();
        buffer[crate::XLOADFLAGS_OFFSET] = 0;
        let result = parse_setup_header(&buffer);
        assert!(matches!(result, Err(ParseError::No64BitEntry)));
    }

    #[test]

    pub fn fill_boot_params_test() {
        let mut arr: [u8; 4096] = [0u8; 4096];
        let buffer = include_bytes!("../tests/vmlinuz");
        let result = fill_boot_params(
            &mut arr,
            buffer,
            0x0000_0001_2340_0000,
            0x0000_0002_0649_CE6F,
            0x0000_0003_0001_0000,
            0x0000_0000_7FB7_E014,
        );

        assert_eq!(result, Ok(()));
        assert_eq!(read_u32(&arr, 0x202), 0x5372_6448);
        assert_eq!(read_u16(&arr, 0x206), 0x020F);
        assert_eq!(arr[0x210], 0xFF);
        assert_eq!(arr[0x211] & 1, 1);
        assert_eq!(read_u32(&arr, 0x218), 0x2340_0000);
        assert_eq!(read_u32(&arr, 0x0C0), 0x1);
        assert_eq!(read_u32(&arr, 0x21C), 0x0649_CE6F);
        assert_eq!(read_u32(&arr, 0x0C4), 0x2);
        assert_eq!(read_u32(&arr, 0x228), 0x0001_0000);
        assert_eq!(read_u32(&arr, 0x0C8), 0x3);
        assert_eq!(read_u64(&arr, 0x270), 0x7FB7_E014);
    }

    #[test]

    pub fn incorrect_boot_params() {
        let mut arr: [u8; 4096] = [0u8; 4096];
        let buffer: [u8; 65536] = [0u8; 65536];
        let result = fill_boot_params(
            &mut arr,
            &buffer,
            0x0000_0001_2340_0000,
            0x0000_0002_0649_CE6F,
            0x0000_0003_0001_0000,
            0x0000_0000_7FB7_E014,
        );

        assert_eq!(result, Err(BadBootFlag));
        assert_eq!(arr.iter().all(|v| v == &0u8), true);
    }
}
