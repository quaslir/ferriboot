#![no_std]
use core::slice::from_raw_parts;

use uefi::{
    boot::{self, MemoryType},
    cstr16,
};

use crate::fs;
const ADDRESS_ALIGNMENT_MASK: u64 = 0x1FFFFF;
pub fn load_linux() -> uefi::Result<()> {
    let vmlinuz_linux = fs::read_file(cstr16!("vmlinuz-linux"))?;
    let initramfs_linux = fs::read_file(cstr16!("initramfs-linux.img"))?;
    let header = bzimage::parse_setup_header(&vmlinuz_linux)
        .map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?;

    let allocated = boot::allocate_pages(
        boot::AllocateType::Address(header.pref_address),
        MemoryType::LOADER_DATA,
        header.init_size.div_ceil(4096) as usize,
    );

    let kernel_base = match allocated {
        Ok(ptr) => ptr.as_ptr() as u64,
        Err(_) => {
            if header.relocatable_kernel == 0 {
                return Err(uefi::Error::from(uefi::Status::LOAD_ERROR));
            }
            let page_count = (header.init_size + header.kernel_alignment).div_ceil(4096);

            let ptr = boot::allocate_pages(
                boot::AllocateType::AnyPages,
                MemoryType::LOADER_DATA,
                page_count as usize,
            )?
            .as_ptr() as u64;

            (ptr + ADDRESS_ALIGNMENT_MASK) & (!ADDRESS_ALIGNMENT_MASK)
        }
    };

    if kernel_base % header.kernel_alignment as u64 != 0 {
        return Err(uefi::Error::from(uefi::Status::LOAD_ERROR));
    }

    let dst = kernel_base as *mut u8;
    unsafe {
        let src = vmlinuz_linux.as_ptr().add(header.kernel_offset());
        core::ptr::copy_nonoverlapping(src, dst, vmlinuz_linux.len() - header.kernel_offset());
    }

    let initrd_ptr = boot::allocate_pages(
        boot::AllocateType::MaxAddress(header.initrd_addr_max as u64),
        MemoryType::LOADER_DATA,
        initramfs_linux.len().div_ceil(4096),
    )?
    .as_ptr();
    unsafe {
        core::ptr::copy_nonoverlapping(initramfs_linux.as_ptr(), initrd_ptr, initramfs_linux.len());
    }
    let initrd_addr = initrd_ptr as u64;
    let initrd_size = initramfs_linux.len();
    let raw = unsafe { from_raw_parts(initrd_ptr, initrd_size) };
    if initrd_addr as usize + initrd_size - 1 > header.initrd_addr_max as usize {
        return Err(uefi::Error::from(uefi::Status::LOAD_ERROR));
    }
    if raw != initramfs_linux {
        return Err(uefi::Error::from(uefi::Status::LOAD_ERROR));
    }
    drop(initramfs_linux);
    Ok(())
}
