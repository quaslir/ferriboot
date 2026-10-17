use crate::ElfError;

pub struct ProgramHeader {
    pub p_type: u32,
    pub p_offset: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
}

pub fn parse_program_header(bytes: &[u8]) -> Result<ProgramHeader, ElfError> {
    if bytes.len() < 56 {
        return Err(ElfError::TooShort);
    }

    let seg_type = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let offset = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let addr = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
    let filesz = u64::from_le_bytes(bytes[32..40].try_into().unwrap());
    let memsz = u64::from_le_bytes(bytes[40..48].try_into().unwrap());

    Ok(ProgramHeader {
        p_type: seg_type,
        p_offset: offset,
        p_paddr: addr,
        p_filesz: filesz,
        p_memsz: memsz,
    })
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]

    pub fn too_short() {
        let bytes: [u8; 32] = [0; 32];
        let result = parse_program_header(&bytes);
        assert!(matches!(result, Err(ElfError::TooShort)));
    }

    #[test]

    pub fn success_parse_program_header() {
        let mut bytes: [u8; 64] = [0; 64];
        bytes[0..4].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&0x1000u64.to_le_bytes());
        bytes[24..32].copy_from_slice(&0x100000u64.to_le_bytes());
        bytes[32..40].copy_from_slice(&0x2000u64.to_le_bytes());
        bytes[40..48].copy_from_slice(&0x3000u64.to_le_bytes());
        let result = parse_program_header(&bytes);
        let program_header = result.unwrap();
        assert_eq!(program_header.p_type, 1);
        assert_eq!(program_header.p_offset, 0x1000);
        assert_eq!(program_header.p_paddr, 0x100000);
        assert_eq!(program_header.p_filesz, 0x2000);
        assert_eq!(program_header.p_memsz, 0x3000);
    }
}
