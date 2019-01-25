use super::bus::Bus;
use super::cpu::CPU;

const CPU_CLOCK: u64 = 4_194_304_000; // mHz
const VSYNC_CLOCK: u64 = 59_730; // mHz

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
        let until_clk = self.cpu.clk + u128::from(CPU_CLOCK / VSYNC_CLOCK);

        while self.cpu.clk < until_clk && !self.cpu.halted {
            self.cpu.exec(&mut self.bus);
        }
    }

    pub fn rasterize(&self, vbuf: &mut [u8]) {
        self.bus.ppu().rasterize(vbuf);
    }
}
