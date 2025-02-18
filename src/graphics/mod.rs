pub mod allocators;
pub mod mock;
pub mod types;

use std::{marker::PhantomData, usize};

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
    item: Option<T>,
    gen: u32,
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
                item: Some(value),
                gen: 1,
            });

            1
        } else {
            self.array[idx].item = Some(value);
            self.array[idx].gen
        };

        Handle::new(idx as u32, gen)
    }

    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        if let Some(e) = self.array.get_mut(handle.index()) {
            if e.gen != handle.gen() as u32 {
                std::hint::cold_path();
                return None;
            }

            self.allocator.free(handle.index());
            e.gen += 1;

            e.item.take()
        } else {
            std::hint::cold_path();
            None
        }
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        if let Some(e) = self.array.get(handle.index()) {
            if e.gen != handle.gen() as u32 {
                std::hint::cold_path();
                return None;
            }
            e.item.as_ref()
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
            e.item.as_mut()
        } else {
            std::hint::cold_path();
            None
        }
    }

    pub fn get_many<const N: usize>(
        &mut self,
        handles: [Handle<T>; N],
    ) -> Option<[Option<&mut T>; N]> {
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

        Some(entries.map(|e| e.item.as_mut()))
    }
}
