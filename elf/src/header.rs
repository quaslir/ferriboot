use crate::error::ElfError;

pub struct ElfHeader {
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_phentsize: u16,
    pub e_phnum: u16,
}

pub fn parse_header(bytes: &[u8]) -> Result<ElfHeader, ElfError> {
    if bytes.len() < 64 {
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
    let phoff = u64::from_le_bytes(bytes[32..40].try_into().unwrap());
    let phentsize = u16::from_le_bytes(bytes[54..56].try_into().unwrap());
    let phnum = u16::from_le_bytes(bytes[56..58].try_into().unwrap());
    Ok(ElfHeader {
        e_entry: entry,
        e_phoff: phoff,
        e_phentsize: phentsize,
        e_phnum: phnum,
    })
}

#[cfg(test)]

mod tests {

    use super::*;

    #[test]
    fn rejects_too_short() {
        let bytes = [0];
        let result = parse_header(&bytes);
        assert!(matches!(result, Err(ElfError::TooShort)));
    }

    #[test]

    fn invalid_magic() {
        let bytes: [u8; 64] = [0; 64];
        let result = parse_header(&bytes);
        assert!(matches!(result, Err(ElfError::BadMagic)));
    }

    #[test]

    fn unsupported_class() {
        let mut bytes: [u8; 64] = [0; 64];
        bytes[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        let result = parse_header(&bytes);
        assert!(matches!(result, Err(ElfError::UnsupportedClass)));
    }

    #[test]

    fn unsupported_endianness() {
        let mut bytes: [u8; 64] = [0; 64];
        bytes[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        bytes[4] = 2;
        let result = parse_header(&bytes);
        assert!(matches!(result, Err(ElfError::UnsupportedEndianness)));
    }

    #[test]

    fn success_parse() {
        let mut bytes: [u8; 64] = [0; 64];
        bytes[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        bytes[4] = 2;
        bytes[5] = 1;
        bytes[24..32].copy_from_slice(&0x1000u64.to_le_bytes());
        bytes[32..40].copy_from_slice(&0x2000u64.to_le_bytes());
        bytes[54..56].copy_from_slice(&56u16.to_le_bytes());
        bytes[56..58].copy_from_slice(&3u16.to_le_bytes());
        let result = parse_header(&bytes);
        let header = result.unwrap();
        assert_eq!(header.e_entry, 0x1000);
        assert_eq!(header.e_phoff, 0x2000);
        assert_eq!(header.e_phentsize, 56);
        assert_eq!(header.e_phnum, 3);
    }
}
