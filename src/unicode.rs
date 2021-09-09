// SPDX-License-Identifier: MIT OR BlueOak-1.0.0

#[inline]
pub fn is_lead_surrogate(cu: u16) -> bool {
    (0xD800..0xDC00).contains(&cu)
}

#[inline]
pub fn is_trail_surrogate(cu: u16) -> bool {
    (0xDC00..0xE000).contains(&cu)
}

#[inline]
pub fn compose_surrogates(ls: u16, ts: u16) -> char {
    char::from_u32((((ls - 0xD800) as u32) << 10 | (ts - 0xDC00) as u32) + 0x1_0000).unwrap()
}
