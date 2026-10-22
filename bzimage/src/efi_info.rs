use crate::utils::write_u32;

pub const EFI_LOADER_SIGNATURE_OFFSET: usize = 0x1C0;
pub const EFI_SYSTAB_OFFSET: usize = 0x1C4;
pub const EFI_MEMDESC_SIZE_OFFSET: usize = 0x1C8;
pub const EFI_MEMDESC_VERSION_OFFSET: usize = 0x1CC;
pub const EFI_MEMMAP_OFFSET: usize = 0x1D0;
pub const EFI_MEMMAP_SIZE_OFFSET: usize = 0x1D4;
pub const EFI_SYSTAB_HI_OFFSET: usize = 0x1D8;
pub const EFI_MEMMAP_HI_OFFSET: usize = 0x1DC;

pub const EFI64_LOADER_SIGNATURE: &[u8; 4] = b"EL64";
pub fn fill_efi_info(
    params: &mut [u8],
    systab: u64,
    memmap: u64,
    memmap_size: u32,
    desc_size: u32,
    desc_version: u32,
) {
    params[EFI_LOADER_SIGNATURE_OFFSET..EFI_LOADER_SIGNATURE_OFFSET + 4]
        .copy_from_slice(EFI64_LOADER_SIGNATURE);
    write_u32(params, EFI_SYSTAB_OFFSET, systab as u32);
    write_u32(params, EFI_MEMDESC_SIZE_OFFSET, desc_size);
    write_u32(params, EFI_MEMDESC_VERSION_OFFSET, desc_version);
    write_u32(params, EFI_MEMMAP_OFFSET, memmap as u32);
    write_u32(params, EFI_MEMMAP_SIZE_OFFSET, memmap_size);
    write_u32(params, EFI_SYSTAB_HI_OFFSET, (systab >> 32) as u32);
    write_u32(params, EFI_MEMMAP_HI_OFFSET, (memmap >> 32) as u32);
}
