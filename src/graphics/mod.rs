pub mod allocators;
pub mod mock;
pub mod types;

use std::{marker::PhantomData, mem::MaybeUninit, usize};

use allocators::LinearIndexAllocator;

#[cfg(target_pointer_width = "32")]
type usize_half = u16;

#[cfg(target_pointer_width = "64")]
type usize_half = u32;

const HALF_USIZE: usize = usize::BITS as usize / 2;
const ID_MASK: usize = (1 << HALF_USIZE) - 1;
const COOKIE_MASK: usize = !((1 << HALF_USIZE) - 1);

#[derive(Debug)]
pub struct Handle<T, C: Cookie = GenCookie> {
    id: usize,
    _marker: PhantomData<fn() -> (T, C)>,
}

impl<T, C: Cookie> Handle<T, C> {
    #[inline]
    pub fn new(id: u32, cookie: C) -> Self {
        Self {
            id: (cookie.get_repr() << usize::BITS / 2) | id as usize,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.id & ID_MASK
    }

    #[inline]
    pub fn cookie(&self) -> C {
        C::get_cookie((self.id & COOKIE_MASK) >> usize::BITS / 2)
    }
}

impl<T, C: Cookie> Clone for Handle<T, C> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T, C: Cookie> Copy for Handle<T, C> {}

impl<T, C: Cookie> Default for Handle<T, C> {
    fn default() -> Self {
        Self {
            id: usize::MAX,
            _marker: PhantomData,
        }
    }
}

impl<T, C: Cookie> PartialEq for Handle<T, C> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T, C: Cookie> Eq for Handle<T, C> {}

#[derive(Debug)]
pub struct Pool<T> {
    array: Vec<PoolEntry<T>>,
    allocator: LinearIndexAllocator,
}

#[derive(Debug)]
struct PoolEntry<T> {
    item: MaybeUninit<T>,
    gen: GenCookie,
}

impl<T> Drop for PoolEntry<T> {
    fn drop(&mut self) {
        if self.gen.active {
            unsafe {
                self.item.assume_init_drop();
            }
        }
    }
}

impl<T> Pool<T> {
    pub fn new(capacity: Option<usize>) -> Self {
        let capacity = capacity.unwrap_or(128);

        Self {
            array: Vec::with_capacity(capacity),
            allocator: LinearIndexAllocator::new(),
        }
    }

    pub fn push(&mut self, value: T) -> Handle<T, GenCookie> {
        let idx = self.allocator.allocate();

        let gen = if idx == self.array.len() {
            self.array.push(PoolEntry {
                item: MaybeUninit::new(value),
                gen: GenCookie {
                    gen: 1,
                    active: true,
                },
            });

            GenCookie {
                gen: 1,
                active: true,
            }
        } else {
            self.array[idx].item = MaybeUninit::new(value);
            self.array[idx].gen.active = true;
            self.array[idx].gen
        };

        Handle::new(idx as u32, gen)
    }

    pub fn remove(&mut self, handle: Handle<T, GenCookie>) {
        if let Some(e) = self.array.get_mut(handle.index()) {
            if e.gen != handle.cookie() {
                std::hint::cold_path();
                return;
            }

            self.allocator.free(handle.index());
            e.gen.gen = e.gen.gen.wrapping_add(1);
            e.gen.active = false;

            unsafe {
                e.item.assume_init_drop();
            }

            return;
        }

        std::hint::cold_path();
    }

    pub fn get(&self, handle: Handle<T, GenCookie>) -> Option<&T> {
        if let Some(e) = self.array.get(handle.index()) {
            if e.gen != handle.cookie() {
                std::hint::cold_path();
                return None;
            }

            unsafe { Some(e.item.assume_init_ref()) }
        } else {
            std::hint::cold_path();
            None
        }
    }

    pub fn get_mut(&mut self, handle: Handle<T, GenCookie>) -> Option<&mut T> {
        if let Some(e) = self.array.get_mut(handle.index()) {
            if e.gen != handle.cookie() {
                std::hint::cold_path();
                return None;
            }

            unsafe { Some(e.item.assume_init_mut()) }
        } else {
            std::hint::cold_path();
            None
        }
    }

    pub fn get_many<const N: usize>(
        &mut self,
        handles: [Handle<T, GenCookie>; N],
    ) -> Option<[&mut T; N]> {
        let indices = handles.map(|v| v.index());

        let entries = self.array.get_disjoint_mut(indices).ok()?;

        if !entries
            .iter()
            .zip(handles.iter())
            .all(|(e, h)| e.gen == h.cookie())
        {
            std::hint::cold_path();
            return None;
        }

        Some(entries.map(|e| unsafe { e.item.assume_init_mut() }))
    }
}

pub trait Cookie: Copy + Sized {
    fn get_cookie(raw: usize) -> Self;
    fn get_repr(&self) -> usize;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenCookie {
    gen: usize_half,
    active: bool,
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
