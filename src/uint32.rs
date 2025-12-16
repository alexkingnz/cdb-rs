// Contributions from Bruce Guenter and Alex King
// This file is in the public domain

pub fn unpack(data: &[u8]) -> u32 {
    u32::from_le_bytes(data[0..4].try_into().unwrap())
}

pub fn unpack2(buf: &[u8]) -> (u32, u32) {
    (unpack(&buf[0..4]), unpack(&buf[4..8]))
}

fn _pack(src: u32) -> [u8; 4] {
    src.to_le_bytes()
}

pub fn pack(data: &mut [u8], src: u32) {
    assert!(data.len() >= 4);
    data[..4].copy_from_slice(&_pack(src));
}

pub fn pack2(data: &mut [u8], src0: u32, src1: u32) {
    assert!(data.len() >= 8);
    pack(&mut data[0..4], src0);
    pack(&mut data[4..8], src1);
}
