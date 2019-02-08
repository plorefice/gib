use std::ops::{BitAnd, BitAndAssign, BitOrAssign, Not, Shl};

/// Blanket implementation of MemR/MemW/MemRW for a bitflags!-generated struct
macro_rules! mem_rw {
    ($reg:ident, $mask:expr) => {
        impl<'a> MemR for &'a $reg {
            fn read(&self, _addr: u16) -> Result<u8, dbg::TraceEvent> {
                Ok(self.bits() | $mask)
            }
        }

        impl<'a> MemR for &'a mut $reg {
            fn read(&self, addr: u16) -> Result<u8, dbg::TraceEvent> {
                (&*self as &$reg).read(addr)
            }
        }

        impl<'a> MemW for &'a mut $reg {
            fn write(&mut self, _addr: u16, val: u8) -> Result<(), dbg::TraceEvent> {
                self.bits = val;
                Ok(())
            }
        }

        impl<'a> MemRW for &'a mut $reg {}
    };
}

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

#[derive(Default, Copy, Clone, Debug, PartialEq)]
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

/// A Latch is a wrapper around a value that needs to be updated with a cycle delay.
///
/// When `load` is called on a Latch, the new value is not presented until `tick` is called.
/// A Latch also provides an asynchronous `reset` to override the latching mechanism.
#[derive(Copy, Clone)]
pub struct Latch<T: Copy + Clone>(T, T);

impl<T: Copy + Clone> Latch<T> {
    /// Create a new Latch loaded with `val`.
    pub fn new(val: T) -> Latch<T> {
        Latch(val, val)
    }

    /// Return the currently loaded value.
    pub fn loaded(&self) -> &T {
        &self.0
    }

    /// Return the current latched value.
    pub fn value(&self) -> &T {
        &self.1
    }

    /// Prepare `val` to be loaded at the next `tick` invocation.
    pub fn load(&mut self, val: T) {
        self.0 = val;
    }

    /// Immediately reset to current latched value to `val`.
    pub fn reset(&mut self, val: T) {
        self.0 = val;
        self.1 = val;
    }

    /// If a value was previously loaded, swap it with the current latched value.
    /// This does nothing if no value is loaded or no `load` was performed since the last `tick`.
    pub fn tick(&mut self) {
        self.1 = self.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latch_works() {
        let mut l = Latch::new(0xABBA_u16);
        assert_eq!(*l.value(), 0xABBA);
        assert_eq!(*l.loaded(), *l.value());

        // The value does not change immediately on load
        l.load(0xCAFE);
        assert_eq!(*l.loaded(), 0xCAFE);
        assert_eq!(*l.value(), 0xABBA);

        // It takes a single tick to change
        l.tick();
        assert_eq!(*l.value(), 0xCAFE);
        assert_eq!(*l.loaded(), *l.value());

        // Multiple loads result in the last value being used
        l.load(0x0A0A);
        l.load(0xBABA);
        l.tick();
        assert_eq!(*l.value(), 0xBABA);

        // Several ticks in a row have no effect
        l.tick();
        assert_eq!(*l.value(), 0xBABA);

        // Reset is asynchronous
        l.reset(0x55AA);
        assert_eq!(*l.value(), 0x55AA);

        // Reset also removes any loaded value
        l.load(0xBCDE);
        l.reset(0xCAEF);
        assert_eq!(*l.value(), 0xCAEF);
        l.tick();
        assert_eq!(*l.value(), 0xCAEF);
    }
}
