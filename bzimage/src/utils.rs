pub fn read_u16(buf: &[u8], offset: usize) -> u16 {
    let value = buf[offset..offset + 2].try_into().unwrap();
    u16::from_le_bytes(value)
}

pub fn read_u32(buf: &[u8], offset: usize) -> u32 {
    let value = buf[offset..offset + 4].try_into().unwrap();
    u32::from_le_bytes(value)
}

pub fn read_u64(buf: &[u8], offset: usize) -> u64 {
    let value = buf[offset..offset + 8].try_into().unwrap();
    u64::from_le_bytes(value)
}

pub fn write_u16(buf: &mut [u8], offset: usize, value: u16) {
    buf[offset..offset + 2].copy_from_slice(&u16::to_le_bytes(value));
}
pub fn write_u32(buf: &mut [u8], offset: usize, value: u32) {
    buf[offset..offset + 4].copy_from_slice(&u32::to_le_bytes(value));
}
pub fn write_u64(buf: &mut [u8], offset: usize, value: u64) {
    buf[offset..offset + 8].copy_from_slice(&u64::to_le_bytes(value));
}
