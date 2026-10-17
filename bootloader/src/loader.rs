use elf::ElfHeader;
use uefi::boot::{self, AllocateType, MemoryType};
const PAGE_SIZE: u64 = 4096;
const PAGE_ROUND_MASK: u64 = 0xFFF;
const PT_LOAD: u32 = 1;
fn program_header_at(
    elf_bytes: &[u8],
    header: &ElfHeader,
    i: u16,
) -> uefi::Result<elf::ProgramHeader> {
    let offset = header.e_phoff as usize + i as usize * header.e_phentsize as usize;
    elf::parse_program_header(&elf_bytes[offset..offset + header.e_phentsize as usize])
        .map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))
}

fn bounds(elf_bytes: &[u8], header: &ElfHeader) -> uefi::Result<(u64, u64)> {
    let mut min = u64::MAX;
    let mut max = 0;
    for i in 0..header.e_phnum {
        let program_header = program_header_at(elf_bytes, &header, i)?;
        if program_header.p_type != PT_LOAD {
            continue;
        }

        min = u64::min(min, program_header.p_paddr);
        max = u64::max(max, program_header.p_paddr + program_header.p_memsz);
    }

    if min == u64::MAX {
        return Err(uefi::Error::from(uefi::Status::LOAD_ERROR));
    }
    min = min & !(PAGE_ROUND_MASK);
    max = (max + PAGE_ROUND_MASK) & !(PAGE_ROUND_MASK);
    Ok((min, max))
}

pub fn load_kernel(elf_bytes: &[u8]) -> uefi::Result<u64> {
    let header =
        elf::parse_header(elf_bytes).map_err(|_| uefi::Error::from(uefi::Status::LOAD_ERROR))?;
    let (min, max) = bounds(elf_bytes, &header)?;
    let pages = (max - min) / PAGE_SIZE;
    let _result = boot::allocate_pages(
        AllocateType::Address(min),
        MemoryType::LOADER_DATA,
        pages as usize,
    )?;

    for i in 0..header.e_phnum {
        let program_header = program_header_at(elf_bytes, &header, i)?;
        if program_header.p_type != 1 {
            continue;
        }

        let dst = program_header.p_paddr as usize as *mut u8;

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
