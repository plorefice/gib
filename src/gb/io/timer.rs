use super::dbg;
use super::IoReg;
use super::{MemR, MemRW, MemSize, MemW};

// Timer tick rates in CPU clocks per tick
const DIV_RATE: u64 = 256;
const TIMA_01_RATE: u64 = 16;
const TIMA_10_RATE: u64 = 64;
const TIMA_11_RATE: u64 = 256;
const TIMA_00_RATE: u64 = 1024;

#[derive(Default)]
pub struct Timer {
    pub div: IoReg,
    pub tima: IoReg,
    pub tma: IoReg,
    pub tac: IoReg,

    div_elapsed_clks: u64,
    tima_elapsed_clks: u64,
}

impl Timer {
    pub fn new() -> Timer {
        Timer::default()
    }

    pub fn tick(&mut self, elapsed: u64) {
        self.tick_div(elapsed);
        self.tick_tima(elapsed);
    }

    fn tick_div(&mut self, elapsed: u64) {
        self.div.0 += ((self.div_elapsed_clks + elapsed) / DIV_RATE) as u8;
        self.div_elapsed_clks = (self.div_elapsed_clks + elapsed) % DIV_RATE;
    }

    fn tick_tima(&mut self, elapsed: u64) {
        // Do nothing if timer is disable
        if !self.tac.bit(2) {
            return;
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
            // TODO: an interrupt needs to be generated in this occasion
        }
    }

    fn reset_div(&mut self) {
        self.div.0 = 0;
        self.div_elapsed_clks = 0;
        self.tima_elapsed_clks = 0;
    }
}

impl MemR for Timer {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        if T::byte_size() != 1 {
            // Only single-byte access is supported
            Err(dbg::TraceEvent::IoFault(dbg::Peripheral::TIM, addr))
        } else {
            match addr {
                0xFF04 => T::read_le(&[self.div.0]),
                0xFF05 => T::read_le(&[self.tima.0]),
                0xFF06 => T::read_le(&[self.tma.0]),
                0xFF07 => T::read_le(&[self.tac.0]),
                _ => Err(dbg::TraceEvent::IoFault(dbg::Peripheral::TIM, addr)),
            }
        }
    }
}

impl MemW for Timer {
    fn write<T: MemSize>(&mut self, addr: u16, val: T) -> Result<(), dbg::TraceEvent> {
        if T::byte_size() != 1 {
            // Only single-byte access is supported
            Err(dbg::TraceEvent::IoFault(dbg::Peripheral::TIM, addr))
        } else {
            // Any write to DIV resets it to 0
            if addr == 0xFF04 {
                self.reset_div();
                return Ok(());
            }

            let dest = match addr {
                0xFF05 => &mut self.tima.0,
                0xFF06 => &mut self.tma.0,
                0xFF07 => &mut self.tac.0,
                _ => return Err(dbg::TraceEvent::IoFault(dbg::Peripheral::TIM, addr)),
            };

            let mut scratch = [*dest];
            T::write_le(&mut scratch[..], val)?;
            *dest = scratch[0];

            Ok(())
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
        assert_eq!(timer.div.0, 1);

        for _ in 0..128 {
            timer.tick(4);
        }
        assert_eq!(timer.div.0, 3);
    }

    #[test]
    fn div_reset() {
        let mut timer = Timer::default();

        for _ in 0..63 {
            timer.tick(4);
        }
        assert_eq!(timer.div.0, 0);

        timer.reset_div();
        assert_eq!(timer.div.0, 0);

        for _ in 0..63 {
            timer.tick(4);
        }
        assert_eq!(timer.div.0, 0);

        timer.tick(4);
        assert_eq!(timer.div.0, 1);
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
