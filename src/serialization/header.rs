use std::ptr;

#[repr(C, packed)]
pub(crate) struct RawHeader {
    pub(crate) length: u64,
    pub(crate) checksum: u32,
    pub(crate) tag: u8,
}

pub(crate) unsafe fn serialize_header_unsafe(header: &RawHeader, buffer: &mut Vec<u8>) {
    let header_size = size_of::<RawHeader>();
    let offset = buffer.len();
    buffer.reserve(header_size);
    unsafe {
        buffer.set_len(offset + header_size);
        let dest_ptr = buffer.as_mut_ptr().add(offset);
        let header_ptr = header as *const RawHeader;
        ptr::copy_nonoverlapping(header_ptr as *const u8, dest_ptr, header_size);
    }
}

pub(crate) unsafe fn deserialize_header_unsafe(bytes: &[u8]) -> Option<RawHeader> {
    let header_size = size_of::<RawHeader>();
    if bytes.len() < header_size {
        return None;
    }
    let header_ptr = bytes.as_ptr() as *const RawHeader;
    unsafe {
        Some(ptr::read_unaligned(header_ptr))
    }
}

pub(crate) fn calculate_crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}
