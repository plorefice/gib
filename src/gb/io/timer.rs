use super::dbg;
use super::IoReg;
use super::{MemR, MemRW, MemSize, MemW};

// Timer tick rates in CPU clocks per tick
const TIMA_01_RATE: u64 = 16;
const TIMA_10_RATE: u64 = 64;
const TIMA_11_RATE: u64 = 256;
const TIMA_00_RATE: u64 = 1024;

#[derive(Default)]
pub struct Timer {
    pub div: IoReg<u16>,
    pub tima: IoReg<u8>,
    pub tma: IoReg<u8>,
    pub tac: IoReg<u8>,

    tima_elapsed_clks: u64,
}

impl Timer {
    pub fn new() -> Timer {
        Timer::default()
    }

    pub fn div(&self) -> u8 {
        (self.div.0 >> 8) as u8
    }

    pub fn tick(&mut self, elapsed: u64) -> bool {
        self.tick_div(elapsed);
        self.tick_tima(elapsed)
    }

    fn tick_div(&mut self, elapsed: u64) {
        self.div.0 += elapsed as u16;
    }

    fn tick_tima(&mut self, elapsed: u64) -> bool {
        // Do nothing if timer is disable
        if !self.tac.bit(2) {
            return false;
        }

        let rate = match self.tac.0 & 0x3 {
            0b00 => TIMA_00_RATE,
            0b01 => TIMA_01_RATE,
            0b10 => TIMA_10_RATE,
            0b11 => TIMA_11_RATE,
            _ => unreachable!(),
        };

        let old_tima = self.tima.0;
        self.tima.0 += ((self.tima_elapsed_clks + elapsed) / rate) as u8;
        self.tima_elapsed_clks = (self.tima_elapsed_clks + elapsed) % rate;

        // Reload with TMA when TIMA overflows
        if old_tima > self.tima.0 {
            self.tima = self.tma;
            true
        } else {
            false
        }
    }

    fn reset_div(&mut self) {
        self.div.0 = 0;
        self.tima_elapsed_clks = 0;
    }
}

impl MemR for Timer {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        match addr {
            0xFF04 => T::read_le(&[self.div()]),
            0xFF05 => T::read_le(&[self.tima.0]),
            0xFF06 => T::read_le(&[self.tma.0]),
            0xFF07 => T::read_le(&[self.tac.0]),
            _ => Err(dbg::TraceEvent::IoFault(dbg::Peripheral::TIM, addr)),
        }
    }
}

impl MemW for Timer {
    fn write<T: MemSize>(&mut self, addr: u16, val: T) -> Result<(), dbg::TraceEvent> {
        match addr {
            0xFF04 => {
                self.reset_div();
                Ok(())
            }
            0xFF05 => T::write_mut_le(&mut [&mut self.tima.0], val),
            0xFF06 => T::write_mut_le(&mut [&mut self.tma.0], val),
            0xFF07 => T::write_mut_le(&mut [&mut self.tac.0], val),
            _ => Err(dbg::TraceEvent::IoFault(dbg::Peripheral::TIM, addr)),
        }
    }
}

impl MemRW for Timer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn div_tick() {
        let mut timer = Timer::default();

        for _ in 0..64 {
            timer.tick(4);
        }
        assert_eq!(timer.div(), 1);

        for _ in 0..128 {
            timer.tick(4);
        }
        assert_eq!(timer.div(), 3);
    }

    #[test]
    fn div_reset() {
        let mut timer = Timer::default();

        for _ in 0..63 {
            timer.tick(4);
        }
        assert_eq!(timer.div(), 0);

        timer.reset_div();
        assert_eq!(timer.div(), 0);

        for _ in 0..63 {
            timer.tick(4);
        }
        assert_eq!(timer.div(), 0);

        timer.tick(4);
        assert_eq!(timer.div(), 1);
    }

    #[test]
    fn tima_tick() {
        let mut timer = Timer::default();
        timer.tac.0 = 0x07;

        for _ in 0..63 {
            timer.tick(4);
        }
        assert_eq!(timer.tima.0, 0);

        timer.tick(4);
        assert_eq!(timer.tima.0, 1);

        let mut timer = Timer::default();
        timer.tac.0 = 0x05;

        timer.tick(64);
        assert_eq!(timer.tima.0, 4);
    }
}
