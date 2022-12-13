use crate::{
    dbg,
    io::IoReg,
    mem::{MemR, MemRW, MemW},
};

/// Possible sources of interrupt in the system
pub enum IrqSource {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

impl From<IrqSource> for usize {
    fn from(irq: IrqSource) -> Self {
        match irq {
            IrqSource::VBlank => 0,
            IrqSource::LcdStat => 1,
            IrqSource::Timer => 2,
            IrqSource::Serial => 3,
            IrqSource::Joypad => 4,
        }
    }
}

pub trait InterruptSource {
    fn get_and_clear_irq(&mut self) -> Option<IrqSource>;
}

#[derive(Default)]
pub struct IrqController {
    pub ien: IoReg<u8>,
    pub ifg: IoReg<u8>,
}

impl IrqController {
    pub fn new() -> IrqController {
        IrqController::default()
    }

    pub fn pending_irqs(&self) -> bool {
        self.get_pending_irq().is_some()
    }

    pub fn get_pending_irq(&self) -> Option<usize> {
        (0..=4).find(|&req_id| self.ien.bit(req_id) && self.ifg.bit(req_id))
    }

    pub fn set_irq(&mut self, irq: usize) {
        self.ifg.set_bit(irq);
    }

    pub fn clear_irq(&mut self, irq: usize) {
        self.ifg.clear_bit(irq);
    }
}

impl MemR for IrqController {
    fn read(&self, addr: u16) -> Result<u8, dbg::TraceEvent> {
        Ok(match addr {
            0xFF0F => self.ifg.0 | 0xE0,
            0xFFFF => self.ien.0,
            _ => unreachable!(),
        })
    }
}

impl MemW for IrqController {
    fn write(&mut self, addr: u16, val: u8) -> Result<(), dbg::TraceEvent> {
        match addr {
            0xFF0F => self.ifg.0 = val,
            0xFFFF => self.ien.0 = val,
            _ => unreachable!(),
        };
        Ok(())
    }
}

impl MemRW for IrqController {}
