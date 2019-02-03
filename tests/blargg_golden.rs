extern crate gb_rs;

use gb_rs::GameBoy;

struct RomTest {
    gb: GameBoy,
    rom: &'static [u8],
}

impl RomTest {
    fn new(rom: &'static [u8]) -> RomTest {
        RomTest {
            gb: GameBoy::new(),
            rom,
        }
    }

    fn must_run_and_match(&mut self, until: u64, output: &'static [u8]) {
        let mut vbuf = vec![0xFF; 160 * 144 * 4];

        self.gb.load_rom(&self.rom[..]).unwrap();

        while self.gb.cpu().clk < until {
            self.gb.step().unwrap();
        }
        self.gb.rasterize(&mut vbuf[..]);

        assert_eq!(&vbuf[..], &output[..]);
    }
}

#[test]
fn passes_blargg_cpu_instrs() {
    RomTest::new(include_bytes!("../roms/blargg/cpu_instrs.gb"))
        .must_run_and_match(261_175_572u64, include_bytes!("blargg/cpu_instrs.bin"));
}

#[test]
fn passes_blargg_instr_timing() {
    RomTest::new(include_bytes!("../roms/blargg/instr_timing.gb"))
        .must_run_and_match(29_566_032u64, include_bytes!("blargg/instr_timing.bin"));
}
