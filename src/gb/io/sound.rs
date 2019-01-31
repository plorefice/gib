use super::dbg;
use super::{MemR, MemSize, MemW};

pub struct APU;

impl APU {
    pub fn new() -> APU {
        APU
    }
}

impl MemR for APU {
    fn read<T: MemSize>(&self, _addr: u16) -> Result<T, dbg::TraceEvent> {
        // TODO: it's gonna be a while before sound is implemented :)
        Ok(T::default())
    }
}

impl MemW for APU {
    fn write<T: MemSize>(&mut self, _addr: u16, _val: T) -> Result<(), dbg::TraceEvent> {
        // TODO: it's gonna be a while before sound is implemented :)
        Ok(())
    }
}
