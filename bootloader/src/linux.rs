use core::slice::from_raw_parts;

use crate::fs;
use uefi::table::cfg::ConfigTableEntry;
use uefi::{
    boot::{self, MemoryType},
    cstr16,
};
pub struct LinuxImage {
    pub kernel_base: u64,
    pub initrd_addr: u64,
    pub initrd_size: usize,
    pub header: bzimage::SetupHeader,
    pub cmdline_addr: u64,
    pub boot_params_addr: u64,
}

fn load_cmdline(cmd: &str, max_len: usize) -> uefi::Result<u64> {
    if cmd.len() > max_len {
        return Err(uefi::Error::from(uefi::Status::LOAD_ERROR));
    }

    let page = boot::allocate_pages(
        boot::AllocateType::MaxAddress(0xFFFF_FFFF),
        MemoryType::LOADER_DATA,
        1,
    )?
    .as_ptr();

    unsafe {
        core::ptr::copy_nonoverlapping(cmd.as_bytes().as_ptr(), page, cmd.len());
        core::ptr::write(page.add(cmd.len()), 0);
    }

    Ok(page as u64)
}

const ADDRESS_ALIGNMENT_MASK: u64 = 0x1FFFFF;
const CMDLINE: &str = "console=ttyS0 earlyprintk=serial,ttyS0,115200 ignore_loglevel";
pub fn load_linux() -> uefi::Result<LinuxImage> {
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

    let boot_params_page =
        boot::allocate_pages(boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)?.as_ptr();

    unsafe { core::ptr::write_bytes(boot_params_page, 0, 4096) };
    let span = unsafe { core::slice::from_raw_parts_mut(boot_params_page, 4096) };
    let cmdline_addr = load_cmdline(CMDLINE, header.cmdline_size as usize)?;
    let rsdp_addr = find_rsdp()?;
    let boot_params_addr = boot_params_page as u64;
    bzimage::fill_boot_params(
        span,
        &vmlinuz_linux,
        initrd_addr,
        initrd_size,
        cmdline_addr,
        rsdp_addr,
    )
    .map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?;
    drop(initramfs_linux);

    Ok(LinuxImage {
        kernel_base,
        initrd_addr,
        initrd_size,
        header,
        cmdline_addr,
        boot_params_addr,
    })
}

pub fn find_rsdp() -> uefi::Result<u64> {
    let rsdp = uefi::system::with_config_table(|table| {
        table
            .iter()
            .find(|entry| entry.guid == ConfigTableEntry::ACPI2_GUID)
            .map(|page| page.address as u64)
    })
    .ok_or(uefi::Error::from(uefi::Status::NOT_FOUND))?;
    Ok(rsdp)
}
