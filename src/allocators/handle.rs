use std::marker::PhantomData;

use super::{
    cookies::{Cookie, GenCookie},
    COOKIE_MASK, ID_MASK,
};

pub struct Handle<T, C: Cookie = GenCookie> {
    id: usize,
    _marker: PhantomData<(T, C)>,
}

impl<T, C: Cookie> Handle<T, C> {
    #[inline]
    pub fn new(id: u32, cookie: C) -> Self {
        Self {
            id: (cookie.get_repr() << usize::BITS / 2) | id as usize,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.id & ID_MASK
    }

    #[inline]
    pub fn cookie(&self) -> C {
        C::get_cookie((self.id & COOKIE_MASK) >> usize::BITS / 2)
    }
}

impl<T, C: Cookie> Clone for Handle<T, C> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T, C: Cookie> Copy for Handle<T, C> {}

impl<T, C: Cookie> Default for Handle<T, C> {
    fn default() -> Self {
        Self {
            id: usize::MAX,
            _marker: PhantomData,
        }
    }
}

impl<T, C: Cookie> PartialEq for Handle<T, C> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T, C: Cookie> Eq for Handle<T, C> {}

impl<T, C: Cookie> std::fmt::Debug for Handle<T, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("index", &self.index())
            .field("cookie", &self.cookie())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UntypedHandle {
    id: usize,
}

impl UntypedHandle {
    #[inline]
    pub fn index(&self) -> usize {
        self.id & ID_MASK
    }

    #[inline]
    pub fn cookie(&self) -> usize {
        (self.id & COOKIE_MASK) >> usize::BITS / 2
    }
}

impl<T, C: Cookie> From<UntypedHandle> for Handle<T, C> {
    fn from(value: UntypedHandle) -> Self {
        Self {
            id: value.id,
            _marker: PhantomData,
        }
    }
}

impl<T, C: Cookie> From<Handle<T, C>> for UntypedHandle {
    fn from(value: Handle<T, C>) -> Self {
        Self { id: value.id }
    }
}
