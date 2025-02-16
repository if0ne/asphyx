#[derive(Debug)]
pub struct LinearIndexAllocator {
    size: usize,
    free_list: Vec<usize>,
}

impl LinearIndexAllocator {
    pub fn new() -> Self {
        Self {
            size: 0,
            free_list: Vec::new(),
        }
    }

    pub fn allocate(&mut self) -> usize {
        self.free_list.pop().unwrap_or_else(|| {
            let idx = self.size;
            self.size += 1;

            idx
        })
    }

    pub fn free(&mut self, idx: usize) {
        if idx < self.size {
            self.free_list.push(idx);
        }
    }
}

#[derive(Debug)]
pub struct RingBufferIndexAllocator<const N: usize> {
    back: usize,
    front: usize,

    size: usize,
    capacity: usize,
    sizes: [usize; N],

    index: usize,
}

impl<const N: usize> RingBufferIndexAllocator<N> {
    pub fn new(capacity: Option<usize>) -> Self {
        Self {
            back: 0,
            front: 0,
            size: 0,
            capacity: capacity.unwrap_or(128),
            sizes: [0; N],
            index: 0,
        }
    }

    pub fn next_ring(&mut self) {
        self.index = (self.index + 1) % N;
        let size_to_free = self.sizes[self.index];

        self.back = (self.back + size_to_free) % self.capacity;
        self.size -= size_to_free;
        self.sizes[self.index] = 0;
    }

    pub fn allocate(&mut self, size: usize) -> Option<usize> {
        if size > self.capacity {
            return None;
        }

        if self.front > self.back || (self.front == self.back && self.size == 0) {
            if self.size == 0 {
                self.front = 0;
                self.back = 0;
            }

            if self.front + size <= self.capacity {
                let idx = self.front;

                self.front = (self.front + size) % self.capacity;
                self.size += size;
                self.sizes[self.index] += size;

                return Some(idx);
            } else {
                if size > self.back {
                    return None;
                }

                let unused_reminder_at_end = self.capacity - self.front;
                self.size += unused_reminder_at_end;
                self.sizes[self.index] += unused_reminder_at_end;
                self.front = size % self.capacity;
                self.size += size;
                self.sizes[self.index] += size;

                return Some(0);
            }
        } else {
            if self.front + size > self.back {
                return None;
            }

            let idx = self.front;
            self.front = (self.front + size) % self.capacity;
            self.size += size;
            self.sizes[self.index] += size;

            return Some(idx);
        }
    }
}
