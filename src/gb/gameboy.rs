use super::bus::Bus;
use super::cpu::CPU;

const CPU_CLOCK: u64 = 4_194_304; // Hz
const HSYNC_CLOCK: u64 = 9_198; // Hz

pub struct GameBoy {
    cpu: CPU,
    bus: Bus,
}

impl GameBoy {
    pub fn with_cartridge(rom: &[u8]) -> GameBoy {
        GameBoy {
            cpu: CPU::new(),
            bus: Bus::new(rom),
        }
    }

    pub fn run_to_vblank(&mut self) {
        for _ in 0..154 {
            let until_clk = self.cpu.clk + u128::from(CPU_CLOCK / HSYNC_CLOCK);

            while self.cpu.clk < until_clk && !self.cpu.halted {
                self.cpu.exec(&mut self.bus);
            }
            self.bus.ppu.hsync();
        }
    }

    pub fn rasterize(&self, vbuf: &mut [u8]) {
        self.bus.ppu.rasterize(vbuf);
    }
}
