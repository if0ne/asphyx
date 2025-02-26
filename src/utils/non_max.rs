use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    num::{NonZero, ZeroablePrimitive},
    ops::{Add, Not},
};

use static_assertions::const_assert_eq;

const_assert_eq!(
    size_of::<Option<NonMax<usize>>>(),
    size_of::<NonMax<usize>>()
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NonMax<T: NumberLimits>(NonZero<T>);

impl<T: NumberLimits> PartialOrd for NonMax<T>
where
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl<T: NumberLimits + Ord> Ord for NonMax<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(&other.get())
    }
}

impl<T: NumberLimits> Hash for NonMax<T>
where
    T: Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

impl<T: NumberLimits + std::fmt::Display> std::fmt::Display for NonMax<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl<T: NumberLimits> NonMax<T> {
    #[inline]
    pub fn new(n: T) -> Option<Self> {
        let n = NonZero::new(n.wrapping_sub(T::MAX))?;

        Some(Self(n))
    }

    #[inline]
    pub unsafe fn new_unchecked(n: T) -> Self {
        debug_assert!(n != T::MAX, "NonMax::new_unchecked requires non-MAX value");

        Self(NonZero::new_unchecked(n.wrapping_sub(T::MAX)))
    }

    #[inline]
    pub fn get(self) -> T {
        let n = self.0.get();
        n.wrapping_add(T::MAX)
    }
}

pub trait NumberLimits:
    ZeroablePrimitive + PartialEq + Not<Output = Self> + Add<Output = Self>
{
    const MAX: Self;
    const MIN: Self;
    const ZERO: Self;

    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
}

macro_rules! impl_max_and_zero {
    ($($t:ty),*) => {
        $(
            impl NumberLimits for $t {
                const MAX: Self = <$t>::MAX;
                const MIN: Self = <$t>::MIN;
                const ZERO: Self = 0 as Self;

                fn wrapping_add(self, rhs: Self) -> Self {
                    self.wrapping_add(rhs)
                }

                fn wrapping_sub(self, rhs: Self) -> Self {
                    self.wrapping_sub(rhs)
                }
            }
        )*
    };
}

impl_max_and_zero!(usize, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    mod basic {
        use super::*;

        #[test]
        fn new_valid_values() {
            assert!(NonMax::<u8>::new(0).is_some());
            assert!(NonMax::<u8>::new(254).is_some());
            assert!(NonMax::<u8>::new(255).is_none());

            assert!(NonMax::<usize>::new(0).is_some());
            assert!(NonMax::<usize>::new(usize::MAX - 1).is_some());
            assert!(NonMax::<usize>::new(usize::MAX).is_none());
        }

        #[test]
        fn get_returns_correct_value() {
            let n = NonMax::<u16>::new(1234).unwrap();
            assert_eq!(n.get(), 1234);

            let n = NonMax::<i32>::new(-42).unwrap();
            assert_eq!(n.get(), -42);
        }

        #[test]
        fn display_formatting() {
            let n = NonMax::<u8>::new(100).unwrap();
            assert_eq!(format!("{}", n), "100");

            let n = NonMax::<i64>::new(-999).unwrap();
            assert_eq!(n.to_string(), "-999");
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn zero_handling() {
            assert!(NonMax::<u32>::new(0).is_some());
            assert!(NonMax::<i64>::new(0).is_some());
        }

        #[test]
        fn max_value_handling() {
            assert!(NonMax::<u8>::new(u8::MAX).is_none());
            assert!(NonMax::<i128>::new(i128::MAX).is_none());
        }
    }

    mod unsafe_tests {
        use super::*;

        #[test]
        fn new_unchecked_with_valid_value() {
            let n = unsafe { NonMax::<usize>::new_unchecked(42) };
            assert_eq!(n.get(), 42);
        }

        #[test]
        #[should_panic(expected = "requires non-MAX value")]
        fn new_unchecked_with_max_value() {
            let _ = unsafe { NonMax::<u8>::new_unchecked(u8::MAX) };
        }
    }

    mod layout_tests {
        use super::*;
        use static_assertions::const_assert_eq;

        #[test]
        fn size_check() {
            const_assert_eq!(
                size_of::<Option<NonMax<usize>>>(),
                size_of::<NonMax<usize>>()
            );

            assert_eq!(size_of::<Option<NonMax<u8>>>(), size_of::<NonMax<u8>>());
        }
    }

    mod trait_impls {
        use super::*;

        #[test]
        fn clone_and_copy() {
            let a = NonMax::<i32>::new(42).unwrap();
            let b = a.clone();
            assert_eq!(a, b);

            let c = a;
            let d = c;
            assert_eq!(c, d);
        }

        #[test]
        fn ordering() {
            let a = NonMax::<u8>::new(10).unwrap();
            let b = NonMax::<u8>::new(20).unwrap();
            assert!(a < b);
        }

        #[test]
        fn hash_impl() {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let a = NonMax::<usize>::new(42).unwrap();
            let mut hasher = DefaultHasher::new();
            a.hash(&mut hasher);
            let hash_a = hasher.finish();

            let mut hasher = DefaultHasher::new();
            42usize.hash(&mut hasher);
            let hash_b = hasher.finish();

            assert_eq!(hash_a, hash_b);
        }
    }
}
