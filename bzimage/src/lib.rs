#![no_std]

use core::u8;
pub mod e820;
pub mod efi_info;
pub mod screen_info;
pub mod setup;
mod utils;
use {e820::*, setup::*, utils::*};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    TooShort,
    BadBootFlag,
    BadMagic,
    UnsupportedVersion,
    No64BitEntry,
    FramebufferError,
    TooManyE820Entries,
}
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
        assert_eq!(read_u64(&arr, 0x070), 0x7FB7_E014);
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
