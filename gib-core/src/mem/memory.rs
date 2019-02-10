use super::dbg;
use super::{MemR, MemRW, MemW};

#[derive(Clone)]
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

impl MemR for Memory {
    fn read(&self, addr: u16) -> Result<u8, dbg::TraceEvent> {
        Ok(self.data[usize::from(addr)])
    }
}

impl MemW for Memory {
    fn write(&mut self, addr: u16, val: u8) -> Result<(), dbg::TraceEvent> {
        self.data[usize::from(addr)] = val;
        Ok(())
    }
}

impl MemRW for Memory {}
