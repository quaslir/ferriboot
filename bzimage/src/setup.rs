use {
    crate::ParseError, crate::SetupHeader, crate::read_u16, crate::read_u32, crate::read_u64,
    crate::write_u32, crate::write_u64,
};

pub const BOOT_FLAG_OFFSET: usize = 0x1FE;
pub const HEADER_OFFSET: usize = 0x202;
pub const VERSION_OFFSET: usize = 0x206;
pub const LOAD_FLAGS_OFFSET: usize = 0x211;
pub const KERNEL_ALIGNMENT_OFFSET: usize = 0x230;
pub const RELOCATABLE_KERNEL_OFFSET: usize = 0x234;
pub const XLOADFLAGS_OFFSET: usize = 0x236;
pub const CMDLINE_SIZE_OFFSET: usize = 0x238;
pub const PREF_ADDRESS_OFFSET: usize = 0x258;
pub const INIT_SIZE_OFFSET: usize = 0x260;
pub const INITRD_ADDR_MAX_OFFSET: usize = 0x22C;

pub const EXPECTED_BOOT_FLAG: u16 = 0xAA55;
pub const EXPECTED_MAGIC: u32 = 0x53726448;
pub const SUPPORTED_VERSION: u16 = 0x020C;
pub const XLF_KERNEL_64: u16 = 1 << 0;
pub const SETUP_SECTS_OFFSET: usize = 0x1F1;
pub fn fill_boot_params(
    params: &mut [u8],
    kernel: &[u8],
    initrd_addr: u64,
    initrd_size: usize,
    cmdline_addr: u64,
    rsdp_addr: u64,
) -> Result<(), ParseError> {
    if params.len() < 4096 {
        return Err(ParseError::TooShort);
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
    write_u64(params, 0x070, rsdp_addr);
    Ok(())
}

pub fn parse_setup_header(buffer: &[u8]) -> Result<SetupHeader, ParseError> {
    if buffer.len() < INIT_SIZE_OFFSET + 4 {
        return Err(ParseError::TooShort);
    }
    let mut setup_sects = buffer[SETUP_SECTS_OFFSET];

    let boot_flag = read_u16(buffer, BOOT_FLAG_OFFSET);
    if boot_flag != EXPECTED_BOOT_FLAG {
        return Err(ParseError::BadBootFlag);
    }
    let header = read_u32(buffer, HEADER_OFFSET);
    if header != EXPECTED_MAGIC {
        return Err(ParseError::BadMagic);
    }
    if setup_sects == 0 {
        setup_sects = 4;
    }
    if (setup_sects as usize + 1) * 512 >= buffer.len() {
        return Err(ParseError::TooShort);
    }
    let version = read_u16(buffer, VERSION_OFFSET);
    if version < SUPPORTED_VERSION {
        return Err(ParseError::UnsupportedVersion);
    }
    let loadflags = buffer[LOAD_FLAGS_OFFSET];
    let initrd_addr_max = read_u32(buffer, INITRD_ADDR_MAX_OFFSET);
    let kernel_alignment = read_u32(buffer, KERNEL_ALIGNMENT_OFFSET);
    let relocatable_kernel = buffer[RELOCATABLE_KERNEL_OFFSET];
    let xloadflags = read_u16(buffer, XLOADFLAGS_OFFSET);
    if (xloadflags & XLF_KERNEL_64) == 0 {
        return Err(ParseError::No64BitEntry);
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
