use super::dbg;
use super::IoReg;
use super::{MemR, MemRW, MemSize, MemW};

#[derive(Default)]
pub struct Serial {
    _sb: IoReg,
    _sc: IoReg,
}

impl Serial {
    pub fn new() -> Serial {
        Serial::default()
    }
}

impl MemR for Serial {
    fn read<T: MemSize>(&self, _addr: u16) -> Result<T, dbg::TraceEvent> {
        // TODO: it's gonna be a while before serial link is implemented :)
        Ok(T::default())
    }
}

impl MemW for Serial {
    fn write<T: MemSize>(&mut self, _addr: u16, _val: T) -> Result<(), dbg::TraceEvent> {
        // TODO: it's gonna be a while before serial link is implemented :)
        Ok(())
    }
}

impl MemRW for Serial {}
