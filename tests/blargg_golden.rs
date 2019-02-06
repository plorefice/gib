mod common;

use common::RomTest;

#[test]
fn passes_blargg_cpu_instrs() {
    RomTest::new(include_bytes!("../roms/blargg/cpu_instrs.gb"))
        .must_run_and_match(225_000_000u64, include_bytes!("blargg/cpu_instrs.bin"));
}

#[test]
fn passes_blargg_instr_timing() {
    RomTest::new(include_bytes!("../roms/blargg/instr_timing.gb"))
        .must_run_and_match(3_000_000u64, include_bytes!("blargg/instr_timing.bin"));
}
