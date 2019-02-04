mod common;

use common::RomTest;

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
