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
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        // TODO: it's gonna be a while before sound is implemented :)
        match addr {
            0xFF10 => T::read_le(&[0x80]),
            0xFF14 => T::read_le(&[0x38]),
            0xFF19 => T::read_le(&[0x38]),
            0xFF1A => T::read_le(&[0x7F]),
            0xFF1C => T::read_le(&[0x9F]),
            0xFF1E => T::read_le(&[0x38]),
            0xFF20 => T::read_le(&[0xC0]),
            0xFF23 => T::read_le(&[0x3F]),
            0xFF26 => T::read_le(&[0x70]),
            _ => T::read_le(&[0xFF]),
        }
    }
}

impl MemW for APU {
    fn write<T: MemSize>(&mut self, _addr: u16, _val: T) -> Result<(), dbg::TraceEvent> {
        // TODO: it's gonna be a while before sound is implemented :)
        Ok(())
    }
}
