use super::dbg;
use super::{MemR, MemRW, MemSize, MemW};

#[derive(Default)]
pub struct Joypad;

impl Joypad {
    pub fn new() -> Joypad {
        Joypad::default()
    }
}

impl MemR for Joypad {
    fn read<T: MemSize>(&self, _addr: u16) -> Result<T, dbg::TraceEvent> {
        // TODO: Soon™ :)
        T::read_le(&[0xFF][..])
    }
}

impl MemW for Joypad {
    fn write<T: MemSize>(&mut self, _addr: u16, _val: T) -> Result<(), dbg::TraceEvent> {
        // TODO: Soon™ :)
        Ok(())
    }
}

impl MemRW for Joypad {}
