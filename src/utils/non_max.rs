use std::{
    num::{NonZero, ZeroablePrimitive},
    ops::Not,
};

use static_assertions::const_assert_eq;

const_assert_eq!(
    size_of::<Option<NonMax<usize>>>(),
    size_of::<NonMax<usize>>()
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonMax<T: MaxAndZero>(NonZero<T>);

impl<T: MaxAndZero + std::fmt::Display> std::fmt::Display for NonMax<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: MaxAndZero> NonMax<T> {
    #[inline]
    pub fn new(n: T) -> Option<Self> {
        let n = NonZero::new(!n)?;

        Some(Self(n))
    }

    #[inline]
    pub unsafe fn new_unchecked(n: T) -> Self {
        Self(NonZero::new_unchecked(n))
    }

    #[inline]
    pub fn get(self) -> T {
        let n = self.0.get();
        !n
    }
}

pub trait MaxAndZero: ZeroablePrimitive + PartialEq + Not<Output = Self> {
    const MAX: Self;
    const ZERO: Self;
}

macro_rules! impl_max_and_zero {
    ($($t:ty),*) => {
        $(
            impl MaxAndZero for $t {
                const MAX: Self = <$t>::MAX;
                const ZERO: Self = 0 as Self;
            }
        )*
    };
}

impl_max_and_zero!(usize, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, isize);
