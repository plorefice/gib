extern crate gb_rs;

use gb_rs::GameBoy;

pub struct RomTest {
    gb: GameBoy,
    rom: &'static [u8],
}

impl RomTest {
    pub fn new(rom: &'static [u8]) -> RomTest {
        RomTest {
            gb: GameBoy::new(),
            rom,
        }
    }

    pub fn must_run_and_match(&mut self, until: u64, output: &'static [u8]) {
        let mut vbuf = vec![0xFF; 160 * 144 * 4];

        self.gb.load_rom(&self.rom[..]).unwrap();

        while self.gb.cpu().clk < until {
            self.gb.step().unwrap();
        }
        self.gb.rasterize(&mut vbuf[..]);

        assert_eq!(&vbuf[..], &output[..]);
    }
}
