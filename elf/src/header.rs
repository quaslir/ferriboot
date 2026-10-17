use crate::error::ElfError;

pub struct ElfHeader {
    pub entry: u64,
}

pub fn parse_header(bytes: &[u8]) -> Result<ElfHeader, ElfError> {
    if bytes.len() < 32 {
        return Err(ElfError::TooShort);
    }

    if bytes[0..4] != [0x7F, b'E', b'L', b'F'] {
        return Err(ElfError::BadMagic);
    }
    if bytes[4] != 2 {
        return Err(ElfError::UnsupportedClass);
    }

    if bytes[5] != 1 {
        return Err(ElfError::UnsupportedEndianness);
    }
    let entry = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
    Ok(ElfHeader { entry: entry })
}
