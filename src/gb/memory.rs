use super::bus::{MemR, MemSize, MemW};

pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(size: u16) -> Memory {
        Memory {
            data: vec![0; usize::from(size)],
        }
    }
}

impl<T: MemSize> MemR<T> for Memory {
    fn read(&self, addr: u16) -> T {
        T::read_le(&self.data[usize::from(addr)..])
    }
}

impl<T: MemSize> MemW<T> for Memory {
    fn write(&mut self, addr: u16, val: T) {
        T::write_le(&mut self.data[usize::from(addr)..], val)
    }
}
