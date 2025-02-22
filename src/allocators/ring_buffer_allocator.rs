use std::mem::MaybeUninit;

use super::{cookies::EmptyCookie, Handle, RingBufferIndexAllocator};

#[derive(Debug)]
pub struct RingBuffer<T, const N: usize> {
    array: Vec<RingBufferEntry<T>>,
    allocator: RingBufferIndexAllocator<N>,
}

#[derive(Debug)]
struct RingBufferEntry<T> {
    item: Option<T>,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new(capacity: Option<usize>) -> Self {
        let capacity = capacity.unwrap_or(1024);
        let mut array = Vec::with_capacity(N * capacity);
        array.resize_with(N * capacity, || RingBufferEntry { item: None });

        Self {
            array,
            allocator: RingBufferIndexAllocator::new(Some(capacity)),
        }
    }

    pub fn next_ring(&mut self) {
        self.allocator.next_ring();
    }

    pub fn push(&mut self, value: T) -> Option<Handle<T, EmptyCookie>> {
        let idx = self.allocator.allocate(1)?;

        self.array[idx].item = Some(value);

        Some(Handle::new(idx as u32, EmptyCookie))
    }

    pub fn get(&self, handle: Handle<T, EmptyCookie>) -> Option<&T> {
        self.array
            .get(handle.index())
            .map(|e| e.item.as_ref())
            .flatten()
    }

    pub fn get_mut(&mut self, handle: Handle<T, EmptyCookie>) -> Option<&mut T> {
        self.array
            .get_mut(handle.index())
            .map(|e| e.item.as_mut())
            .flatten()
    }

    pub fn get_many<const M: usize>(
        &mut self,
        handles: [Handle<T, EmptyCookie>; M],
    ) -> Option<[Option<&mut T>; M]> {
        let indices = handles.map(|v| v.index());

        let entries = self.array.get_disjoint_mut(indices).ok()?;

        Some(entries.map(|e| e.item.as_mut()))
    }
}
