use super::dbg;
use super::{InterruptSource, IoReg};
use super::{MemR, MemRW, MemSize, MemW};

pub struct Timer {
    pub sys_counter: IoReg<u16>,
    pub tima: IoReg<u8>,
    pub tma: IoReg<u8>,
    pub tac: IoReg<u8>,

    irq_pending: bool,
}

impl Default for Timer {
    fn default() -> Timer {
        Timer {
            sys_counter: IoReg(0),
            tima: IoReg(0),
            tma: IoReg(0),
            tac: IoReg(0),

            irq_pending: false,
        }
    }
}

impl Timer {
    pub fn new() -> Timer {
        Timer::default()
    }

    pub fn div(&self) -> IoReg<u8> {
        IoReg((self.sys_counter.0 >> 8) as u8)
    }

    pub fn tick(&mut self, mut elapsed: u64) {
        let rb = self.curr_rate();

        if !self.running() {
            self.sys_counter.0 += elapsed as u16;
        } else {
            while elapsed > 0 {
                let n = 8u64.min(elapsed) as u8;

                let old = self.sys_counter;
                self.sys_counter.0 += u16::from(n);
                let new = self.sys_counter;

                // TIMA is incremented when a falling edge is detected on the rate bit.
                if old.bit(rb) && !new.bit(rb) {
                    self.inc_timer();
                }
                elapsed -= u64::from(n);
            }
        }
    }

    pub fn running(&self) -> bool {
        self.tac.bit(2)
    }

    fn inc_timer(&mut self) {
        self.tima.0 += 1;

        // Wehn TIMA overflows, TMA gets loaded in it and an IRQ request is registered.
        if self.tima.0 == 0 {
            self.tima = self.tma;
            self.irq_pending = true;
        }
    }

    fn reset_sys_counter(&mut self) {
        // HW BUG: resetting DIV while the multiplexer bit corresponding
        // to the current tick rate is set causes TIMA to increment.
        if self.running() && self.rate_bit() {
            self.inc_timer()
        }
        self.sys_counter.0 = 0;
    }

    fn write_to_tac<T: MemSize>(&mut self, val: T) {
        let val = IoReg(val.low());

        // HW BUG: when changing TAC register value, if the old selected bit
        // by the multiplexer was 0, the new one is 1, and the new enable bit
        // of TAC is set to 1, it will increase TIMA.
        let c1 = val.bit(2) && !self.rate_bit() && self.sys_counter.bit(Timer::rate_of(val));

        // HW BUG: whenever half the clocks of the count are reached,
        // TIMA will increase when disabling the timer.
        let c2 = self.running() && !val.bit(2) && self.rate_bit();

        if c1 || c2 {
            self.inc_timer();
        }
        self.tac = val;
    }

    fn curr_rate(&self) -> usize {
        Timer::rate_of(self.tac)
    }

    fn rate_bit(&self) -> bool {
        self.sys_counter.bit(self.curr_rate())
    }

    fn rate_of(r: IoReg<u8>) -> usize {
        match r.0 & 0x3 {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            0b11 => 7,
            _ => unreachable!(),
        }
    }
}

impl InterruptSource for Timer {
    fn irq_pending(&self) -> bool {
        self.irq_pending
    }
}

impl MemR for Timer {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        match addr {
            0xFF04 => T::read_le(&[self.div().0]),
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
                self.reset_sys_counter();
                Ok(())
            }
            0xFF05 => T::write_mut_le(&mut [&mut self.tima.0], val),
            0xFF06 => T::write_mut_le(&mut [&mut self.tma.0], val),
            0xFF07 => {
                self.write_to_tac(val);
                Ok(())
            }
            _ => Err(dbg::TraceEvent::IoFault(dbg::Peripheral::TIM, addr)),
        }
    }
}

impl MemRW for Timer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_counter_tick() {
        let mut timer = Timer::default();

        // Counter starts at 0
        assert_eq!(timer.div().0, 0);

        // A whole tick happens every 256 clock cycles
        timer.tick(255);
        assert_eq!(timer.sys_counter.0, 255);
        assert_eq!(timer.div().0, 0);

        timer.tick(1);
        assert_eq!(timer.sys_counter.0, 256);
        assert_eq!(timer.div().0, 1);

        // It can handle any kind of step
        timer.tick(12);
        assert_eq!(timer.sys_counter.0, 268);
        assert_eq!(timer.div().0, 1);

        timer.tick(256 * 4);
        assert_eq!(timer.div().0, 5);
    }

    #[test]
    fn system_counter_reset() {
        let mut timer = Timer::default();

        timer.tick(516);
        assert_eq!(timer.sys_counter.0, 516);

        timer.reset_sys_counter();
        assert_eq!(timer.sys_counter.0, 0);

        timer.tick(4);
        assert_eq!(timer.sys_counter.0, 4);
    }

    #[test]
    fn timer_tick() {
        let mut timer = Timer::default();

        // Ticking does not affect a stopped timer
        timer.tick(2048);
        assert_eq!(timer.tima.0, 0);

        // Enabling the timer on a even system counter
        // makes them synchronized.
        timer.write_to_tac(0b101_u8);

        timer.tick(15);
        assert_eq!(timer.tima.0, 0);
        timer.tick(1);
        assert_eq!(timer.tima.0, 1);

        timer.tick(65);
        assert_eq!(timer.tima.0, 5);
    }

    #[test]
    fn replicate_timer_hw_bugs() {
        // Test 1: when writing to DIV register the TIMA register can be increased
        // if the counter has reached half the clocks it needs to increase.
        let mut timer = Timer::default();
        timer.tac.0 = 0b101;

        timer.tick(12);
        assert_eq!(timer.tima.0, 0);
        timer.reset_sys_counter();
        assert_eq!(timer.tima.0, 1);
    }
}
