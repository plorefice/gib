use std::fs;

use gib_core::GameBoy;

macro_rules! test_cases {
    (
        $(
            $name:ident($path:expr) $seconds:expr;
        )+
    ) => {
        $(
            #[test]
            fn $name() {
                run_test($path, $seconds);
            }
        )+
    };
}

test_cases! {
    cpu_instrs("cpu_instrs/cpu_instrs") 55;
    instr_timing("instr_timing/instr_timing") 1;
    mem_timing("mem_timing/mem_timing") 3;
    mem_timing_2("mem_timing-2/mem_timing") 4;
    halt_bug("halt_bug") 2;
    dmg_sound("dmg_sound/dmg_sound") 40;
}

fn run_test(name: &str, seconds: u64) {
    let path = format!("assets/roms/blargg/{name}");

    let rom = fs::read(format!("{path}.gb")).expect("failed to load test binary");

    let image = image::ImageReader::open(format!("{path}-dmg.png"))
        .or_else(|_| image::ImageReader::open(format!("{path}-dmg-cgb.png")))
        .expect("screenshot not found")
        .decode()
        .expect("invalid screenshot format");

    let mut gameboy = GameBoy::new();
    gameboy.load_rom(&rom).unwrap();

    let emulated_cycles = seconds * gib_core::CPU_CLOCK;

    while gameboy.clock_cycles() < emulated_cycles {
        gameboy.run_for_vblank().expect("unexpected trace event");
    }

    let mut buffer = vec![0xff; 160 * 144 * 4];
    gameboy.rasterize(&mut buffer);

    assert_eq!(buffer, image.to_rgba8().to_vec());
}
