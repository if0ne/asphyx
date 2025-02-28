use std::{hash::Hash, marker::PhantomData, mem::MaybeUninit};

use static_assertions::const_assert_eq;

use crate::utils::non_max::NonMax;

const_assert_eq!(
    size_of::<Option<RenderHandle<usize>>>(),
    size_of::<RenderHandle<usize>>()
);

pub struct RenderHandle<T> {
    index: u32,
    gen: NonMax<u32>,
    _marker: PhantomData<T>,
}

impl<T> Clone for RenderHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RenderHandle<T> {}

impl<T> Hash for RenderHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> PartialEq for RenderHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.gen == other.gen
    }
}

impl<T> Eq for RenderHandle<T> {}

impl<T> std::fmt::Debug for RenderHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceHandle")
            .field("index", &self.index)
            .field("gen", &self.gen())
            .finish()
    }
}

impl<T> RenderHandle<T> {
    pub fn new(index: u32, gen: u32) -> Self {
        Self {
            index,
            gen: NonMax::new(gen).expect("wrong gen"),
            _marker: PhantomData,
        }
    }

    pub fn idx(&self) -> u32 {
        self.index
    }

    pub fn gen(&self) -> u32 {
        self.gen.get()
    }
}

#[derive(Debug)]
pub struct RenderHandleAllocator<T> {
    gens: Vec<u32>,
    free_list: Vec<u32>,
    _marker: PhantomData<T>,
}

impl<T> RenderHandleAllocator<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            gens: Vec::new(),
            free_list: Vec::new(),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn allocate(&mut self) -> RenderHandle<T> {
        if let Some(idx) = self.free_list.pop() {
            RenderHandle::new(idx, self.gens[idx as usize])
        } else {
            let idx = self.gens.len();
            self.gens.push(0);

            RenderHandle::new(idx as u32, 0)
        }
    }

    #[inline]
    pub fn is_valid(&self, handle: RenderHandle<T>) -> bool {
        self.gens
            .get(handle.index as usize)
            .is_some_and(|h| *h == handle.gen.get())
    }

    #[inline]
    pub fn free(&mut self, handle: RenderHandle<T>) {
        if let Some(gen) = self.gens.get_mut(handle.index as usize) {
            *gen += 1;
            self.free_list.push(handle.index);
        }
    }
}

#[derive(Debug)]
pub struct SparseArray<T> {
    sparse: Vec<Option<RenderHandle<T>>>,
    dense: Vec<MaybeUninit<T>>,
}

impl<T> SparseArray<T> {
    pub fn contains(&self, handle: RenderHandle<T>) -> bool {
        self.sparse
            .get(handle.index as usize)
            .is_some_and(|h| h.is_some_and(|h| h.gen == handle.gen))
    }

    pub fn set(&mut self, handle: RenderHandle<T>, value: T) {
        if self.sparse.len() < handle.index as usize {
            self.sparse.resize((handle.index + 1) as usize, None);
        }

        if let Some(ref mut h) = self.sparse[handle.index as usize] {
            unsafe {
                self.dense[h.index as usize].assume_init_drop();
            }
            h.gen = handle.gen;
            self.dense[h.index as usize] = MaybeUninit::new(value);
        } else {
            let pos = self.dense.len();
            self.dense.push(MaybeUninit::new(value));
            self.sparse[handle.index as usize] = Some(RenderHandle {
                index: pos as u32,
                gen: handle.gen,
                _marker: PhantomData,
            });
        }
    }
}
