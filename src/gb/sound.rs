use super::bus::{MemR, MemSize, MemW};

pub struct APU;

impl APU {
    pub fn new() -> APU {
        APU
    }
}

impl MemR for APU {
    fn read<T: MemSize>(&self, addr: u16) -> T {
        // TODO: it's gonna be a while before sound is implemented :)
        match addr {
            _ => T::default(),
        }
    }
}

impl MemW for APU {
    fn write<T: MemSize>(&mut self, addr: u16, _val: T) {
        // TODO: it's gonna be a while before sound is implemented :)
        match addr {
            _ => (),
        }
    }
}
