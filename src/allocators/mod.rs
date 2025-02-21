pub mod handle;
pub mod linear_index_allocator;
pub mod pool_allocator;
pub mod ring_buffer_index_allocator;

mod cookies;

pub use handle::*;
pub use linear_index_allocator::*;
pub use pool_allocator::*;
pub use ring_buffer_index_allocator::*;

#[cfg(target_pointer_width = "32")]
#[allow(non_camel_case_types)]
pub type usize_half = u16;

#[cfg(target_pointer_width = "64")]
#[allow(non_camel_case_types)]
pub type usize_half = u32;

const HALF_USIZE: usize = usize::BITS as usize / 2;
const ID_MASK: usize = (1 << HALF_USIZE) - 1;
const COOKIE_MASK: usize = !((1 << HALF_USIZE) - 1);
