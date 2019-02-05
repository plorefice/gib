mod common;

use common::RomTest;

/*
 * Gekkio's COMMON acceptance tests
 */

#[test]
fn passes_gekkio_acceptance_boot_div() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/boot_div-dmgABCmgb.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/boot_div-dmgABCmgb.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_boot_regs() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/boot_regs-dmgABC.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/boot_regs-dmgABC.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_div_timing() {
    RomTest::new(include_bytes!("../roms/gekkio/acceptance/div_timing.gb")).must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/div_timing.bin"),
    );
}

/*
 * Gekkio's BITS acceptance tests
 */

#[test]
fn passes_gekkio_acceptance_bits_mem_oam() {
    RomTest::new(include_bytes!("../roms/gekkio/acceptance/bits/mem_oam.gb")).must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/bits/mem_oam.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_bits_reg_f() {
    RomTest::new(include_bytes!("../roms/gekkio/acceptance/bits/reg_f.gb")).must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/bits/reg_f.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_bits_unused_hwio() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/bits/unused_hwio-GS.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/bits/unused_hwio-GS.bin"),
    );
}

/*
 * Gekkio's TIMER acceptance tests
 */

#[test]
fn passes_gekkio_acceptance_timer_div_write() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/timer/div_write.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/div_write.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_rapid_toggle() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/timer/rapid_toggle.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/rapid_toggle.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_tim00() {
    RomTest::new(include_bytes!("../roms/gekkio/acceptance/timer/tim00.gb")).must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim00.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_tim00_div_trigger() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/timer/tim00_div_trigger.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim00_div_trigger.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_tim01() {
    RomTest::new(include_bytes!("../roms/gekkio/acceptance/timer/tim01.gb")).must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim01.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_tim01_div_trigger() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/timer/tim01_div_trigger.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim01_div_trigger.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_tim10() {
    RomTest::new(include_bytes!("../roms/gekkio/acceptance/timer/tim10.gb")).must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim10.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_tim10_div_trigger() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/timer/tim10_div_trigger.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim10_div_trigger.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_tim11() {
    RomTest::new(include_bytes!("../roms/gekkio/acceptance/timer/tim11.gb")).must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim11.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_tim11_div_trigger() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/timer/tim11_div_trigger.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim11_div_trigger.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_tima_reload() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/timer/tima_reload.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tima_reload.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_tima_write_reloading() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/timer/tima_write_reloading.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tima_write_reloading.bin"),
    );
}

#[test]
fn passes_gekkio_acceptance_timer_tma_write_reloading() {
    RomTest::new(include_bytes!(
        "../roms/gekkio/acceptance/timer/tma_write_reloading.gb"
    ))
    .must_run_and_match(
        30_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tma_write_reloading.bin"),
    );
}
