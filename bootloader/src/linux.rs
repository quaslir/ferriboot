use crate::fs;
use crate::graphics;
use uefi::CString16;
use uefi::boot::{self, MemoryType};
use uefi::table::cfg::ConfigTableEntry;
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
        uefi::println!(
            "ferriboot: cmdline too long ({} bytes, max {})",
            cmd.len(),
            max_len
        );
        return Err(uefi::Error::from(uefi::Status::LOAD_ERROR));
    }

    let page = boot::allocate_pages(
        boot::AllocateType::MaxAddress(0xFFFF_FFFF),
        MemoryType::LOADER_DATA,
        1,
    )
    .map_err(|e| {
        uefi::println!(
            "ferriboot: cannot allocate memory for cmdline ({:?})",
            e.status()
        );
        e
    })?
    .as_ptr();

    unsafe {
        core::ptr::copy_nonoverlapping(cmd.as_bytes().as_ptr(), page, cmd.len());
        core::ptr::write(page.add(cmd.len()), 0);
    }

    Ok(page as u64)
}

const ADDRESS_ALIGNMENT_MASK: u64 = 0x1FFFFF;
pub fn load_linux(entry: &config::Entry) -> uefi::Result<LinuxImage> {
    if entry.protocol != config::Protocol::Linux {
        uefi::println!(
            "ferriboot: entry '{}' uses an unsupported protocol",
            entry.name
        );
        return Err(uefi::Error::from(uefi::Status::UNSUPPORTED));
    }

    let kernel_path = CString16::try_from(entry.kernel.as_str()).map_err(|e| {
        uefi::println!("ferriboot: invalid kernel path '{}'", entry.kernel);
        uefi::Error::from(uefi::Status::LOAD_ERROR)
    })?;

    let mut vmlinuz_linux_file = fs::open_file(&kernel_path).map_err(|e| {
        uefi::println!(
            "ferriboot: cannot open kernel '{}' ({:?})",
            entry.kernel,
            e.status()
        );
        e
    })?;
    let vmlinuz_linux_size = fs::file_size(&mut vmlinuz_linux_file)?;

    let mut vmlinuz_linux = alloc::vec![0u8; vmlinuz_linux_size];

    fs::read_file(&mut vmlinuz_linux_file, &mut vmlinuz_linux)?;
    let header = bzimage::setup::parse_setup_header(&vmlinuz_linux)
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
    let (initrd_addr, initrd_size) = match entry.initrd.as_deref() {
        Some(initrd) => {
            let mut initramfs_linux_file = fs::open_file(
                &CString16::try_from(initrd)
                    .map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?,
            )?;
            let initramfs_linux_size = fs::file_size(&mut initramfs_linux_file)?;
            let initrd_ptr = boot::allocate_pages(
                boot::AllocateType::MaxAddress(header.initrd_addr_max as u64),
                MemoryType::LOADER_DATA,
                initramfs_linux_size.div_ceil(4096),
            )?
            .as_ptr();
            let initrd_addr = initrd_ptr as u64;
            let initramfs =
                unsafe { core::slice::from_raw_parts_mut(initrd_ptr, initramfs_linux_size) };

            if initrd_addr as usize + initramfs_linux_size - 1 > header.initrd_addr_max as usize {
                return Err(uefi::Error::from(uefi::Status::LOAD_ERROR));
            }

            fs::read_file(&mut initramfs_linux_file, initramfs)?;
            (initrd_addr, initramfs_linux_size)
        }
        None => (0, 0),
    };

    let boot_params_page =
        boot::allocate_pages(boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)?.as_ptr();

    unsafe { core::ptr::write_bytes(boot_params_page, 0, 4096) };
    let span = unsafe { core::slice::from_raw_parts_mut(boot_params_page, 4096) };

    let cmdline_addr = load_cmdline(
        &entry.cmdline.as_deref().unwrap_or(""),
        header.cmdline_size as usize,
    )?;
    let rsdp_addr = find_rsdp()?;
    let boot_params_addr = boot_params_page as u64;
    bzimage::setup::fill_boot_params(
        span,
        &vmlinuz_linux,
        initrd_addr,
        initrd_size,
        cmdline_addr,
        rsdp_addr,
    )
    .map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?;

    let graphics_info = graphics::graphics_info();

    match graphics_info {
        Ok(graphics) => {
            bzimage::screen_info::fill_screen_info(
                span,
                graphics.base,
                graphics.size as u64,
                graphics.width as u16,
                graphics.height as u16,
                graphics.stride as u16,
                graphics.pixel_format == boot_info::PixelFormat::Bgr,
            )
            .map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?;
        }
        Err(_) => {}
    }

    Ok(LinuxImage {
        kernel_base,
        initrd_addr,
        initrd_size: initrd_size,
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

pub unsafe fn boot_linux(kernel_base: u64, boot_params: u64) -> ! {
    let jump_addr = kernel_base + 0x200;
    unsafe {
        core::arch::asm!("cli", "jmp {}", in(reg) jump_addr, in("rsi") boot_params, options(noreturn));
    }
}
