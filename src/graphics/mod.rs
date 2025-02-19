pub mod allocators;
pub mod mock;
pub mod types;

use std::{marker::PhantomData, mem::MaybeUninit, usize};

use allocators::LinearIndexAllocator;

#[derive(Debug)]
pub struct Handle<T, C: Cookie = GenCookie> {
    id: usize,
    _marker: PhantomData<fn() -> (T, C)>,
}

impl<T, C: Cookie> Handle<T, C> {
    const ID_MASK: usize = !0 >> usize::BITS / 2;
    const COOKIE_MASK: usize = !0 << usize::BITS / 2;

    #[inline]
    pub fn new(id: u32, cookie: C) -> Self {
        Self {
            id: (cookie.get_repr() << usize::BITS / 2) | id as usize,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.id & Self::ID_MASK
    }

    #[inline]
    pub fn cookie(&self) -> C {
        C::get_cookie((self.id & Self::COOKIE_MASK) >> usize::BITS / 2)
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
        if self.gen.1 {
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
                gen: GenCookie(1, true),
            });

            GenCookie(1, true)
        } else {
            self.array[idx].item = MaybeUninit::new(value);
            self.array[idx].gen.1 = true;
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
            e.gen.0 += 1;
            e.gen.1 = false;

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
pub struct GenCookie(usize, bool);

impl Cookie for GenCookie {
    #[inline]
    fn get_cookie(raw: usize) -> Self {
        let value = raw & (!0 >> (usize::BITS / 2 + 1));
        let flag = (raw >> usize::BITS / 2 - 1) & 1 != 0;

        GenCookie(value, flag)
    }

    #[inline]
    fn get_repr(&self) -> usize {
        ((self.1 as usize) << usize::BITS / 2) | self.0
    }
}
