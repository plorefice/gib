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
