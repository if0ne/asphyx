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
        if let Some(idx) = self.free_list.pop() {
            idx
        } else {
            let idx = self.size;
            self.size += 1;

            idx
        }
    }

    pub fn free(&mut self, idx: usize) {
        if idx >= self.size {
            std::hint::cold_path();
            return;
        }

        self.free_list.push(idx);
    }
}
