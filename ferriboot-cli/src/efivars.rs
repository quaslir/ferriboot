use std::path::PathBuf;
pub const BLI_GUID: &str = "4a67b082-0a4c-41cf-b6c7-440b29bb8c4f";
pub const GLOBAL_GUID: &str = "8be4df61-93ca-11d2-aa0d-00e098032b8c";
pub fn read_bytes(name: &str, guid: &str) -> Option<Vec<u8>> {
    let path = PathBuf::from(format!("/sys/firmware/efi/efivars/{}-{}", name, guid));
    let bytes = std::fs::read(&path).ok()?;
    Some(bytes.get(4..)?.to_vec())
}

pub fn read_string(name: &str, guid: &str) -> Option<String> {
    let bytes = read_bytes(name, guid)?;
    let units = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect::<Vec<u16>>();
    let text = String::from_utf16(&units).ok()?;
    Some(text.trim_end_matches('\0').to_string())
}
