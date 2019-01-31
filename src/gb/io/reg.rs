#[derive(Default, Copy, Clone, Debug)]
pub struct IoReg(pub u8);

impl IoReg {
    pub fn bit(&mut self, b: usize) -> bool {
        debug_assert!(b < 8);
        (self.0 & (1 << b)) != 0
    }

    pub fn set_bit(&mut self, b: usize) {
        debug_assert!(b < 8);
        self.0 |= 1 << b;
    }

    pub fn clear_bit(&mut self, b: usize) {
        debug_assert!(b < 8);
        self.0 &= !(1 << b);
    }

    pub fn toggle_bit(&mut self, b: usize) {
        debug_assert!(b < 8);
        self.0 ^= 1 << b;
    }

    pub fn put_bit(&mut self, b: usize, v: bool) {
        if v {
            self.set_bit(b);
        } else {
            self.clear_bit(b);
        }
    }
}
