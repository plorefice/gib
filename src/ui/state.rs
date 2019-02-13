use gib_core::{bus::Bus, cpu::CPU, dbg, GameBoy};

use failure::Error;

use std::path::{Path, PathBuf};

pub struct EmuState {
    gb: GameBoy,
    rom_file: PathBuf,
    snd_sample_rate: f32,

    turbo_mode: bool,
    step_to_next: bool,
    run_to_breakpoint: bool,
    trace_event: Option<dbg::TraceEvent>,
}

impl EmuState {
    pub fn new<P: AsRef<Path>>(rom: P, snd_sample_rate: f32) -> Result<EmuState, Error> {
        let mut gb = GameBoy::new(snd_sample_rate);
        let rom_buf = std::fs::read(rom.as_ref())?;

        gb.load_rom(&rom_buf[..])?;

        Ok(EmuState {
            gb,
            rom_file: rom.as_ref().to_path_buf(),
            snd_sample_rate,

            turbo_mode: false,
            step_to_next: false,
            run_to_breakpoint: false,
            trace_event: None,
        })
    }

    pub fn pause(&mut self) {
        self.turbo_mode = false;
        self.step_to_next = false;
        self.run_to_breakpoint = false;
        self.gb.cpu_mut().pause();
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
            let r = self.gb.step();
            self.pause();
            r
        } else if self.turbo_mode {
            self.gb.run_for_vblank()
        } else if self.run_to_breakpoint {
            self.run_to_audio_sync()
        } else {
            Ok(())
        };

        if let Err(ref evt) = res {
            self.trace_event = Some(*evt);
            self.pause();
        };
    }

    /// Runs the emulator until the audio queue is full, to avoid dropping
    /// audio samples and cause skipping/popping.
    fn run_to_audio_sync(&mut self) -> Result<(), dbg::TraceEvent> {
        let audio_queue = self.gb.get_sound_output();

        while audio_queue.len() < audio_queue.capacity() {
            self.gb.step()?;
        }

        Ok(())
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
    pub fn set_turbo(&mut self, enable: bool) {
        self.turbo_mode = enable;
    }

    pub fn paused(&mut self) -> bool {
        self.gb.cpu().paused() && !(self.step_to_next || self.run_to_breakpoint)
    }

    /// Returns true if turbo mode is enabled, false otherwise.
    pub fn turbo(&mut self) -> bool {
        self.turbo_mode
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        // Save breakpoints to restore after reset
        let bkps = self.cpu().breakpoints().clone();

        *self = EmuState::new(&self.rom_file, self.snd_sample_rate)?;
        for b in bkps.iter() {
            self.cpu_mut().set_breakpoint(*b);
        }

        // Default to running state
        self.set_running();

        Ok(())
    }

    pub fn gameboy(&self) -> &GameBoy {
        &self.gb
    }

    pub fn gameboy_mut(&mut self) -> &mut GameBoy {
        &mut self.gb
    }

    pub fn cpu(&self) -> &CPU {
        self.gb.cpu()
    }

    pub fn cpu_mut(&mut self) -> &mut CPU {
        self.gb.cpu_mut()
    }

    pub fn bus(&self) -> &Bus {
        self.gb.bus()
    }
}
