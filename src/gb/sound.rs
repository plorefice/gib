use super::bus::{MemR, MemSize, MemW};

pub struct APU;

impl APU {
    pub fn new() -> APU {
        APU
    }
}

impl<T: MemSize> MemR<T> for APU {
    fn read(&self, addr: u16) -> T {
        // TODO: it's gonna be a while before sound is implemented :)
        match addr {
            _ => T::default(),
        }
    }
}

impl<T: MemSize> MemW<T> for APU {
    fn write(&mut self, addr: u16, _val: T) {
        // TODO: it's gonna be a while before sound is implemented :)
        match addr {
            _ => (),
        }
    }
}
