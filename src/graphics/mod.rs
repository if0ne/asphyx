pub mod allocators;
pub mod mock;
pub mod types;

use std::marker::PhantomData;

use allocators::LinearIndexAllocator;

pub struct Handle<T> {
    id: u32,
    gen: u32,
    _marker: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            gen: self.gen,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self {
            id: 0,
            gen: 0,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct Pool<T> {
    array: Vec<Option<T>>,
    gens: Vec<u32>,
    allocator: LinearIndexAllocator,
}

impl<T> Pool<T> {
    pub fn new(capacity: Option<usize>) -> Self {
        let capacity = capacity.unwrap_or(128);

        Self {
            array: Vec::with_capacity(capacity),
            gens: Vec::with_capacity(capacity),
            allocator: LinearIndexAllocator::new(),
        }
    }

    pub fn push(&mut self, value: T) -> Handle<T> {
        let idx = self.allocator.allocate();

        if idx == self.array.len() {
            self.array.push(None);
            self.gens.push(1);
        }

        self.array[idx] = Some(value);

        Handle {
            id: idx as u32,
            gen: self.gens[idx],
            _marker: PhantomData,
        }
    }

    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        if handle.id as usize >= self.array.len() || handle.id as usize >= self.gens.len() {
            return None;
        }

        if handle.gen != self.gens[handle.id as usize] {
            return None;
        }

        let value = self.array[handle.id as usize].take();
        self.gens[handle.id as usize] += 1;
        self.allocator.free(handle.id as usize);

        value
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        if handle.id as usize >= self.array.len() || handle.id as usize >= self.gens.len() {
            return None;
        }

        if handle.gen != self.gens[handle.id as usize] {
            return None;
        }

        self.array[handle.id as usize].as_ref()
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        if handle.id as usize >= self.array.len() || handle.id as usize >= self.gens.len() {
            return None;
        }

        if handle.gen != self.gens[handle.id as usize] {
            return None;
        }

        self.array[handle.id as usize].as_mut()
    }
}
