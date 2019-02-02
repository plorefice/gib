use super::dbg::{self, Peripheral};
use super::{IoReg, MemR, MemRW, MemSize, MemW};

#[derive(Default)]
pub struct IrqController {
    pub ien: IoReg,
    pub ifg: IoReg,
}

impl IrqController {
    pub fn new() -> IrqController {
        IrqController::default()
    }
}

impl MemR for IrqController {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        match addr {
            0xFF0F => T::read_le(&[self.ifg.0]),
            0xFFFF => T::read_le(&[self.ien.0]),
            _ => Err(dbg::TraceEvent::IoFault(Peripheral::ITR, addr)),
        }
    }
}

impl MemW for IrqController {
    fn write<T: MemSize>(&mut self, addr: u16, val: T) -> Result<(), dbg::TraceEvent> {
        match addr {
            0xFF0F => T::write_mut_le(&mut [&mut self.ifg.0], val),
            0xFFFF => T::write_mut_le(&mut [&mut self.ien.0], val),
            _ => Err(dbg::TraceEvent::IoFault(Peripheral::ITR, addr)),
        }
    }
}

impl MemRW for IrqController {}
