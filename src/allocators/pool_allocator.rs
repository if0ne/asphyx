use std::mem::MaybeUninit;

use super::{cookies::GenCookie, Handle, LinearIndexAllocator};

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
