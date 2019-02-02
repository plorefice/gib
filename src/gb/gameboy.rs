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
        self.bus.tim.tick(elapsed);

        Ok(())
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
