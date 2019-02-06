use super::dbg;
use super::IoReg;
use super::{InterruptSource, IrqSource};
use super::{MemR, MemRW, MemSize, MemW};

pub struct Serial {
    sb: IoReg<u8>,
    sc: IoReg<u8>,
}

impl Default for Serial {
    fn default() -> Serial {
        Serial {
            sb: IoReg(0x00),
            sc: IoReg(0x00),
        }
    }
}

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
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        // TODO: it's gonna be a while before serial link is implemented :)
        match addr {
            0xFF01 => T::read_le(&[self.sb.0]),
            0xFF02 => T::read_le(&[self.sc.0 | 0x7E]),
            _ => unreachable!(),
        }
    }
}

impl MemW for Serial {
    fn write<T: MemSize>(&mut self, addr: u16, val: T) -> Result<(), dbg::TraceEvent> {
        // TODO: it's gonna be a while before serial link is implemented :)
        match addr {
            0xFF01 => T::write_mut_le(&mut [&mut self.sb.0], val),
            0xFF02 => T::write_mut_le(&mut [&mut self.sc.0], val),
            _ => unreachable!(),
        }
    }
}

impl MemRW for Serial {}
