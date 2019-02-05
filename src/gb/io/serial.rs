use super::dbg;
use super::{InterruptSource, IrqSource};
use super::{MemR, MemRW, MemSize, MemW};

#[derive(Default)]
pub struct Serial;

impl Serial {
    pub fn new() -> Serial {
        Serial::default()
    }
}

impl InterruptSource for Serial {
    fn get_and_clear_irq(&mut self) -> Option<IrqSource> {
        None
    }
}

impl MemR for Serial {
    fn read<T: MemSize>(&self, _addr: u16) -> Result<T, dbg::TraceEvent> {
        // TODO: it's gonna be a while before serial link is implemented :)
        T::read_le(&[0xFF][..])
    }
}

impl MemW for Serial {
    fn write<T: MemSize>(&mut self, _addr: u16, _val: T) -> Result<(), dbg::TraceEvent> {
        // TODO: it's gonna be a while before serial link is implemented :)
        Ok(())
    }
}

impl MemRW for Serial {}
