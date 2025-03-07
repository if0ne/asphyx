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
pub struct SparseArray<U, W> {
    sparse: Vec<Option<RenderHandle<W>>>,
    dense: Vec<MaybeUninit<W>>,
    dense_to_sparse: Vec<usize>,
    _marker: PhantomData<U>,
}

impl<U, W> SparseArray<U, W> {
    pub fn new(capacity: usize) -> Self {
        Self {
            sparse: vec![None; capacity],
            dense: Vec::new(),
            dense_to_sparse: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn contains(&self, handle: RenderHandle<U>) -> bool {
        self.sparse
            .get(handle.index as usize)
            .is_some_and(|h| h.is_some_and(|h| h.gen == handle.gen))
    }

    pub fn set(&mut self, handle: RenderHandle<U>, value: W) {
        if self.sparse.len() <= handle.index as usize {
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
            self.dense_to_sparse.push(handle.index as usize);
            self.sparse[handle.index as usize] = Some(RenderHandle {
                index: pos as u32,
                gen: handle.gen,
                _marker: PhantomData,
            });
        }
    }

    pub fn get(&self, handle: RenderHandle<U>) -> Option<&W> {
        self.sparse.get(handle.index as usize).and_then(|h| {
            if let Some(h) = h {
                if h.gen == handle.gen {
                    unsafe { Some(self.dense[h.index as usize].assume_init_ref()) }
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    pub fn get_mut(&mut self, handle: RenderHandle<U>) -> Option<&mut W> {
        self.sparse.get(handle.index as usize).and_then(|h| {
            if let Some(h) = h {
                if h.gen == handle.gen {
                    unsafe { Some(self.dense[h.index as usize].assume_init_mut()) }
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    pub fn remove(&mut self, handle: RenderHandle<U>) {
        let Some(Some(sparse_h)) = self.sparse.get(handle.index as usize) else {
            return;
        };

        if sparse_h.gen != handle.gen {
            return;
        }

        let dense_pos = sparse_h.index as usize;

        unsafe {
            self.dense[dense_pos].assume_init_drop();
        }

        self.dense.swap_remove(dense_pos);
        self.dense_to_sparse.swap_remove(dense_pos);
        self.sparse[handle.index as usize] = None;

        if self.dense_to_sparse.len() > 0 {
            let Some(Some(handle)) = self.sparse.get_mut(self.dense_to_sparse[dense_pos]) else {
                return;
            };

            handle.index = dense_pos as u32;
        }
    }
}

impl<U, W> Drop for SparseArray<U, W> {
    fn drop(&mut self) {
        for handle in self.sparse.iter_mut() {
            if let Some(handle) = handle.take() {
                unsafe {
                    self.dense[handle.index as usize].assume_init_drop();
                }
            }
        }
    }
}
