use std::ops::{BitAnd, BitAndAssign, BitOrAssign, Not, Shl};

pub trait Zero {
    fn zero() -> Self;
}

impl Zero for u8 {
    fn zero() -> u8 {
        0
    }
}

impl Zero for u16 {
    fn zero() -> u16 {
        0
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct IoReg<T>(pub T);

impl<T> IoReg<T>
where
    T: PartialEq
        + From<bool>
        + Zero
        + Not<Output = T>
        + Shl<usize, Output = T>
        + BitAnd<T, Output = T>
        + BitAndAssign<T>
        + BitOrAssign<T>,
{
    pub fn bit(self, b: usize) -> bool {
        (self.0 & (T::from(true) << b)) != T::zero()
    }

    pub fn set_bit(&mut self, b: usize) {
        self.0 |= T::from(true) << b;
    }

    pub fn clear_bit(&mut self, b: usize) {
        self.0 &= !(T::from(true) << b);
    }

    // pub fn toggle_bit(&mut self, b: usize) {
    //     debug_assert!(b < 8);
    //     self.0 ^= 1 << b;
    // }

    // pub fn put_bit(&mut self, b: usize, v: bool) {
    //     if v {
    //         self.set_bit(b);
    //     } else {
    //         self.clear_bit(b);
    //     }
    // }
}

pub trait InterruptSource {
    fn irq_pending(&self) -> bool;
}
