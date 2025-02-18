pub mod allocators;
pub mod mock;
pub mod types;

use std::{marker::PhantomData, mem::MaybeUninit, usize};

use allocators::LinearIndexAllocator;

#[derive(Debug)]
pub struct Handle<T> {
    id: usize,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    const ID_MASK: usize = !0 >> usize::BITS / 2;
    const GEN_MASK: usize = !0 << usize::BITS / 2;

    #[inline]
    pub fn new(id: u32, gen: u32) -> Self {
        Self {
            id: ((gen as usize) << usize::BITS / 2) | id as usize,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.id & Self::ID_MASK
    }

    #[inline]
    pub fn gen(&self) -> usize {
        (self.id & Self::GEN_MASK) >> usize::BITS / 2
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self {
            id: usize::MAX,
            _marker: PhantomData,
        }
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Handle<T> {}

#[derive(Debug)]
pub struct Pool<T> {
    array: Vec<PoolEntry<T>>,
    allocator: LinearIndexAllocator,
}

#[derive(Debug)]
struct PoolEntry<T> {
    item: MaybeUninit<T>,
    gen: u32,
    active: bool,
}

impl<T> Drop for PoolEntry<T> {
    fn drop(&mut self) {
        if self.active {
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

    pub fn push(&mut self, value: T) -> Handle<T> {
        let idx = self.allocator.allocate();

        let gen = if idx == self.array.len() {
            self.array.push(PoolEntry {
                item: MaybeUninit::new(value),
                gen: 1,
                active: true,
            });

            1
        } else {
            self.array[idx].item = MaybeUninit::new(value);
            self.array[idx].active = true;
            self.array[idx].gen
        };

        Handle::new(idx as u32, gen)
    }

    pub fn remove(&mut self, handle: Handle<T>) {
        if let Some(e) = self.array.get_mut(handle.index()) {
            if e.gen != handle.gen() as u32 {
                std::hint::cold_path();
                return;
            }

            self.allocator.free(handle.index());
            e.gen += 1;
            e.active = false;

            unsafe {
                e.item.assume_init_drop();
            }

            return;
        }

        std::hint::cold_path();
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        if let Some(e) = self.array.get(handle.index()) {
            if e.gen != handle.gen() as u32 {
                std::hint::cold_path();
                return None;
            }

            unsafe { Some(e.item.assume_init_ref()) }
        } else {
            std::hint::cold_path();
            None
        }
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        if let Some(e) = self.array.get_mut(handle.index()) {
            if e.gen != handle.gen() as u32 {
                std::hint::cold_path();
                return None;
            }

            unsafe { Some(e.item.assume_init_mut()) }
        } else {
            std::hint::cold_path();
            None
        }
    }

    pub fn get_many<const N: usize>(&mut self, handles: [Handle<T>; N]) -> Option<[&mut T; N]> {
        let indices = handles.map(|v| v.index());

        let entries = self.array.get_disjoint_mut(indices).ok()?;

        if !entries
            .iter()
            .zip(handles.iter())
            .all(|(e, h)| e.gen == h.gen() as u32)
        {
            std::hint::cold_path();
            return None;
        }

        Some(entries.map(|e| unsafe { e.item.assume_init_mut() }))
    }
}
