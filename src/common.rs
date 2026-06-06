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
