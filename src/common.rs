use num_traits::PrimInt;
use num_traits::WrappingSub;
use num_traits::ops::overflowing::OverflowingAdd;
use std::ops;

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
    Self: std::marker::Sized,
{
    /// return a `Self` such that all bits up to the `bit`th are 1 and the rest are 0
    fn bitmask(bit: usize) -> Self;

    /// add `nums` with wrapping but check if `bit` has carry
    /// akin to overflowing_add but over N elements and bit-specific
    fn bit_overflowing_add(nums: &[Self], bit: usize) -> (Self, bool);

    /// sub `nums` with wrapping but check if `bit` has borrow
    /// akin to overflowing_sub but over N elements and bit-specific
    fn bit_overflowing_sub(nums: &[Self], bit: usize) -> (Self, bool);
}

impl<T: PrimInt + OverflowingAdd + WrappingSub> NumTraitsExt for T {
    fn bitmask(bit: usize) -> Self {
        let max_bits = 8 * std::mem::size_of::<Self>();
        assert!(bit < max_bits);

        if bit == max_bits - 1 {
            Self::max_value()
        } else {
            (Self::one() << (bit + 1)) - Self::one()
        }
    }

    fn bit_overflowing_add(nums: &[Self], bit: usize) -> (Self, bool) {
        assert!(!nums.is_empty());

        let mask = Self::bitmask(bit);
        let mut acc = nums[0] & mask;
        let mut carry = false;

        for &x in &nums[1..] {
            let x_masked = x & mask;
            let (sum, overflow) = acc.overflowing_add(&x_masked);

            if (sum & !mask) != Self::zero() || overflow {
                carry = true;
            }

            acc = sum;
        }

        (acc, carry)
    }

    fn bit_overflowing_sub(nums: &[Self], bit: usize) -> (Self, bool) {
        assert!(!nums.is_empty());

        let mask = Self::bitmask(bit - 1);
        let mut acc = nums[0] & mask;
        let mut borrow = false;

        for &x in &nums[1..] {
            let x_masked = x & mask;

            if acc < x_masked {
                borrow = true;
            }

            acc = acc.wrapping_sub(&x_masked);
        }

        (acc, borrow)
    }
}

// TODO: const binary search
pub struct ConstMap<K: Eq, V, const N: usize>([(K, V); N]);

impl<K: Eq + Copy, V, const N: usize> ConstMap<K, V, N> {
    pub const fn new(map: [(K, V); N]) -> Self {
        //map.sort_by_key(|(k, _)| *k);
        ConstMap(map)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        // self.0
        //     .binary_search_by_key(key, |&(k, _)| k)
        //     .map(|idx| &self.0[idx].1)
        //     .ok()
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v) // TODO: why is `map(|(_, v)| v)` not const?
    }
}

pub trait Array2D<const W: usize, const H: usize> {
    type Item;

    fn get_2d(&self, idx: impl Into<usize>) -> &Self::Item;
    fn get_2d_mut(&mut self, idx: impl Into<usize>) -> &mut Self::Item;
    
    fn coords_1to2(idx: usize) -> (usize, usize) {
        if idx >= W * H {
            panic!("Index {idx} out of bounds for array {W}x{H}");
        }
        (idx / H, idx % H)
    }

    #[allow(unused)]
    fn coords_2to1((x, y): (usize, usize)) -> usize {
        if x >= W || y >= H {
            panic!("Index {x}x{y} out of bounds for array {W}x{H}");
        }
        x * H + y
    }

    fn set_2d(&mut self, idx: impl Into<usize>, val: Self::Item) {
        *self.get_2d_mut(idx) = val;
    }
}

impl<T, const W: usize, const H: usize> Array2D<W, H> for [[T; H]; W] {
    type Item = T;

    fn get_2d(&self, idx: impl Into<usize>) -> &T {
        let (x, y) = Self::coords_1to2(idx.into());
        &self[x][y]
    }

    fn get_2d_mut(&mut self, idx: impl Into<usize>) -> &mut T {
        let (x, y) = Self::coords_1to2(idx.into());
        &mut self[x][y]
    }
}

pub trait Array3D<const W: usize, const H: usize, const D: usize> {
    type Item;

    fn get_3d(&self, idx: impl Into<usize>) -> &Self::Item;
    fn get_3d_mut(&mut self, idx: impl Into<usize>) -> &mut Self::Item;
    
    fn coords_1to3(idx: usize) -> (usize, usize, usize) {
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
    fn coords_3to1((x, y, z): (usize, usize, usize)) -> usize {
        if x >= W || y >= H || z >= D {
            panic!("Index {x}x{y}x{z} out of bounds for array {W}x{H}x{D}");
        }
        x * H * D + y * D + z
    }

    fn set_3d(&mut self, idx: impl Into<usize>, val: Self::Item) {
        *self.get_3d_mut(idx) = val;
    }
}

impl<T, const W: usize, const H: usize, const D: usize> Array3D<W, H, D>
    for [[[T; D]; H]; W]
{
    type Item = T;

    fn get_3d(&self, idx: impl Into<usize>) -> &T {
        let (x, y, z) = Self::coords_1to3(idx.into());
        &self[x][y][z]
    }

    fn get_3d_mut(&mut self, idx: impl Into<usize>) -> &mut T {
        let (x, y, z) = Self::coords_1to3(idx.into());
        &mut self[x][y][z]
    }
}