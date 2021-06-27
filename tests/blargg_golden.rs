use common::RomTest;

mod common;

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

#[test]
fn passes_blargg_mem_timing() {
    RomTest::new(include_bytes!("../roms/blargg/mem_timing.gb"))
        .must_run_and_match(7_000_000u64, include_bytes!("blargg/mem_timing.bin"));
}

#[test]
fn passes_blargg_mem_timing_2() {
    RomTest::new(include_bytes!("../roms/blargg/mem_timing-2.gb"))
        .must_run_and_match(12_000_000u64, include_bytes!("blargg/mem_timing-2.bin"));
}

#[test]
fn passes_blargg_halt_bug() {
    RomTest::new(include_bytes!("../roms/blargg/halt_bug.gb"))
        .must_run_and_match(10_000_000u64, include_bytes!("blargg/halt_bug.bin"));
}

/*
 * dmg_sound-2 single ROMs
 */

#[test]
fn passes_blargg_dmg_sound_01_registers() {
    RomTest::new(include_bytes!("../roms/blargg/dmg_sound-2/01-registers.gb")).must_run_and_match(
        4_000_000u64,
        include_bytes!("blargg/dmg_sound-2/01-registers.bin"),
    )
}

#[test]
fn passes_blargg_dmg_sound_02_len_ctl() {
    RomTest::new(include_bytes!("../roms/blargg/dmg_sound-2/02-len ctr.gb")).must_run_and_match(
        40_000_000u64,
        include_bytes!("blargg/dmg_sound-2/02-len ctr.bin"),
    )
}

#[test]
fn passes_blargg_dmg_sound_04_sweep() {
    RomTest::new(include_bytes!("../roms/blargg/dmg_sound-2/04-sweep.gb")).must_run_and_match(
        6_000_000u64,
        include_bytes!("blargg/dmg_sound-2/04-sweep.bin"),
    )
}

#[test]
fn passes_blargg_dmg_sound_05_sweep_details() {
    RomTest::new(include_bytes!(
        "../roms/blargg/dmg_sound-2/05-sweep details.gb"
    ))
    .must_run_and_match(
        6_000_000u64,
        include_bytes!("blargg/dmg_sound-2/05-sweep details.bin"),
    )
}

#[test]
fn passes_blargg_dmg_sound_06_overflow_on_trigger() {
    RomTest::new(include_bytes!(
        "../roms/blargg/dmg_sound-2/06-overflow on trigger.gb"
    ))
    .must_run_and_match(
        6_000_000u64,
        include_bytes!("blargg/dmg_sound-2/06-overflow on trigger.bin"),
    )
}

#[test]
fn passes_blargg_dmg_sound_07_len_sweep_period_sync() {
    RomTest::new(include_bytes!(
        "../roms/blargg/dmg_sound-2/07-len sweep period sync.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("blargg/dmg_sound-2/07-len sweep period sync.bin"),
    )
}

#[test]
fn passes_blargg_dmg_sound_08_len_ctr_during_power() {
    RomTest::new(include_bytes!(
        "../roms/blargg/dmg_sound-2/08-len ctr during power.gb"
    ))
    .must_run_and_match(
        9_000_000u64,
        include_bytes!("blargg/dmg_sound-2/08-len ctr during power.bin"),
    )
}

#[test]
fn passes_blargg_dmg_sound_11_regs_after_power() {
    RomTest::new(include_bytes!(
        "../roms/blargg/dmg_sound-2/11-regs after power.gb"
    ))
    .must_run_and_match(
        6_000_000u64,
        include_bytes!("blargg/dmg_sound-2/11-regs after power.bin"),
    )
}
