use common::RomTest;

mod common;

/*
 * Gekkio's COMMON acceptance tests
 */

#[test]
fn gekkio_acceptance_add_sp_e_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/add_sp_e_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/add_sp_e_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_boot_div() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/boot_div-dmgABCmgb.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/boot_div-dmgABCmgb.bin"),
    );
}

#[test]
fn gekkio_acceptance_boot_regs() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/boot_regs-dmgABC.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/boot_regs-dmgABC.bin"),
    );
}

#[test]
fn gekkio_acceptance_boot_hwio() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/boot_hwio-dmgABCmgb.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/boot_hwio-dmgABCmgb.bin"),
    );
}

#[test]
fn gekkio_acceptance_call_cc_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/call_cc_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/call_cc_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_call_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/call_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/call_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_di_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/di_timing-GS.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/di_timing-GS.bin"),
    );
}

#[test]
fn gekkio_acceptance_div_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/div_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/div_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_ei_sequence() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/ei_sequence.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/ei_sequence.bin"),
    );
}

#[test]
fn gekkio_acceptance_ei_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/ei_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/ei_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_halt_ime0_ei() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/halt_ime0_ei.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/halt_ime0_ei.bin"),
    );
}

#[test]
fn gekkio_acceptance_halt_ime0_nointr_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/halt_ime0_nointr_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/halt_ime0_nointr_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_halt_ime1_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/halt_ime1_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/halt_ime1_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_halt_ime1_timing2() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/halt_ime1_timing2-GS.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/halt_ime1_timing2-GS.bin"),
    );
}

#[test]
fn gekkio_acceptance_if_ie_registers() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/if_ie_registers.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/if_ie_registers.bin"),
    );
}

#[test]
fn gekkio_acceptance_intr_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/intr_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/intr_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_jp_cc_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/jp_cc_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/jp_cc_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_jp_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/jp_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/jp_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_ld_hl_sp_e_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/ld_hl_sp_e_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/ld_hl_sp_e_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_oam_dma_restart() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/oam_dma_restart.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/oam_dma_restart.bin"),
    );
}

#[test]
fn gekkio_acceptance_oam_dma_start() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/oam_dma_start.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/oam_dma_start.bin"),
    );
}

#[test]
fn gekkio_acceptance_oam_dma_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/oam_dma_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/oam_dma_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_pop_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/pop_timing.gb"
    ))
    .must_run_and_match(
        1_000_000u64,
        include_bytes!("gekkio/acceptance/pop_timing.bin"),
    );
}

#[test]
fn gekkio_acceptance_rapid_di_ei() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/rapid_di_ei.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/rapid_di_ei.bin"),
    );
}

#[test]
fn gekkio_acceptance_reti_intr_timing() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/reti_intr_timing.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/reti_intr_timing.bin"),
    );
}

/*
 * Gekkio's BITS acceptance tests
 */

#[test]
fn gekkio_acceptance_bits_mem_oam() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/bits/mem_oam.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/bits/mem_oam.bin"),
    );
}

#[test]
fn gekkio_acceptance_bits_reg_f() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/bits/reg_f.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/bits/reg_f.bin"),
    );
}

#[test]
fn gekkio_acceptance_bits_unused_hwio() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/bits/unused_hwio-GS.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/bits/unused_hwio-GS.bin"),
    );
}

/*
 * Gekkio's INSTR acceptance tests
 */

#[test]
fn gekkio_acceptance_instr_daa() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/instr/daa.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/instr/daa.bin"),
    );
}

/*
 * Gekkio's OAM DMA acceptance tests
 */

#[test]
fn gekkio_acceptance_oam_dma_basic() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/oam_dma/basic.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/oam_dma/basic.bin"),
    );
}

#[test]
fn gekkio_acceptance_oam_dma_reg_read() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/oam_dma/reg_read.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/oam_dma/reg_read.bin"),
    );
}

#[test]
fn gekkio_acceptance_oam_dma_sources() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/oam_dma/sources-dmgABCmgbS.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/oam_dma/sources-dmgABCmgbS.bin"),
    );
}

/*
 * Gekkio's TIMER acceptance tests
 */

#[test]
fn gekkio_acceptance_timer_div_write() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/div_write.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/div_write.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_rapid_toggle() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/rapid_toggle.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/rapid_toggle.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_tim00() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/tim00.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim00.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_tim00_div_trigger() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/tim00_div_trigger.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim00_div_trigger.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_tim01() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/tim01.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim01.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_tim01_div_trigger() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/tim01_div_trigger.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim01_div_trigger.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_tim10() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/tim10.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim10.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_tim10_div_trigger() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/tim10_div_trigger.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim10_div_trigger.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_tim11() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/tim11.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim11.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_tim11_div_trigger() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/tim11_div_trigger.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tim11_div_trigger.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_tima_reload() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/tima_reload.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tima_reload.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_tima_write_reloading() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/tima_write_reloading.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tima_write_reloading.bin"),
    );
}

#[test]
fn gekkio_acceptance_timer_tma_write_reloading() {
    RomTest::new(include_bytes!(
        "../assets/roms/gekkio/acceptance/timer/tma_write_reloading.gb"
    ))
    .must_run_and_match(
        4_000_000u64,
        include_bytes!("gekkio/acceptance/timer/tma_write_reloading.bin"),
    );
}
