use num_traits::{PrimInt, WrappingAdd, WrappingSub};
use std::{mem, ops};

#[allow(unused)]
pub const trait Span {
    fn span(&self) -> usize;
}

const impl Span for ops::Range<usize> {
    fn span(&self) -> usize {
        self.end - self.start
    }
}

const impl Span for ops::RangeInclusive<usize> {
    fn span(&self) -> usize {
        (*self.end() - *self.start()) + 1
    }
}

pub trait NumTraitsExt
where
    Self: PrimInt,
{
    /// bitmask for the "half overflow"
    fn half_overflow_bitmask() -> Self;

    /// wrapping add and check overflow to `numbits/2` bit
    fn half_overflowing_add(self, other: Self) -> (Self, bool);

    /// wrapping sub and check borrow from `numbits/2` bit
    fn half_overflowing_sub(self, other: Self) -> (Self, bool);

    /// fold `self` and `others` with `predicate` which returns (Self, bool)
    fn fold_overflowing_op(
        self,
        others: &[Self],
        predicate: fn(Self, Self) -> (Self, bool),
    ) -> (Self, bool) {
        others.iter().fold((self, false), |(acc, carry1), n| {
            let (r, carry2) = predicate(acc, *n);
            (r, carry1 || carry2)
        })
    }
}

impl<T: PrimInt + WrappingAdd + WrappingSub> NumTraitsExt for T {
    fn half_overflow_bitmask() -> Self {
        let numbits = mem::size_of::<Self>() * 8;
        (Self::one() << (numbits / 2)) - Self::one()
    }

    fn half_overflowing_add(self, other: Self) -> (Self, bool) {
        let res = self.wrapping_add(&other);
        let bitmask = Self::half_overflow_bitmask();
        let half_carry = (self & bitmask) + (other & bitmask) > bitmask;
        (res, half_carry)
    }

    fn half_overflowing_sub(self, other: Self) -> (Self, bool) {
        let res = self.wrapping_sub(&other);
        let bitmask = Self::half_overflow_bitmask();
        let half_carry = (self & bitmask) < (other & bitmask);
        (res, half_carry)
    }
}

// TODO: const binary search
pub struct ConstMap<K: Eq, V, const N: usize>([(K, V); N]);

impl<K: Eq + Copy, V: Copy, const N: usize> ConstMap<K, V, N> {
    pub const fn new(map: [(K, V); N]) -> Self {
        //map.sort_by_key(|(k, _)| *k);
        ConstMap(map)
    }

    pub fn get(&self, key: K) -> Option<V> {
        // self.0
        //     .binary_search_by_key(key, |&(k, _)| k)
        //     .map(|idx| &self.0[idx].1)
        //     .ok()
        self.0
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v)
            .copied() // TODO: why is `map(|(_, v)| v)` not const?
    }

    #[allow(unused)]
    pub fn index(&self, idx: usize) -> V {
        self.0[idx].1
    }
}

// NOTE:
// - by keeping the generics all on the function (rather than a trait), the compiler infers them more easily
// - boxed_1d can work for >1D arrays if the number of elements is small, since Default is impl'd for small arrays

pub fn boxed_1d<T: Default + Copy, const N: usize>() -> Box<[T; N]> {
    Box::new([T::default(); N])
}

pub fn boxed_2d<T: Default + Copy, const W: usize, const H: usize>() -> Box<[[T; H]; W]> {
    Box::new([[T::default(); H]; W])
}

pub fn boxed_3d<T: Default + Copy, const W: usize, const H: usize, const D: usize>()
-> Box<[[[T; D]; H]; W]> {
    Box::new([[[T::default(); D]; H]; W])
}

pub trait Array2D<const W: usize, const H: usize> {
    type Item;

    fn get_1to2(&self, idx: impl Into<usize>) -> &Self::Item;

    fn get_mut_1to2(&mut self, idx: impl Into<usize>) -> &mut Self::Item;

    fn idx_1to2(idx: usize) -> (usize, usize) {
        if idx >= W * H {
            panic!("Index {idx} out of bounds for array {W}x{H}");
        }
        (idx / H, idx % H)
    }

    #[allow(unused)]
    fn idx_2to1((x, y): (usize, usize)) -> usize {
        if x >= W || y >= H {
            panic!("Index {x}x{y} out of bounds for array {W}x{H}");
        }
        x * H + y
    }

    fn set_1to2(&mut self, idx: impl Into<usize>, val: Self::Item) {
        *self.get_mut_1to2(idx) = val;
    }
}

impl<T, const W: usize, const H: usize> Array2D<W, H> for [[T; H]; W] {
    type Item = T;

    fn get_1to2(&self, idx: impl Into<usize>) -> &T {
        let (x, y) = Self::idx_1to2(idx.into());
        &self[x][y]
    }

    fn get_mut_1to2(&mut self, idx: impl Into<usize>) -> &mut T {
        let (x, y) = Self::idx_1to2(idx.into());
        &mut self[x][y]
    }
}

pub trait Array3D<const W: usize, const H: usize, const D: usize> {
    type Item;

    fn get_1to3(&self, idx: impl Into<usize>) -> &Self::Item;

    fn get_mut_1to3(&mut self, idx: impl Into<usize>) -> &mut Self::Item;

    fn idx_1to3(idx: usize) -> (usize, usize, usize) {
        if idx >= W * H * D {
            panic!("Index {idx} out of bounds for array {W}x{H}x{D}");
        }
        let x = idx / (H * D);
        let rem = idx % (H * D);
        let y = rem / D;
        let z = rem % D;

        (x, y, z)
    }

    #[allow(unused)]
    fn idx_3to1((x, y, z): (usize, usize, usize)) -> usize {
        if x >= W || y >= H || z >= D {
            panic!("Index {x}x{y}x{z} out of bounds for array {W}x{H}x{D}");
        }
        x * H * D + y * D + z
    }

    fn set_1to3(&mut self, idx: impl Into<usize>, val: Self::Item) {
        *self.get_mut_1to3(idx) = val;
    }
}

impl<T, const W: usize, const H: usize, const D: usize> Array3D<W, H, D> for [[[T; D]; H]; W] {
    type Item = T;

    fn get_1to3(&self, idx: impl Into<usize>) -> &T {
        let (x, y, z) = Self::idx_1to3(idx.into());
        &self[x][y][z]
    }

    fn get_mut_1to3(&mut self, idx: impl Into<usize>) -> &mut T {
        let (x, y, z) = Self::idx_1to3(idx.into());
        &mut self[x][y][z]
    }
}
