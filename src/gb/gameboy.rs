use super::bus::Bus;
use super::cpu::CPU;
use super::dbg;

const CPU_CLOCK: u64 = 4_194_304; // Hz
const HSYNC_CLOCK: u64 = 9_198; // Hz

const CYCLES_PER_HSYNC: u64 = CPU_CLOCK / HSYNC_CLOCK;

pub struct GameBoy {
    cpu: CPU,
    bus: Bus,
}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy {
            cpu: CPU::new(),
            bus: Bus::new(),
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), dbg::TraceEvent> {
        self.bus.load_rom(rom)
    }

    pub fn step(&mut self) -> Result<(), dbg::TraceEvent> {
        if self.cpu.intr_enabled {
            self.run_irqs()?;
        }

        let elapsed = {
            let clk = self.cpu.clk;

            if !self.cpu.halted {
                self.cpu.exec(&mut self.bus)?;
            } else {
                self.cpu.clk += 4;
            }
            self.cpu.clk - clk
        };

        self.bus.ppu.tick(elapsed);

        if self.bus.tim.tick(elapsed) {
            self.bus.itr.ifg.set_bit(2);
        }

        Ok(())
    }

    fn run_irqs(&mut self) -> Result<(), dbg::TraceEvent> {
        let mut req = || {
            let itr = &mut self.bus.itr;

            for req_id in 0..=4 {
                if itr.ien.bit(req_id) && itr.ifg.bit(req_id) {
                    // Disable interrupts when entering ISR
                    itr.ifg.clear_bit(req_id);
                    self.cpu.intr_enabled = false;

                    // Address of the ISR to be run
                    return Some((0x40 + 0x08 * req_id) as u8);
                }
            }
            None
        };

        if let Some(r) = req() {
            // Execute RST instruction
            self.cpu.op(&mut self.bus, r)
        } else {
            Ok(())
        }
    }

    pub fn run_for_vblank(&mut self) -> Result<(), dbg::TraceEvent> {
        let until_clk = self.cpu.clk + CYCLES_PER_HSYNC * 154;

        while self.cpu.clk < until_clk {
            self.step()?;
        }

        Ok(())
    }

    pub fn rasterize(&self, vbuf: &mut [u8]) {
        self.bus.ppu.rasterize(vbuf);
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
