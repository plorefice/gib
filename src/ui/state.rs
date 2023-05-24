use std::{fs, path::Path};

use anyhow::Error;
use gib_core::{bus::Bus, cpu::Cpu, dbg, AudioSource, GameBoy};

pub struct Emulator {
    gameboy: GameBoy,
    turbo_mode: bool,
    step_to_next: bool,
    run_to_breakpoint: bool,
    trace_event: Option<dbg::TraceEvent>,
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            gameboy: GameBoy::new(),
            turbo_mode: false,
            step_to_next: false,
            run_to_breakpoint: false,
            trace_event: None,
        }
    }
}

impl Emulator {
    pub fn load_rom<P: AsRef<Path>>(&mut self, rom: P) -> Result<(), Error> {
        self.gameboy.load_rom(&(fs::read(rom)?)[..])?;
        self.reset();
        Ok(())
    }

    pub fn pause(&mut self) {
        self.turbo_mode = false;
        self.step_to_next = false;
        self.run_to_breakpoint = false;
        self.gameboy.cpu_mut().pause();
    }

    /// Performs a single emulation step, depending on the emulator's state:
    ///
    /// * if we are in step mode, execute a single instruction
    /// * if we are in run mode, run to audio sync (ie. audio queue full)
    ///
    /// In both cases, if an event happens, pause the emulator.
    pub fn do_step(&mut self) {
        if self.paused() {
            return;
        }

        self.trace_event = None;

        let res = if self.step_to_next {
            let r = self.gameboy.step();
            self.pause();
            r
        } else if self.run_to_breakpoint {
            self.gameboy.run_for_vblank()
        } else {
            Ok(())
        };

        if let Err(ref evt) = res {
            tracing::error!(%evt, "Trace event occurred");
            self.trace_event = Some(*evt);
            self.pause();
        };
    }

    /// Configures the emulator's audio channel.
    pub fn configure_audio_channel(&mut self, source: AudioSource, sample_rate: f32) {
        self.gameboy.configure_audio_channel(source, sample_rate);
    }

    pub fn last_event(&self) -> &Option<dbg::TraceEvent> {
        &self.trace_event
    }

    pub fn set_single_step(&mut self) {
        self.step_to_next = true;
    }

    pub fn set_running(&mut self) {
        self.run_to_breakpoint = true;
    }

    /// Sets or resets turbo mode.
    ///
    /// In turbo mode, the emulator runs to video-sync rather than audio-sync,
    /// likely dropping audio samples.
    pub fn set_turbo(&mut self, turbo: bool) {
        self.gameboy.enable_audio_sync(!turbo);
    }

    pub fn paused(&mut self) -> bool {
        self.gameboy.cpu().paused() && !(self.step_to_next || self.run_to_breakpoint)
    }

    /// Reset the emulator's sate.
    pub fn reset(&mut self) {
        self.gameboy.reset();
        self.set_running();
    }

    pub fn gameboy(&self) -> &GameBoy {
        &self.gameboy
    }

    pub fn gameboy_mut(&mut self) -> &mut GameBoy {
        &mut self.gameboy
    }

    pub fn cpu(&self) -> &Cpu {
        self.gameboy.cpu()
    }

    pub fn cpu_mut(&mut self) -> &mut Cpu {
        self.gameboy.cpu_mut()
    }

    pub fn bus(&self) -> &Bus {
        self.gameboy.bus()
    }
}
