#[derive(Debug)]

pub enum ElfError {
    TooShort,
    BadMagic,
    UnsupportedClass,
    UnsupportedEndianness,
}
