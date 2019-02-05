mod common;

use common::RomTest;

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
