use uefi::boot::{self, AllocateType, MemoryType};

pub fn load_kernel(elf_bytes: &[u8]) -> uefi::Result<u64> {
    let header =
        elf::parse_header(elf_bytes).map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?;
    for i in 0..header.e_phnum {
        let offset = header.e_phoff as usize + i as usize * header.e_phentsize as usize;
        let program_header =
            elf::parse_program_header(&elf_bytes[offset..offset + header.e_phentsize as usize])
                .map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?;
        if program_header.p_type != 1 {
            continue;
        }
        let pages = (program_header.p_memsz + 4095) / 4096;
        let result = boot::allocate_pages(
            AllocateType::Address(program_header.p_paddr),
            MemoryType::LOADER_DATA,
            pages as usize,
        )?;

        let dst = result.as_ptr();

        unsafe {
            core::ptr::copy_nonoverlapping(
                elf_bytes[program_header.p_offset as usize
                    ..(program_header.p_offset + program_header.p_filesz) as usize]
                    .as_ptr(),
                dst,
                program_header.p_filesz as usize,
            );
            let bss_start = dst.add(program_header.p_filesz as usize);
            core::ptr::write_bytes(
                bss_start,
                0,
                (program_header.p_memsz - program_header.p_filesz) as usize,
            );
        }
    }

    Ok(header.e_entry)
}
