use super::dbg;
use super::{InterruptSource, IrqSource};
use super::{MemR, MemSize, MemW};

pub struct APU;

impl Default for APU {
    fn default() -> APU {
        APU
    }
}

impl APU {
    pub fn new() -> APU {
        APU::default()
    }
}

impl InterruptSource for APU {
    fn get_and_clear_irq(&mut self) -> Option<IrqSource> {
        None
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
