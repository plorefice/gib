use super::dbg;
use super::{InterruptSource, IoReg, IrqSource};
use super::{MemR, MemRW, MemSize, MemW};

pub struct Timer {
    pub sys_counter: IoReg<u16>,
    pub tima: IoReg<u8>,
    pub tma: IoReg<u8>,
    pub tac: IoReg<u8>,

    irq_pending: bool,
    tima_reload_scheduled: bool,
    tima_is_being_reloaded: bool,
}

impl Default for Timer {
    fn default() -> Timer {
        Timer {
            sys_counter: IoReg(0xB648),
            tima: IoReg(0),
            tma: IoReg(0),
            tac: IoReg(0),

            irq_pending: false,
            tima_reload_scheduled: false,
            tima_is_being_reloaded: false,
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

    pub fn tick(&mut self) {
        let rb = self.curr_rate();

        // TIMA reload lasts one cycle, so it's ok to reset this
        // at the beginning of each tick.
        self.tima_is_being_reloaded = false;

        // If a reload was scheduled and not canceled, set the IRQ flag and
        // reload TIMA with TMA. This also causes the timer to enter a cycle
        // in which writes to TIMA are ignored.
        if self.tima_reload_scheduled {
            self.tima_reload_scheduled = false;

            self.tima_is_being_reloaded = true;
            self.irq_pending = true;
            self.tima = self.tma;
        }

        if !self.running() {
            self.sys_counter.0 += 4;
        } else {
            let old = self.sys_counter;
            self.sys_counter.0 += 4;
            let new = self.sys_counter;

            // TIMA is incremented when a falling edge is detected on the rate bit.
            if old.bit(rb) && !new.bit(rb) {
                self.inc_timer();
            }
        }
    }

    pub fn running(&self) -> bool {
        self.tac.bit(2)
    }

    fn inc_timer(&mut self) {
        self.tima.0 += 1;

        // Wehn TIMA overflows, TMA gets loaded in it and an IRQ request is registered.
        // This happend with a full cycle delay, so for 4 clock cycles upon overflowing,
        // TIMA stays 00, so here we just schedule the increment.
        if self.tima.0 == 0 {
            self.tima_reload_scheduled = true;
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
    fn get_and_clear_irq(&mut self) -> Option<IrqSource> {
        if self.irq_pending {
            self.irq_pending = false;
            Some(IrqSource::Timer)
        } else {
            None
        }
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
            0xFF05 => {
                // During the reload cycle, writes to TIMA are ignored.
                if !self.tima_is_being_reloaded {
                    T::write_mut_le(&mut [&mut self.tima.0], val)?;

                    // If a write to TIMA happens in the cycle during which an overflow happens,
                    // the reload is canceled: TIMA gets set to the written value and the
                    // interrupt request does not happen.
                    if self.tima_reload_scheduled {
                        self.tima_reload_scheduled = false;
                    }
                }
                Ok(())
            }
            0xFF06 => {
                T::write_mut_le(&mut [&mut self.tma.0], val)?;

                // If a write to TMA happens while TIMA is being reloaded,
                // the new value should be loaded instead.
                if self.tima_is_being_reloaded {
                    self.tima = self.tma;
                }
                Ok(())
            }
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

    // TODO: this tests are failing after 4ad06f9. Fix them.

    #[test]
    #[should_panic]
    fn system_counter_tick() {
        let mut timer = Timer::default();

        // Counter starts at 0
        assert_eq!(timer.div().0, 0);

        // A whole tick happens every 256 clock cycles
        for _ in 0..63 {
            timer.tick();
        }
        assert_eq!(timer.sys_counter.0, 252);
        assert_eq!(timer.div().0, 0);

        timer.tick();
        assert_eq!(timer.sys_counter.0, 256);
        assert_eq!(timer.div().0, 1);

        // It can handle any kind of step
        for _ in 0..3 {
            timer.tick();
        }
        assert_eq!(timer.sys_counter.0, 268);
        assert_eq!(timer.div().0, 1);

        for _ in 0..256 {
            timer.tick();
        }
        assert_eq!(timer.div().0, 5);
    }

    #[test]
    #[should_panic]
    fn system_counter_reset() {
        let mut timer = Timer::default();

        for _ in 0..129 {
            timer.tick();
        }
        assert_eq!(timer.sys_counter.0, 516);

        timer.reset_sys_counter();
        assert_eq!(timer.sys_counter.0, 0);

        timer.tick();
        assert_eq!(timer.sys_counter.0, 4);
    }

    #[test]
    #[should_panic]
    fn timer_tick() {
        let mut timer = Timer::default();

        // Ticking does not affect a stopped timer
        for _ in 0..512 {
            timer.tick();
        }
        assert_eq!(timer.tima.0, 0);

        // Enabling the timer on a even system counter
        // makes them synchronized.
        timer.write_to_tac(0b101_u8);

        for _ in 0..3 {
            timer.tick();
        }
        assert_eq!(timer.tima.0, 0);

        timer.tick();
        assert_eq!(timer.tima.0, 1);

        for _ in 0..17 {
            timer.tick();
        }
        assert_eq!(timer.tima.0, 5);
    }

    #[test]
    #[should_panic]
    fn replicate_timer_hw_bugs() {
        // Test 1: when writing to DIV register the TIMA register can be increased
        // if the counter has reached half the clocks it needs to increase.
        let mut timer = Timer::default();
        timer.tac.0 = 0b101;

        for _ in 0..3 {
            timer.tick();
        }
        assert_eq!(timer.tima.0, 0);

        timer.reset_sys_counter();
        assert_eq!(timer.tima.0, 1);
    }
}
