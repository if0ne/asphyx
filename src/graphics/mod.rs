use std::marker::PhantomData;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Handle<T> {
    id: u32,
    gen: u32,
    _marker: PhantomData<T>,
}

#[derive(Debug)]
pub struct Pool<T> {
    array: Vec<Option<T>>,
    gens: Vec<u32>,
    free_list: Vec<usize>,
}

impl<T> Pool<T> {
    pub fn new(capacity: Option<usize>) -> Self {
        let capacity = capacity.unwrap_or(128);

        Self {
            array: Vec::with_capacity(capacity),
            gens: Vec::with_capacity(capacity),
            free_list: Vec::new(),
        }
    }

    pub fn push(&mut self, value: T) -> Handle<T> {
        let idx = self.free_list.pop().unwrap_or_else(|| {
            let idx = self.array.len();
            self.array.push(None);
            self.gens.push(1);

            idx
        });
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
        self.free_list.push(handle.id as usize);

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
