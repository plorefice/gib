use super::dbg::{self, Peripheral};
use super::{IoReg, MemR, MemRW, MemSize, MemW};

#[derive(Default)]
pub struct IrqController {
    ien: IoReg,
    ifg: IoReg,
}

impl IrqController {
    pub fn new() -> IrqController {
        IrqController::default()
    }
}

impl MemR for IrqController {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        if T::byte_size() != 1 {
            Err(dbg::TraceEvent::AccessFault)
        } else {
            match addr {
                0xFF0F => T::read_le(&[self.ifg.0]),
                0xFFFF => T::read_le(&[self.ien.0]),
                _ => Err(dbg::TraceEvent::IoFault(Peripheral::ITR, addr)),
            }
        }
    }
}

impl MemW for IrqController {
    fn write<T: MemSize>(&mut self, addr: u16, val: T) -> Result<(), dbg::TraceEvent> {
        if T::byte_size() != 1 {
            Err(dbg::TraceEvent::AccessFault)
        } else {
            let dest = match addr {
                0xFF0F => &mut self.ifg.0,
                0xFFFF => &mut self.ien.0,
                _ => return Err(dbg::TraceEvent::IoFault(Peripheral::ITR, addr)),
            };

            let mut scratch = [*dest];
            T::write_le(&mut scratch[..], val)?;
            *dest = scratch[0];

            Ok(())
        }
    }
}

impl MemRW for IrqController {}
