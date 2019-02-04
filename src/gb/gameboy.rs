use super::bus::Bus;
use super::cpu::CPU;
use super::dbg;
use super::io::InterruptSource;

const CPU_CLOCK: u64 = 4_194_304; // Hz
const HSYNC_CLOCK: u64 = 9_198; // Hz

const CYCLES_PER_HSYNC: u64 = CPU_CLOCK / HSYNC_CLOCK;

pub struct GameBoy {
    cpu: CPU,
    bus: Bus,
    cycles: u64,
}

impl Default for GameBoy {
    fn default() -> GameBoy {
        GameBoy {
            cpu: CPU::new(),
            bus: Bus::new(),
            cycles: 0,
        }
    }
}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy::default()
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), dbg::TraceEvent> {
        self.bus.load_rom(rom)
    }

    pub fn step(&mut self) -> Result<(), dbg::TraceEvent> {
        if !self.cpu.executing {
            self.handle_irqs()?;
        }

        self.tick()?;
        while self.cpu.executing {
            self.tick()?;
        }

        Ok(())
    }

    fn tick(&mut self) -> Result<(), dbg::TraceEvent> {
        if !self.cpu.halted {
            self.cpu.tick(&mut self.bus)?;

            // Section 4.10 of "The Cycle-Accurate GameBoy Docs"
            // =================================================
            // The HALT bug triggers if a HALT instruction is executed when IME = 0 && (IE & IF) != 0.
            // In this case, the CPU is NOT halted, and the HALT bug is triggered, causing the PC
            // to NOT be incremented when the next instruction is executed (ie. the next instruction
            // is executed twice).
            if self.cpu.should_halt {
                self.cpu.should_halt = false;

                if self.cpu.intr_enabled || !self.bus.itr.pending_irqs() {
                    self.cpu.halted = true;
                } else {
                    self.cpu.halt_bug = true;
                }
            }
        }

        self.bus.ppu.tick();
        self.bus.tim.tick();

        self.cycles += 4;

        Ok(())
    }

    fn handle_irqs(&mut self) -> Result<(), dbg::TraceEvent> {
        // Fetch interrupt requests from interrupt sources
        if self.bus.tim.irq_pending() {
            self.bus.itr.set_irq(2);
        }

        if let Some(id) = self.bus.itr.get_pending_irq() {
            let addr = (0x40 + 0x08 * id) as u16;

            self.cpu.halted = false;

            // If IME = 1, disable HALT mode (if in it), set IME = 0,
            // clear IF and run the corresponding ISR.
            // If IME = 0, simply leave HALT mode.
            if self.cpu.intr_enabled {
                self.cpu.intr_enabled = false;

                self.bus.itr.clear_irq(id);
                self.cpu.jump_to_isr(&mut self.bus, addr)?;
            }
        }
        Ok(())
    }

    pub fn run_for_vblank(&mut self) -> Result<(), dbg::TraceEvent> {
        let until = self.cycles + (CYCLES_PER_HSYNC * 154);

        while self.cycles < until {
            self.step()?;
        }
        Ok(())
    }

    pub fn rasterize(&self, vbuf: &mut [u8]) {
        self.bus.ppu.rasterize(vbuf);
    }

    pub fn clock_cycles(&self) -> u64 {
        self.cycles
    }

    pub fn cpu(&self) -> &CPU {
        &self.cpu
    }

    pub fn cpu_mut(&mut self) -> &mut CPU {
        &mut self.cpu
    }

    pub fn bus(&self) -> &Bus {
        &self.bus
    }
}
