mod gen_cookie;

pub use gen_cookie::*;

use std::fmt::Debug;

pub trait Cookie: Copy + Debug + Sized {
    fn get_cookie(raw: usize) -> Self;
    fn get_repr(&self) -> usize;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmptyCookie;

impl Cookie for EmptyCookie {
    fn get_cookie(raw: usize) -> Self {
        EmptyCookie
    }

    fn get_repr(&self) -> usize {
        0
    }
}
