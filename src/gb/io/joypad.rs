use super::dbg;
use super::IoReg;
use super::{MemR, MemRW, MemSize, MemW};

pub struct Joypad {
    reg: IoReg<u8>,
}

impl Default for Joypad {
    fn default() -> Joypad {
        Joypad { reg: IoReg(0x0F) }
    }
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad::default()
    }
}

impl MemR for Joypad {
    fn read<T: MemSize>(&self, _addr: u16) -> Result<T, dbg::TraceEvent> {
        // TODO: Soon™ :)
        T::read_le(&[self.reg.0 | 0xC0][..])
    }
}

impl MemW for Joypad {
    fn write<T: MemSize>(&mut self, _addr: u16, _val: T) -> Result<(), dbg::TraceEvent> {
        // TODO: Soon™ :)
        Ok(())
    }
}

impl MemRW for Joypad {}
