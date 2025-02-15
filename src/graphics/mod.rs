use std::marker::PhantomData;

const ID_BITS: usize = 32;
const GENERATION_BITS: usize = 28;

const ID_MASK: usize = !0 >> ID_BITS;
const DEVICE_MASK: usize = !0 << (ID_BITS + GENERATION_BITS);
const GENERATION_MASK: usize = !0 & !ID_MASK & !DEVICE_MASK;

#[derive(Clone, Copy, Default)]
pub struct Handle<T> {
    pub v: usize,
    _marker: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new(id: usize, gen: usize, device_mask: usize) -> Self {
        Self {
            v: device_mask << (ID_BITS + GENERATION_BITS) | gen << ID_BITS | id,
            _marker: PhantomData,
        }
    }

    pub fn id(&self) -> usize {
        self.v & ID_MASK
    }

    pub fn gen(&self) -> usize {
        self.v & GENERATION_MASK
    }

    pub fn device_mask(&self) -> usize {
        self.v & DEVICE_MASK
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id() && self.gen() == other.gen()
    }
}

impl<T> Eq for Handle<T> {}
