#[derive(Debug)]

pub enum Error {
    TooShort,
    BadMagic,
    UnsupportedClass,
    UnsupportedEndianness,
}
