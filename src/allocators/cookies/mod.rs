mod gen_cookie;

pub use gen_cookie::*;

use std::fmt::Debug;

pub trait Cookie: Copy + Debug + Sized {
    fn get_cookie(raw: usize) -> Self;
    fn get_repr(&self) -> usize;
}
