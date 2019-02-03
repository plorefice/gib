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

    pub fn pending_irqs(&self) -> bool {
        (self.ien.0 & self.ifg.0) != 0
    }

    pub fn get_pending_irq(&self) -> Option<usize> {
        for req_id in 0..=4 {
            if self.ien.bit(req_id) && self.ifg.bit(req_id) {
                return Some(req_id);
            }
        }
        None
    }

    pub fn clear_irq(&mut self, irq: usize) {
        self.ifg.clear_bit(irq);
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
