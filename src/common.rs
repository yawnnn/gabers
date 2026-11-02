use num_traits::{PrimInt, WrappingAdd, WrappingSub, ops::overflowing::OverflowingSub};
use std::ops;

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
    Self: PrimInt + WrappingAdd + WrappingSub + OverflowingSub + From<bool> + Into<usize>,
{
    fn half_carry_bitmask() -> usize;

    /// like `carrying_add` but checks half-carry
    fn half_carrying_add(self, other: Self, carry: bool) -> (Self, bool) {
        let res = self.wrapping_add(&other).wrapping_add(&carry.into());
        let bitmask = Self::half_carry_bitmask();
        let half_carry =
            (self.into() & bitmask) + (other.into() & bitmask) + (carry as usize) > bitmask;
        (res, half_carry)
    }

    /// like `borrowing_sub` but checks half-borrow
    fn half_borrowing_sub(self, other: Self, borrow: bool) -> (Self, bool) {
        let res = self.wrapping_sub(&other).wrapping_sub(&borrow.into());
        let bitmask = Self::half_carry_bitmask();
        let half_borrow = (self.into() & bitmask) < (other.into() & bitmask) + (borrow as usize);
        (res, half_borrow)
    }
}

impl NumTraitsExt for u8 {
    fn half_carry_bitmask() -> usize {
        0x0F
    }
}

impl NumTraitsExt for u16 {
    fn half_carry_bitmask() -> usize {
        0x0FFF
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

pub fn boxed_2d<T: Default + Copy, const W: usize, const H: usize>() -> Box<[[T; W]; H]> {
    Box::new([[T::default(); W]; H])
}

pub fn boxed_3d<T: Default + Copy, const W: usize, const H: usize, const D: usize>()
-> Box<[[[T; W]; H]; D]> {
    Box::new([[[T::default(); W]; H]; D])
}

pub trait Array2D<const W: usize, const H: usize> {
    type Item;

    fn get_1to2(&self, idx: impl Into<usize>) -> &Self::Item;

    fn get_mut_1to2(&mut self, idx: impl Into<usize>) -> &mut Self::Item;

    fn idx_1to2(idx: usize) -> (usize, usize) {
        if idx >= W * H {
            panic!("Index {idx} out of bounds for array {W};{H}");
        }
        (idx % W, idx / W)
    }

    #[allow(unused)]
    fn idx_2to1((x, y): (usize, usize)) -> usize {
        if x >= W || y >= H {
            panic!("Index {x};{y} out of bounds for array {W};{H}");
        }
        x + y * W
    }

    fn set_1to2(&mut self, idx: impl Into<usize>, val: Self::Item) {
        *self.get_mut_1to2(idx) = val;
    }
}

impl<T, const W: usize, const H: usize> Array2D<W, H> for [[T; W]; H] {
    type Item = T;

    fn get_1to2(&self, idx: impl Into<usize>) -> &T {
        let (x, y) = Self::idx_1to2(idx.into());
        &self[y][x]
    }

    fn get_mut_1to2(&mut self, idx: impl Into<usize>) -> &mut T {
        let (x, y) = Self::idx_1to2(idx.into());
        &mut self[y][x]
    }
}

pub trait Array3D<const W: usize, const H: usize, const D: usize> {
    type Item;

    fn get_1to3(&self, idx: impl Into<usize>) -> &Self::Item;

    fn get_mut_1to3(&mut self, idx: impl Into<usize>) -> &mut Self::Item;

    fn idx_1to3(idx: usize) -> (usize, usize, usize) {
        if idx >= W * H * D {
            panic!("Index {idx} out of bounds for array {W};{H};{D}");
        }
        let z = idx / (W * H);
        let rem = idx % (W * H);
        let y = rem / W;
        let x = rem % W;

        (x, y, z)
    }

    #[allow(unused)]
    fn idx_3to1((x, y, z): (usize, usize, usize)) -> usize {
        if x >= W || y >= H || z >= D {
            panic!("Index {x};{y};{z} out of bounds for array {W};{H};{D}");
        }
        x + y * W + z * W * H
    }

    fn set_1to3(&mut self, idx: impl Into<usize>, val: Self::Item) {
        *self.get_mut_1to3(idx) = val;
    }
}

impl<T, const W: usize, const H: usize, const D: usize> Array3D<W, H, D> for [[[T; W]; H]; D] {
    type Item = T;

    fn get_1to3(&self, idx: impl Into<usize>) -> &T {
        let (x, y, z) = Self::idx_1to3(idx.into());
        &self[z][y][x]
    }

    fn get_mut_1to3(&mut self, idx: impl Into<usize>) -> &mut T {
        let (x, y, z) = Self::idx_1to3(idx.into());
        &mut self[z][y][x]
    }
}
