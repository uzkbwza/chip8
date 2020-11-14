use std::num::Wrapping;

pub fn u12(i: u16) -> bool {
    // check if number is a valid u12, since rust doesnt support them natively.
    if i > 0xFFF {
        return false;
    }
    true
}

pub fn halfwords(bytes: &[u8]) -> Vec<u16> {
    // takes an array of bytes and constructs them into u16s
    let mut halfwords = Vec::new();
    for i in (0..bytes.len()).step_by(2) {
        let halfword = ((bytes[i] as u16) << 8) + bytes[i + 1] as u16;
        halfwords.push(halfword);
    }
    halfwords
}

pub fn get_bit_at(input: u8, n: u8) -> u8 {
    if n < 8 {
        input & (1 << n)
    } else {
        0
    }
}

pub trait Wrapable { 
    fn wrap_add(self, other: Self) -> Self;
    fn wrap_sub(self, other: Self) -> Self;
}

impl Wrapable for u8 {
    fn wrap_add(self, other: u8) -> u8 {
        (Wrapping(self) + Wrapping(other)).0
    }
    fn wrap_sub(self, other: u8) -> u8 {
        (Wrapping(self) - Wrapping(other)).0
    }
}

impl Wrapable for u16 {
    fn wrap_add(self, other: u16) -> u16 {
        (Wrapping(self) + Wrapping(other)).0
    }

    fn wrap_sub(self, other: u16) -> u16 {
        (Wrapping(self) - Wrapping(other)).0
    }
}
