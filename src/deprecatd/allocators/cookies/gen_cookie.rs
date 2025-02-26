use crate::allocators::{usize_half, HALF_USIZE};

use super::Cookie;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenCookie {
    pub gen: usize_half,
    pub active: bool,
}

impl Cookie for GenCookie {
    #[inline]
    fn get_cookie(raw: usize) -> Self {
        let value = raw & ((1 << (HALF_USIZE - 1)) - 1);
        let flag = (raw >> (HALF_USIZE - 1)) & 1 != 0;

        GenCookie {
            gen: value as usize_half,
            active: flag,
        }
    }

    #[inline]
    fn get_repr(&self) -> usize {
        ((self.active as usize) << (HALF_USIZE - 1)) | self.gen as usize
    }
}
