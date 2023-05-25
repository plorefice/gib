use std::{
    fs,
    time::{Duration, Instant},
};

use gib_core::GameBoy;

macro_rules! test_cases {
    (
        $(
            $name:ident($path:expr);
        )+
    ) => {
        $(
            #[test]
            fn $name() {
                run_test($path);
            }
        )+
    };
}

test_cases! {
    add_sp_e_timing("acceptance/add_sp_e_timing");
    boot_div_dmg_abc_mgb("acceptance/boot_div-dmgABCmgb");
    boot_regs_dmg_abc("acceptance/boot_regs-dmgABC");
    boot_hwio_dmg_abc_mgb("acceptance/boot_hwio-dmgABCmgb");
    call_cc_timing("acceptance/call_cc_timing");
    call_timing("acceptance/call_timing");
    di_timing_gs("acceptance/di_timing-GS");
    div_timing("acceptance/div_timing");
    ei_sequence("acceptance/ei_sequence");
    ei_timing("acceptance/ei_timing");
    halt_ime0_ei("acceptance/halt_ime0_ei");
    halt_ime0_nointr_timing("acceptance/halt_ime0_nointr_timing");
    halt_ime1_timing("acceptance/halt_ime1_timing");
    halt_ime1_timing2_gs("acceptance/halt_ime1_timing2-GS");
    if_ie_registers("acceptance/if_ie_registers");
    intr_timing("acceptance/intr_timing");
    jp_cc_timing("acceptance/jp_cc_timing");
    jp_timing("acceptance/jp_timing");
    ld_hl_sp_e_timing("acceptance/ld_hl_sp_e_timing");
    oam_dma_restart("acceptance/oam_dma_restart");
    oam_dma_start("acceptance/oam_dma_start");
    oam_dma_timing("acceptance/oam_dma_timing");
    pop_timing("acceptance/pop_timing");
    rapid_di_ei("acceptance/rapid_di_ei");
    reti_intr_timing("acceptance/reti_intr_timing");

    bits_mem_oam("acceptance/bits/mem_oam");
    bits_reg_f("acceptance/bits/reg_f");
    bits_unused_hwio_gs("acceptance/bits/unused_hwio-GS");

    instr_daa("acceptance/instr/daa");

    oam_dma_basic("acceptance/oam_dma/basic");
    oam_dma_reg_read("acceptance/oam_dma/reg_read");
    oam_dma_sources_gs("acceptance/oam_dma/sources-GS");

    timer_div_write("acceptance/timer/div_write");
    timer_rapid_toggle("acceptance/timer/rapid_toggle");
    timer_tim00("acceptance/timer/tim00");
    timer_tim00_div_trigger("acceptance/timer/tim00_div_trigger");
    timer_tim01("acceptance/timer/tim01");
    timer_tim01_div_trigger("acceptance/timer/tim01_div_trigger");
    timer_tim10("acceptance/timer/tim10");
    timer_tim10_div_trigger("acceptance/timer/tim10_div_trigger");
    timer_tim11("acceptance/timer/tim11");
    timer_tim11_div_trigger("acceptance/timer/tim11_div_trigger");
    timer_tima_reload("acceptance/timer/tima_reload");
    timer_tima_write_reloading("acceptance/timer/tima_write_reloading");
    timer_tma_write_reloading("acceptance/timer/tma_write_reloading");
}

fn run_test(name: &str) {
    let rom =
        fs::read(format!("assets/roms/gekkio/{name}.gb")).expect("failed to load test binary");

    let mut gameboy = GameBoy::new();
    gameboy.load_rom(&rom).unwrap();

    let start = Instant::now();
    let timeout = Duration::from_secs(5);

    while start.elapsed() < timeout {
        gameboy.run_for_vblank().expect("unexpected trace event");

        let cpu = gameboy.cpu();
        if cpu.b() == 3
            && cpu.c() == 5
            && cpu.d() == 8
            && cpu.e() == 13
            && cpu.h() == 21
            && cpu.l() == 34
        {
            return;
        }
    }

    panic!("Test failed");
}
