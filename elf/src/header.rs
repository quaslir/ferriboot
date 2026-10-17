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
        let bytes: [u8; 32] = [0; 32];
        let result = parse_header(&bytes);
        assert!(matches!(result, Err(ElfError::BadMagic)));
    }

    #[test]

    fn unsupported_class() {
        let mut bytes: [u8; 32] = [0; 32];
        bytes[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        let result = parse_header(&bytes);
        assert!(matches!(result, Err(ElfError::UnsupportedClass)));
    }

    #[test]

    fn unsupported_endianness() {
        let mut bytes: [u8; 32] = [0; 32];
        bytes[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        bytes[4] = 2;
        let result = parse_header(&bytes);
        assert!(matches!(result, Err(ElfError::UnsupportedEndianness)));
    }

    #[test]

    fn success_parse() {
        let mut bytes: [u8; 32] = [0; 32];
        bytes[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        bytes[4] = 2;
        bytes[5] = 1;
        bytes[24..32].copy_from_slice(&0x1000u64.to_le_bytes());
        let result = parse_header(&bytes);
        let header = result.unwrap();
        assert_eq!(header.entry, 0x1000);
    }
}
